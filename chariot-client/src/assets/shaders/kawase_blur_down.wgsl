[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[group(0), binding(1)]]
var s_color: sampler;

// modified from: https://www.froyok.fr/blog/2021-09-ue4-custom-lens-flare/
// initial filter presented by Masaki Kawase at GDC

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_color);
	let surface_sizef = vec2<f32>(surface_size);

	let tc = vec2<f32>(in.xy) * 2.0 / surface_sizef;
	let pixel_size = 1.0 / surface_sizef;
	//let color = textureLoad(t_color, tc, 0);
	
    let half_pixel = (1.0 / surface_sizef) * 0.5;

    let dir_diag1 = vec2<f32>(-half_pixel.x,  half_pixel.y); // Top left
    let dir_diag2 = vec2<f32>(half_pixel.x,  half_pixel.y); // Top right
    let dir_diag3 = vec2<f32>(half_pixel.x, -half_pixel.y); // Bottom right
    let dir_diag4 = vec2<f32>(-half_pixel.x, -half_pixel.y); // Bottom left

    var color = textureSample(t_color, s_color, tc).rgb * 4.0;
	color = color + textureSample(t_color, s_color, tc + dir_diag1).rgb;
	color = color + textureSample(t_color, s_color, tc + dir_diag2).rgb;
	color = color + textureSample(t_color, s_color, tc + dir_diag3).rgb;
	color = color + textureSample(t_color, s_color, tc + dir_diag4).rgb;

    let out_color = color / 8.0;

	return vec4<f32>(out_color, 1.0);
}