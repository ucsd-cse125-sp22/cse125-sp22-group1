[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[group(0), binding(1)]]
var t_blur_color: texture_2d<f32>;

[[group(0), binding(2)]]
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
	let surface_size = textureDimensions(t_color);
	let surface_sizef = vec2<f32>(surface_size);

	let tc = vec2<i32>(in.xy);
	let tcn = vec2<f32>(tc) / surface_sizef;

	let color = textureLoad(t_color, tc, 0);
	let blur_color = textureSample(t_blur_color, s_color, tcn);

	let alpha = blur_color.a;
	let alpha_out = alpha + color.a * (1.0 - alpha);
	//let color_out = vec4<f32>((1.0 - alpha) * color.rgb + alpha * blur_color.rgb, alpha_out);


	let color_out = color.rgb + blur_color.rgb;

	return vec4<f32>(color_out, 1.0);
}