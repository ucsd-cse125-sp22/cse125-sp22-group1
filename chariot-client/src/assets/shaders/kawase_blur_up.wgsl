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

	let tc = vec2<f32>(in.xy) / (surface_sizef * 2.0);
	let pixel_size = 1.0 / surface_sizef;
	//let color = textureLoad(t_color, tc, 0);
	
    let half_pixel = (1.0 / surface_sizef) * 0.5;

    let dir_diag1 = vec2<f32>(-half_pixel.x,  half_pixel.y); // Top left
    let dir_diag2 = vec2<f32>(half_pixel.x,  half_pixel.y); // Top right
    let dir_diag3 = vec2<f32>(half_pixel.x, -half_pixel.y); // Bottom right
    let dir_diag4 = vec2<f32>(-half_pixel.x, -half_pixel.y); // Bottom left

    let dir_axis1 = vec2<f32>(-half_pixel.x, 0.0);        // Left
    let dir_axis2 = vec2<f32>(half_pixel.x, 0.0);        // Right
    let dir_axis3 = vec2<f32>(0.0,  half_pixel.y);         // Top
    let dir_axis4 = vec2<f32>(0.0, -half_pixel.y);         // Bottom

    var color = vec3<f32>(0.0, 0.0, 0.0);
    color = color + textureSample(t_color, s_color, tc + dir_diag1 ).rgb;
	color = color + textureSample(t_color, s_color, tc + dir_diag2 ).rgb;
	color = color + textureSample(t_color, s_color, tc + dir_diag3 ).rgb;
	color = color + textureSample(t_color, s_color, tc + dir_diag4 ).rgb;
    
	color = color + textureSample(t_color, s_color, tc + dir_axis1 ).rgb * 2.0;
	color = color + textureSample(t_color, s_color, tc + dir_axis2 ).rgb * 2.0;
	color = color + textureSample(t_color, s_color, tc + dir_axis3 ).rgb * 2.0;
	color = color + textureSample(t_color, s_color, tc + dir_axis4 ).rgb * 2.0;
    
    let out_color = color / 12.0;
    
	return vec4<f32>(out_color, 1.0);
}