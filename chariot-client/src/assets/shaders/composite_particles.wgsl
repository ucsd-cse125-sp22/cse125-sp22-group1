[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[group(0), binding(1)]]
var t_depth: texture_2d<f32>;

[[group(0), binding(2)]]
var t_particles_color: texture_2d<f32>;

[[group(0), binding(3)]]
var t_particles_depth: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let tc = vec2<i32>(in.xy);
	let color = textureLoad(t_color, tc, 0);
	let depth = textureLoad(t_depth, tc, 0).r;

	let particles_color = textureLoad(t_particles_color, tc, 0);
	let particles_depth = textureLoad(t_particles_depth, tc, 0).r;

	var color_out = color;
	if (particles_depth < depth) {
		let alpha = particles_color.a;
		let alpha_out = alpha + color.a * (1.0 - alpha);
		color_out = vec4<f32>((1.0 - alpha) * color.rgb + alpha * particles_color.rgb, alpha_out);
	}

	//color_out = vec4<f32>(color_out.rgb, 0.0);

	return color_out;
}