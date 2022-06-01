[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

struct ViewInfo {
	inv_view: mat4x4<f32>;
	inv_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[group(0), binding(1)]]
var t_normal: texture_2d<f32>;

[[group(0), binding(2)]]
var t_depth: texture_2d<f32>;


[[group(1), binding(0)]]
var<uniform> view: ViewInfo;

fn world_pos_from_depth(tex_coord: vec2<f32>, depth: f32) -> vec3<f32> {
	let clip_space_pos = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_pos_h = view.inv_proj * clip_space_pos;
	let view_space_pos = vec4<f32>(view_space_pos_h.xyz / view_space_pos_h.w, 1.0);
	let world_space_pos = view.inv_view * view_space_pos;
	return world_space_pos.xyz;
}

fn view_pos_from_depth(tex_coord: vec2<f32>, depth: f32) -> vec3<f32> {
	let clip_space_pos = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_pos_h = view.inv_proj * clip_space_pos;
	let view_space_pos = vec4<f32>(view_space_pos_h.xyz / view_space_pos_h.w, 1.0);
	return view_space_pos.xyz;
}

fn view_dir(tex_coord: vec2<f32>) -> vec3<f32> {
	let depth = 1.0;
	let clip_space_dir = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_dir_h = view.inv_proj * clip_space_dir;
	let view_space_dir = vec4<f32>(view_space_dir_h.xyz / view_space_dir_h.w, 0.0);
	return normalize(view_space_dir.xyz);
}

fn oct_decode(f: vec2<f32>) -> vec3<f32> {
    // https://twitter.com/Stubbesaurus/status/937994790553227264
    var n = vec3<f32>(f.x, f.y, 1.0 - abs(f.x) - abs(f.y));
    let t = clamp(-n.z, 0.0, 1.0);

	if (n.x >= 0.0) {
		n.x = n.x - t;
	} else {
		n.x = n.x + t;
	}

	if (n.y >= 0.0) {
		n.y = n.y - t;
	} else {
		n.y = n.y + t;
	}

    return normalize(n);
}

let pi: f32 = 3.14159265359;

// modified from https://github.com/Patapom/GodComplex/blob/master/Tests/TestHBIL/TestHBILForm.cs
// 2x2 bayer matrix
fn B2(x: u32, y: u32) -> u32 {
	return ((y << 1u) + x + 1u) & 3u;
}

// Generates the 4x4 matrix
// Expects _P any pixel coordinate
fn B4(x: u32, y: u32) -> u32 {
	return (B2(x & 1u, x & 1u) << 2u)
		+ B2((x >> 1u) & 1u, (y >> 1u) & 1u);
}


// super super lazy version of HIBL: https://github.com/Patapom/GodComplex/blob/master/Tests/TestHBIL/2018%20Mayaux%20-%20Horizon-Based%20Indirect%20Lighting%20(HBIL).pdf
// a lot of things missing but gosh it has the general idea
fn update_horizon(nei_screen_pos: vec2<f32>, view_pos: vec3<f32>, cos_sin_alpha: vec2<f32>, view_normal: vec3<f32>, cos_theta: ptr<function, f32>) -> vec3<f32> {
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let nei_tc = vec2<i32>(nei_screen_pos);
	let nei_tcn = vec2<f32>(nei_tc) / surface_sizef;
	let nei_depth = textureLoad(t_depth, nei_tc, 0).r;
	let nei_view_pos = view_pos_from_depth(nei_tcn, nei_depth);
	let nei_z = nei_view_pos.z;

	let z = view_pos.z - 0.01;
	let z_diff = z - nei_z;
	if (distance(view_pos, nei_view_pos) > 1.0) { // reject if too far away
		return vec3<f32>(0.0, 0.0, 0.0);
	}

	let r_diff = distance(view_pos.xy, nei_view_pos.xy);

	let inv_hypo = inverseSqrt(z_diff * z_diff + r_diff * r_diff);
	let new_cos_theta = (cos_sin_alpha.x * z_diff + cos_sin_alpha.y * r_diff) * inv_hypo;

	if (new_cos_theta < *cos_theta) {
		return vec3<f32>(0.0, 0.0, 0.0);
	}

	*cos_theta = new_cos_theta;

	let dir_in = nei_view_pos - view_pos;
	let diffuse = max(0.0, dot(dir_in, view_normal));
	let radiance = diffuse * textureLoad(t_color, nei_tc, 0).rgb;
	return radiance;
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let tc = vec2<i32>(in.xy);
	let tcu = vec2<u32>(in.xy);
	let tcn = in.xy / surface_sizef;

	let depth = textureLoad(t_depth, tc, 0).r;
	if (depth == 1.0) { // skybox exit
		return vec4<f32>(1.0, 1.0, 1.0, 1.0);
	}


	let view_pos = view_pos_from_depth(tcn, depth);
	let z = view_pos.z - 0.01;

	let oct_normal_matid = textureLoad(t_normal, tc, 0).xyz;
	let mat_id = oct_normal_matid.z;
	if (mat_id > 0.1) {
		return vec4<f32>(1.0, 1.0, 1.0, 1.0);
	}

	let oct_normal = oct_normal_matid.xy * 2.0 - 1.0;
	let view_normal = oct_decode(oct_normal);
	let view_dir = view_dir(tcn);

	let cos_sin_alpha = vec2<f32>(view_dir.z, sqrt(1.0 - view_dir.z * view_dir.z));

	let phi = f32(B4(tcu.x, tcu.y)) * 2.0 * pi / 16.0;
	var slice_dir = vec2<f32>(cos(phi), sin(phi));
	let t = -dot(slice_dir, view_normal.xy) / view_normal.z;

	let max_samples = 10;
	let sphere_radius = 300.0;
	let num_samples = min(max_samples, i32(sphere_radius / length(view_pos)));

	if (length(f32(num_samples) * slice_dir) < sphere_radius / length(view_pos)) {
		slice_dir = slice_dir * (sphere_radius / length(view_pos)) / length(f32(num_samples) * slice_dir);
	}

	var total_irradiance = vec3<f32>(0.0, 0.0, 0.0);

	var cos_theta_front = 0.0; //t / sqrt(1.0 + t * t);
	var screen_pos_front = in.xy;
	for(var i: i32 = 0; i < num_samples; i = i + 1) {
		screen_pos_front = screen_pos_front + slice_dir;
		let slice_irradiance = update_horizon(screen_pos_front, view_pos, cos_sin_alpha, view_normal, &cos_theta_front);
		total_irradiance = total_irradiance + slice_irradiance;
	}

	let cos_sin_alpha_back = vec2<f32>(cos_sin_alpha.x, cos_sin_alpha.y);
	var cos_theta_back = 0.0; //t / sqrt(1.0 + t * t);
	var screen_pos_back = in.xy;
	for(var i: i32 = 0; i < num_samples; i = i + 1) {
		screen_pos_back = screen_pos_back - slice_dir;
		let slice_irradiance = update_horizon(screen_pos_back, view_pos, cos_sin_alpha_back, view_normal, &cos_theta_back);
		total_irradiance = total_irradiance + slice_irradiance;
	}


	let ao = (2.0 - cos_theta_back - cos_theta_front);
	let shaded = vec3<f32>(ao, ao, ao); //total_irradiance * ao;
	return vec4<f32>(shaded, 1.0);
	//return vec4<f32>(view_dir * 0.5 + 0.5, 1.0);
}