[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

struct ViewInfo {
	inv_view: mat4x4<f32>;
	inv_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: ViewInfo;

fn world_dir_from_depth(tex_coord: vec2<f32>) -> vec3<f32> {
	let depth = 1.0;
	let clip_space_dir = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_dir_h = view.inv_proj * clip_space_dir;
	let view_space_dir = vec4<f32>(view_space_dir_h.xyz / view_space_dir_h.w, 0.0);
	let world_space_dir = view.inv_view * view_space_dir;
	return normalize(world_space_dir.xyz);
}

// modified from: https://www.shadertoy.com/view/Ml2cWG

let pi: f32 = 3.14159265359;
let inv_pi: f32 = 0.318309886;

let zenith_offset: f32 = 0.1;
let multi_scatter_phase: f32 = 0.01;
let density: f32 = 0.9;

let anisotropic_intensity: f32 = 0.0; //Higher numbers result in more anisotropic scattering

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
	let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn zenith_density(x: f32) -> f32 {
	return density / pow(max(x - zenith_offset, 0.35e-2), 0.75);
}

fn calc_sky_absorption(x: vec3<f32>, y: f32) -> vec3<f32> {
	var absorption = x * -y;
	absorption = exp2(absorption) * 2.0;
	return absorption;
}

fn calc_sun_point(p: vec2<f32>, lp: vec2<f32>) -> f32 {
	return smoothstep(0.03, 0.026, distance(p, lp)) * 50.0;
}

fn calc_rayleigh_mult(p: vec2<f32>, lp: vec2<f32>) -> f32{
	return 1.0 + pow(1.0 - clamp(distance(p, lp), 0.0, 1.0), 2.0) * pi * 0.5;
}

fn calc_mie(p: vec2<f32>, lp: vec2<f32>) -> f32 {
	let disk = clamp(1.0 - pow(distance(p, lp), 0.1), 0.0, 1.0);
	return disk*disk*(3.0 - 2.0 * disk) * 2.0 * pi;
}

fn calc_atmospheric_scattering(surface_sizef: vec2<f32>, p: vec2<f32>, lp: vec2<f32>) -> vec3<f32> {
	var sky_color: vec3<f32> = vec3<f32>(0.19, 0.97, 1.4) * (1.0 + anisotropic_intensity); //Make sure one of the conponents is never 0.0

	let corrected_lp = lp; // / max(surface_sizef.x, surface_sizef.y) * surface_sizef.xy;
		
	let zenith = zenith_density(p.y);
	let sun_point_dist_mult =  clamp(length(max(corrected_lp.y + multi_scatter_phase - zenith_offset, 0.0)), 0.0, 1.0);
	
	let rayleigh_mult = calc_rayleigh_mult(p, corrected_lp);
	
	let absorption = calc_sky_absorption(sky_color, zenith);
    let sun_absorption = calc_sky_absorption(sky_color, zenith_density(corrected_lp.y + multi_scatter_phase));
	let sky = sky_color * zenith * rayleigh_mult;
	let sun = calc_sun_point(p, corrected_lp) * absorption;
	let mie = calc_mie(p, corrected_lp) * sun_absorption;
	
	var total_sky = mix(sky * absorption, sky / (sky + 0.5), sun_point_dist_mult);
	total_sky = total_sky + sun + mie;
	total_sky = total_sky * (sun_absorption * 0.5 + 0.5 * length(sun_absorption));
	
	return total_sky;
}

fn jodie_reinhard_tonemap(c: vec3<f32>) -> vec3<f32> {
    let l = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    let tc = c / (c + 1.0);
    return mix(c / (l + 1.0), tc, tc);
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

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = vec2<i32>(1920, 1080); //textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let tcn = in.xy / surface_sizef;
	let world_dir = world_dir_from_depth(tcn);
	let angle_dir = vec2<f32>(atan2(world_dir.z, world_dir.x), acos(-world_dir.y) - pi * 0.5);
	let angle_dir_n = angle_dir / vec2<f32>(pi * 0.2, pi * 0.2);
	let position = angle_dir_n + vec2<f32>(1.0, 0.2);

	//let flipped_in = vec2<f32>(in.x, surface_sizef.y - in.y);
	//let position = flipped_in.xy / max(surface_sizef.x, surface_sizef.y) * 2.0;
	let light_position = vec2<f32>(1.0, 0.3);

	let color = calc_atmospheric_scattering(surface_sizef, position, light_position) * pi;
	let tonemapped = jodie_reinhard_tonemap(color);
	let linear = pow(tonemapped, vec3<f32>(2.2, 2.2, 2.2));
	return vec4<f32>(linear * 0.7, 1.0);
}