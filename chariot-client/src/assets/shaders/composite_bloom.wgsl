[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_shade_direct_color: texture_2d<f32>;

[[group(0), binding(1)]]
var t_color: texture_2d<f32>;

[[group(0), binding(2)]]
var t_normal: texture_2d<f32>;

[[group(0), binding(3)]]
var t_depth: texture_2d<f32>;

[[group(0), binding(4)]]
var t_blur_color: texture_2d<f32>;

[[group(0), binding(5)]]
var t_hibl_debayer: texture_2d<f32>;

[[group(0), binding(6)]]
var s_color: sampler;

fn aces_film(x: vec3<f32>) -> vec3<f32>
{
    let a: f32 = 2.51;
    let b: f32 = 0.03;
    let c: f32 = 2.43;
    let d: f32 = 0.59;
    let e: f32 = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), vec3<f32>(0.0), vec3<f32>(1.0));
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_shade_direct_color);
	let surface_sizef = vec2<f32>(surface_size);

	let surface_size_us = textureDimensions(t_color);
	let surface_sizef_us = vec2<f32>(surface_size_us);

	let tc = vec2<i32>(in.xy);
	let tc_us = tc * vec2<i32>(2, 2);

	let tcn = vec2<f32>(tc) / surface_sizef;

	let shade_direct_color = textureLoad(t_shade_direct_color, tc, 0).rgb;
	let blur_color = textureSample(t_blur_color, s_color, tcn).rgb;
	let color = textureLoad(t_color, tc_us, 0).rgb;

	let normal_mat_id = textureLoad(t_normal, tc_us, 0);
	let mat_id = normal_mat_id.z;
	
	let z_near = 0.1;
	let z_far = 1000.0;
	let depth = textureLoad(t_depth, tc_us, 0).r;
    let z_ndc = 2.0 * depth - 1.0;
    let z = 2.0 * z_near * z_far / (z_far + z_near - z_ndc * (z_far - z_near));

	let hibl_tc = tc;
	let hibl_tcn = vec2<f32>(hibl_tc) / (4.0 * surface_sizef);
	let irradiance_ao = textureSample(t_hibl_debayer, s_color, hibl_tcn);
	let irradiance = irradiance_ao.rgb;
	let ao = irradiance_ao.a;

	let ambient_color = vec3<f32>(0.39, 0.57, 1.0) * 1.2;
	let ambient_factor = 0.1;
	//let color_out = (color.rgb + blur_color.rgb) * ao; //(color.rgb + blur_color.rgb) * irradiance;
	let shaded = shade_direct_color + color * (irradiance + ambient_color * ambient_factor * ao) + blur_color;

	var fog_factor = exp(-0.002 * z);
	if (mat_id > 0.5 || depth == 1.0) {
		fog_factor = 1.0;
	}

	let fog_color = vec3<f32>(1.0, 0.384, 0.221) * 0.6;
	let fog_shaded = fog_factor * shaded + (1.0 - fog_factor) * fog_color;

	return vec4<f32>(fog_shaded, 1.0);
}