[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[group(0), binding(1)]]
var s_color: sampler;

// custom 13 tap downsample from CoD: AW: http://advances.realtimerendering.com/s2014/
// https://www.froyok.fr/blog/2021-09-ue4-custom-lens-flare/resources/cod/cod_13taps_filter.png
// from: https://www.froyok.fr/blog/2021-09-ue4-custom-lens-flare/

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_color);
	let surface_sizef = vec2<f32>(surface_size);

	let tc = vec2<f32>(in.xy) * 2.0 / surface_sizef;
	let pixel_size = 1.0 / surface_sizef;
	//let color = textureLoad(t_color, tc, 0);
	
	var out_color: vec3<f32>;
    var color = vec3<f32>(0.0, 0.0, 0.0);

    // 4 central samples
    color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(-1.0, 1.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(1.0, 1.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(-1.0, -1.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(1.0, -1.0)).rgb;

    out_color = (color / 4.0) * 0.5;

    // 3 row samples
    color = vec3<f32>(0.0, 0.0, 0.0);

	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(-2.0, 2.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(0.0, 2.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(2.0, 2.0)).rgb;

	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(-2.0, 0.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(0.0, 0.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(2.0, 0.0)).rgb;

	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(-2.0, -2.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(0.0, 2.0)).rgb;
	color = color + textureSample(t_color, s_color, tc + pixel_size * vec2<f32>(2.0, -2.0)).rgb;
    
    out_color = out_color + (color / 9.0) * 0.5;

    // Threshold
	let threshold_level = 2.1;
	let threshold_range = 1.0;

    let luminance = dot(out_color, vec3<f32>(1.0, 1.0, 1.0));
    let threshold_scale = clamp((luminance - threshold_level) / threshold_range, 0.0, 1.0);

    out_color = out_color * threshold_scale;

	return vec4<f32>(out_color, 1.0);
}