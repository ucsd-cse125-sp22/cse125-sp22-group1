[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

struct ViewInfo {
	inv_view: mat4x4<f32>;
	inv_proj: mat4x4<f32>;
};

struct LightInfo {
	view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var t_forward: texture_2d<f32>;

[[group(0), binding(1)]]
var t_normal: texture_2d<f32>;

[[group(0), binding(2)]]
var t_depth: texture_2d<f32>;

[[group(0), binding(3)]]
var t_shadow: texture_2d<f32>;

[[group(0), binding(4)]]
var t_probes_color: texture_2d<f32>;

[[group(0), binding(5)]]
var t_probes_depth: texture_2d<f32>;

[[group(0), binding(6)]]
var s_probes: sampler;

[[group(1), binding(0)]]
var<uniform> view: ViewInfo;

[[group(2), binding(0)]]
var<uniform> light: LightInfo;

struct VarianceInfo {
	count: i32;
	mean: vec4<f32>;
	m2: vec4<f32>;
};

fn acc_var(acc: VarianceInfo, new: vec4<f32>) -> VarianceInfo {
	var res: VarianceInfo;
	res.count = acc.count + 1;

	let vcount = vec4<f32>(f32(res.count));
	let delta = new - acc.mean;
	res.mean = acc.mean + (delta / vcount);
	let delta2 = new - res.mean;
	res.m2 = acc.m2 + delta * delta2;

	return res;
}

fn world_pos_from_depth(tex_coord: vec2<f32>, depth: f32) -> vec3<f32> {
	let clip_space_pos = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_pos_h = view.inv_proj * clip_space_pos;
	let view_space_pos = vec4<f32>(view_space_pos_h.xyz / view_space_pos_h.w, 1.0);
	let world_space_pos = view.inv_view * view_space_pos;
	return world_space_pos.xyz;
}

fn world_pos_to_light_pos(world_pos: vec3<f32>) -> vec3<f32> {
	let light_pos_h = light.view_proj * vec4<f32>(world_pos, 1.0);
	let light_pos_ndc = light_pos_h.xyz / light_pos_h.w;
	let light_pos = vec3<f32>(light_pos_ndc.x * 0.5 + 0.5, 1.0 - (light_pos_ndc.y * 0.5 + 0.5), light_pos_ndc.z);
	return light_pos;
}

fn srgb_to_linear(x: f32) -> f32{
	if (x <= 0.0) {
		return 0.0;
	} else if (x >= 1.0) {
		return 1.0;
	} else if (x < 0.04045) {
		return x / 12.92;
	} else {
		return pow((x + 0.055) / 1.055, 2.4);
	}
}

fn srgb_to_linear_color(x: vec4<f32>) -> vec4<f32> {
	return vec4<f32>(
		srgb_to_linear(x.r), srgb_to_linear(x.g), 
		srgb_to_linear(x.b), srgb_to_linear(x.a)
	);
}

fn linear_to_srgb(x: f32) -> f32{
	if (x <= 0.0) {
		return 0.0;
	} else if (x >= 1.0) {
		return 1.0;
	} else if (x < 0.0031308) {
		return x * 12.92;
	} else {
		return pow(x, 1.0 / 2.4) * 1.055 - 0.055;
	}
}

fn linear_to_srgb_color(x: vec4<f32>) -> vec4<f32> {
	return vec4<f32>(
		linear_to_srgb(x.r), linear_to_srgb(x.g), 
		linear_to_srgb(x.b), linear_to_srgb(x.a)
	);
}

fn aces_film(x: vec3<f32>) -> vec3<f32>
{
    let a: f32 = 2.51;
    let b: f32 = 0.03;
    let c: f32 = 2.43;
    let d: f32 = 0.59;
    let e: f32 = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn calc_normal_variance(tc: vec2<i32>) -> f32 {
	var acc : VarianceInfo;
	acc.count = 0;
	acc.mean = vec4<f32>(0.0);
	acc.m2 = vec4<f32>(0.0);

	var tmp : vec4<f32>;
	tmp = textureLoad(t_normal, tc + vec2<i32>(-2, -2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, -2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0,  -2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1,  -2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(2, -2), 0);
	acc = acc_var(acc, tmp);

	tmp = textureLoad(t_normal, tc + vec2<i32>(-2, -1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, -1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0, -1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1, -1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(2, -1), 0);
	acc = acc_var(acc, tmp);

	tmp = textureLoad(t_normal, tc + vec2<i32>(-2, 0), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, 0), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0,  0), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1,  0), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(2, 0), 0);
	acc = acc_var(acc, tmp);
	
	tmp = textureLoad(t_normal, tc + vec2<i32>(-2, 1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, 1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0,  1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1,  1), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(2, 1), 0);
	acc = acc_var(acc, tmp);

	tmp = textureLoad(t_normal, tc + vec2<i32>(-2, 2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, 2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0,  2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1,  2), 0);
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(2, 2), 0);
	acc = acc_var(acc, tmp);

	let variance = length(acc.m2 / vec4<f32>(f32(acc.count)));
}

var<private> rng_state: u32;

fn pcg_hash(input: u32) -> u32 {
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn pcg_next_u32() -> u32 {
    let state = rng_state;
    rng_state = rng_state * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn pcg_next_f32() -> f32 {
	let u = (pcg_next_u32() >> 9u) | 0x3f800000u;
    let f = bitcast<f32>(u);
    return f - 1.0;
}

fn concentric_sample_disk(u: vec2<f32>) -> vec2<f32> {
	let pi = 3.14159274;
	let pi_over_2 = pi / 2.0;
	let pi_over_4 = pi / 4.0;

	let u_offset = 2.0 * u - vec2<f32>(1.0, 1.0);
	if (u_offset.x == 0.0 && u_offset.y == 0.0) {
        return vec2<f32>(0.0, 0.0);
	}

	var r: f32;
	var theta: f32;
	if (abs(u_offset.x) > abs(u_offset.y)) {
		r = u_offset.x;
		theta = pi_over_4 * (u_offset.y / u_offset.x);
	} else {
		r = u_offset.y;
		theta = pi_over_2 - pi_over_4 * (u_offset.x / u_offset.y);
	}
	return r * vec2<f32>(cos(theta), sin(theta));
}

fn cosine_sample_hemisphere(u: vec2<f32>) -> vec3<f32> {
    let d = concentric_sample_disk(u);
    let z = sqrt(max(0.0, 1.0 - d.x * d.x - d.y * d.y));
    return vec3<f32>(d.x, d.y, z);
}

fn cosine_hemisphere_pdf(v: vec3<f32>) -> f32 {
	let pi = 3.14159274;
	let inv_pi = 1.0 / pi;

	let cos_theta = v.z;
    return cos_theta * inv_pi;
}

struct TangentFrame {
	n: vec3<f32>;
	b0: vec3<f32>;
	b1: vec3<f32>;
};

// https://graphics.pixar.com/library/OrthonormalB/paper.pdf
fn calc_ONB(n: vec3<f32>) -> TangentFrame {
	var frame: TangentFrame;
	var sign = 1.0;
	if(n.z < 0.0) {
		sign = -1.0;
	}
	let a = -1.0 / (sign + n.z);
	let b = n.x * n.y * a;
	frame.n = n;
	frame.b0 = vec3<f32>(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x);
	frame.b1 = vec3<f32>(b, sign + n.y * n.y * a, -n.y);
	return frame;
}


fn world_to_frame(v: vec3<f32>, frame: TangentFrame) -> vec3<f32> {
	let x = dot(v, frame.b0);
	let y = dot(v, frame.b1);
	let z = dot(v, frame.n);
	return vec3<f32>(x, y, z);
}

fn frame_to_world(v: vec3<f32>, frame: TangentFrame) -> vec3<f32> {
	return v.x * frame.b0 + v.y * frame.b1 + v.z * frame.n;
}

fn oct_wrap(v: vec2<f32>) -> vec2<f32>
{
    return (1.0 - abs(v.yx)) * sign(v.xy);
}

// outputs in (-1, 1)
// https://knarkowicz.wordpress.com/2014/04/16/octahedron-normal-vector-encoding/
fn oct_encode(n: vec3<f32>) -> vec2<f32> {
	var nn = n / (abs(n.x) + abs(n.y) + abs(n.z));
	if (nn.z < 0.0) {
		let wrapped = oct_wrap(nn.xy);
		nn.x = wrapped.x;
		nn.y = wrapped.y;
	}

	return nn.xy;
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let shadow_size = textureDimensions(t_shadow);
	let shadow_sizef = vec2<f32>(shadow_size);

	let tc = vec2<i32>(in.xy);
	let tcn = in.xy / surface_sizef;

	let hash_input = u32(tc.y * surface_size.x + tc.x);
	rng_state = pcg_hash(hash_input);

	let tc_shadow = vec2<i32>(tcn * shadow_sizef);

	let variance = calc_normal_variance(tc);
	let edge_col = vec3<f32>(0.29, 0.22, 0.06);
	let edge_shade = pow(variance, 0.7);

	let light_dir = vec3<f32>(-0.5, -1.0, 0.5);
	let world_normal = textureLoad(t_normal, tc, 0).xyz;

	//let world_normal = normalize((view.normal_to_world * vec4<f32>(local_normal, 0.0)).xyz);
	let color = linear_to_srgb_color(textureLoad(t_forward, tc, 0)).rgb;
	//let color = textureLoad(t_forward, tc, 0).rgb;
	let diffuse = max(dot(world_normal, -light_dir), 0.0);

	let depth = textureLoad(t_depth, tc, 0).r;
	let world_pos = world_pos_from_depth(tcn, depth);
	let light_pos = world_pos_to_light_pos(world_pos);
	
	var light_depth: f32 = 100.0;
	if (light_pos.x > 0.0 && light_pos.x < 1.0 && light_pos.y > 0.0 && light_pos.y < 1.0) {
		light_depth = textureLoad(t_shadow, vec2<i32>(light_pos.xy * shadow_sizef), 0).r;
	}
	
	var shadow = vec3<f32>(1.0); 
	if (light_pos.z > light_depth - 0.00004) {
		shadow = vec3<f32>(0.0); 
	}

	let probe_size = 8;
	let probe_center = probe_size / 2;
	let probe_centerf = f32(probe_center);
	let probe_2d_idx = (tc + probe_center) / probe_size;

	let irradiance_samples = 10;
	var irradiance = vec3<f32>(0.0);
	for (var sample_idx: i32 = 0; sample_idx < irradiance_samples; sample_idx = sample_idx + 1) {
		let sample_frame = calc_ONB(world_normal);

		let sample_local_dir = cosine_sample_hemisphere(vec2<f32>(pcg_next_f32(), pcg_next_f32()));
		let sample_world_dir = frame_to_world(sample_local_dir, sample_frame);

		let oct_coords_n = oct_encode(sample_world_dir);
		let oct_coords = vec2<i32>(oct_coords_n * vec2<f32>(probe_centerf, probe_centerf));

		for(var probe_offset_idx: i32 = 0; probe_offset_idx < 4; probe_offset_idx = probe_offset_idx + 1) {
			let probe_offset = vec2<i32>(probe_offset_idx % 2, probe_offset_idx / 2);
			let cur_probe_idx = probe_2d_idx + probe_offset; 
			let cur_probe_tc = cur_probe_idx * probe_size + probe_center;
			let cur_probe_tcn = vec2<f32>(cur_probe_tc) / surface_sizef;

			let sample_tcn = vec2<f32>(cur_probe_tc + oct_coords) / surface_sizef;
			let sample_radiance = textureSample(t_probes_color, s_probes, sample_tcn).rgb;
			
			let sample_pdf = cosine_hemisphere_pdf(sample_local_dir);
			irradiance = irradiance + sample_radiance / sample_pdf;
		}
	}
	irradiance = irradiance * 0.1 / f32(irradiance_samples);

	let ambient = vec3<f32>(0.06);
	let shaded = ((shadow * diffuse) + irradiance + ambient) * color * (1.0 - edge_shade) + edge_shade * edge_col;

	//let s_col = textureLoad(t_probes_color, tc, 0);
	//return vec4<f32>(irradiance * color, 1.0);
	//return s_col;
	//return vec4<f32>(color, 1.0);
	return vec4<f32>(aces_film(shaded), 1.0);
	//return vec4<f32>((world_normal+1.0)*0.5, 1.0);
}
