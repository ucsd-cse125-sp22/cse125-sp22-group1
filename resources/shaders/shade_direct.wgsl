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
var t_color: texture_2d<f32>;

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

fn linear_to_srgb_color(x: vec3<f32>) -> vec3<f32> {
	return vec3<f32>(
		linear_to_srgb(x.r), linear_to_srgb(x.g), 
		linear_to_srgb(x.b)
	);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let shadow_size = textureDimensions(t_shadow);
	let shadow_sizef = vec2<f32>(shadow_size);

	let tc = vec2<i32>(in.xy);
	let tcn = in.xy / surface_sizef;

	let tc_shadow = vec2<i32>(tcn * shadow_sizef);

	let light_dir = vec3<f32>(-0.5, -1.0, 0.5);
	let world_normal = textureLoad(t_normal, tc, 0).xyz * 2.0 - 1.0;

	//let world_normal = normalize((view.normal_to_world * vec4<f32>(local_normal, 0.0)).xyz);
	let color_alpha = textureLoad(t_color, tc, 0);
	let color = linear_to_srgb_color(color_alpha.rgb);
	//let color = textureLoad(t_color, tc, 0).rgb;
	let diffuse = max(dot(world_normal, -light_dir), 0.0);

	let depth = textureLoad(t_depth, tc, 0).r;
	let view_pos = view_pos_from_depth(tcn, depth);
	let world_pos = (view.inv_view * vec4<f32>(view_pos, 1.0)).xyz;
	//let world_pos = world_pos_from_depth(tcn, depth);
	let light_pos = world_pos_to_light_pos(world_pos);
	
	var light_depth: f32 = 100.0;
	if (light_pos.x > 0.0 && light_pos.x < 1.0 && light_pos.y > 0.0 && light_pos.y < 1.0) {
		light_depth = textureLoad(t_shadow, vec2<i32>(light_pos.xy * shadow_sizef), 0).r;
	}
	
	var shadow = 1.0; 
	if (light_pos.z > light_depth - 0.00004) {
		shadow = 0.0; 
	}

	let light_color = vec3<f32>(1.0, 0.584, 0.521);
	let ambient_color = vec3<f32>(0.39, 0.57, 1.0);
	let ambient = vec3<f32>(0.06);
	let shaded = (shadow * diffuse * light_color + ambient * ambient_color) * color;

	return vec4<f32>(shaded * 0.7, color_alpha.a);
}