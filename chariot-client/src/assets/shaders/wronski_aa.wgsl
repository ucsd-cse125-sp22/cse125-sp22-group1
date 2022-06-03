[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[group(0), binding(1)]]
var s_color: sampler;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let tc = vec2<i32>(in.xy) * vec2<i32>(2, 2); // 2x downsample

	let surface_size = textureDimensions(t_color);
	let surface_sizef = vec2<f32>(surface_size);

	let pixel_size = 1.0 / surface_sizef;
	let uv = vec2<f32>(tc) * pixel_size;

	// maybe not the best choice ik
	// https://twitter.com/BartWronsk/status/1499872955169480708
	// https://www.shadertoy.com/view/fsjBWm

	var col = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    col = col + 0.37487566 * textureSample(t_color, s_color, uv + vec2<f32>(-0.75777156,-0.75777156) * pixel_size);
    col = col + 0.37487566 * textureSample(t_color, s_color, uv + vec2<f32>(0.75777156,-0.75777156) * pixel_size);
    col = col +0.37487566 * textureSample(t_color, s_color, uv + vec2<f32>(0.75777156,0.75777156) * pixel_size);
    col = col + 0.37487566 * textureSample(t_color, s_color, uv + vec2<f32>(-0.75777156,0.75777156) * pixel_size);
    
    col = col - 0.12487566 * textureSample(t_color, s_color, uv + vec2<f32>(-2.90709914,0.0) * pixel_size);
    col = col - 0.12487566 * textureSample(t_color, s_color, uv + vec2<f32>(2.90709914,0.0) * pixel_size);
    col = col - 0.12487566 * textureSample(t_color, s_color, uv + vec2<f32>(0.0,-2.90709914) * pixel_size);
    col = col - 0.12487566 * textureSample(t_color, s_color, uv + vec2<f32>(0.0,2.90709914) * pixel_size);    
    
	return col;
}