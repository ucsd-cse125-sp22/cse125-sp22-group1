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


[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let shadow_size = textureDimensions(t_shadow);
	let shadow_sizef = vec2<f32>(shadow_size);

	let tc = vec2<i32>(in.xy);
	let tcn = in.xy / surface_sizef;

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
	let edge_col = vec3<f32>(0.29, 0.22, 0.06);
	let edge_shade = edge_col * variance;

	let light_dir = vec3<f32>(-0.5, -1.0, 0.5);
	let world_normal = textureLoad(t_normal, tc + vec2<i32>(0,  0), 0).xyz;

	//let world_normal = normalize((view.normal_to_world * vec4<f32>(local_normal, 0.0)).xyz);
	let color = textureLoad(t_forward, tc, 0).rgb;
	let diffuse = max(dot(world_normal, -light_dir), 0.0) * color;

	//return vec4<f32>(variance) + diffuse;
	let depth = textureLoad(t_depth, tc, 0).r;
	let world_pos = world_pos_from_depth(tcn, depth);
	let light_pos = world_pos_to_light_pos(world_pos);
	let light_depth = textureLoad(t_shadow, vec2<i32>(light_pos.xy * shadow_sizef), 0).r;
	
	var shadow = vec3<f32>(1.0); 
	if (light_pos.z > light_depth - 0.00004) {
		shadow = vec3<f32>(0.0); 
	}

	let ambient = vec3<f32>(0.02);
	let shaded = shadow * diffuse + ambient + edge_shade;

	return vec4<f32>(shaded, 1.0);
	//return vec4<f32>((world_normal+1.0)*0.5, 1.0);
}
