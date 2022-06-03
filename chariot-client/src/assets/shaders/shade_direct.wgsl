[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

struct ViewInfo {
	inv_view: mat4x4<f32>;
	inv_proj: mat4x4<f32>;
};

struct LightInfo {
	view: mat4x4<f32>;
	proj: mat4x4<f32>;
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
	let local_pos = light.view * vec4<f32>(world_pos, 1.0);
	let light_pos_h = light.proj * local_pos;
	let light_pos_ndc = light_pos_h.xyz / light_pos_h.w;
	let light_pos = vec3<f32>(light_pos_ndc.x * 0.5 + 0.5, 1.0 - (light_pos_ndc.y * 0.5 + 0.5), -local_pos.z);
	return light_pos;
}

fn calc_light_coverage(depth_depth2: vec2<f32>, pixel_depth: f32) -> f32
{
    let variance = depth_depth2.y - depth_depth2.x * depth_depth2.x;
    let diff = (pixel_depth - depth_depth2.x) * 10.0;
    if(diff > 0.0)
    {
        return clamp(variance / (variance + diff * diff), 0.0, 1.0);
    }
    else
    {
        return 1.0;
    }
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

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let shadow_size = textureDimensions(t_shadow);
	let shadow_sizef = vec2<f32>(shadow_size);

	let tc = vec2<i32>(in.xy);
	let tcn = in.xy / surface_sizef;

	let tc_shadow = vec2<i32>(tcn * shadow_sizef);

	let light_dir = normalize(vec3<f32>(-1.0, -0.5, 0.1)); //vec3<f32>(-0.5, -1.0, 0.5);
	let oct_normal_matid = textureLoad(t_normal, tc, 0).xyz;
	let mat_id = oct_normal_matid.z;
	let oct_normal = oct_normal_matid.xy * 2.0 - 1.0;
	let view_normal = oct_decode(oct_normal);
	let world_normal = normalize((view.inv_view * vec4<f32>(view_normal, 0.0)).xyz);

	//let world_normal = normalize((view.normal_to_world * vec4<f32>(local_normal, 0.0)).xyz);
	let color_alpha = textureLoad(t_color, tc, 0);
	let color = linear_to_srgb_color(color_alpha.rgb);
	//let color = textureLoad(t_color, tc, 0).rgb;
	var diffuse = max(dot(world_normal, -light_dir), 0.0);
	if (mat_id > 0.1) {
		diffuse = 1.0;
	}


	let depth = textureLoad(t_depth, tc, 0).r;
	let view_pos = view_pos_from_depth(tcn, depth);
	let world_pos = (view.inv_view * vec4<f32>(view_pos, 1.0)).xyz;
	//let world_pos = world_pos_from_depth(tcn, depth);
	let light_pos = world_pos_to_light_pos(world_pos);
	
	var light_depth = vec2<f32>(1000.0, 1000.0 * 1000.0);
	if (light_pos.x > 0.0 && light_pos.x < 1.0 && light_pos.y > 0.0 && light_pos.y < 1.0) {
		light_depth = textureLoad(t_shadow, vec2<i32>(light_pos.xy * shadow_sizef), 0).rg;
	}
	
	//var shadow = 1.0; 
	//if (light_pos.z < light_depth.r - 0.00004 && mat_id < 0.5) {
	//	shadow = 0.0; 
	//}
	let shadow = calc_light_coverage(light_depth, light_pos.z - 0.4);

	let light_color = vec3<f32>(1.0, 0.584, 0.521) * 0.4;
	let ambient_color = vec3<f32>(0.39, 0.57, 1.0);
	let fog_color = vec3<f32>(1.0, 0.384, 0.221) * 0.8; 

	var fog_factor = exp(-0.004 * length(view_pos));
	if (mat_id > 0.5) {
		fog_factor = 1.0;
	}

	let ambient = vec3<f32>(0.1);
	var shaded = (shadow * diffuse * light_color) * color;

	if (mat_id > 0.1) {
		shaded = shaded * color.r * 3.0;
	}

	let fog_shaded = fog_factor * shaded + (1.0 - fog_factor) * fog_color;

	return vec4<f32>(shaded * 0.7, color_alpha.a);
}