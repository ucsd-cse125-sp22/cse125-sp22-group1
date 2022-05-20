[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let tc = vec2<i32>(in.xy);
	let color = textureLoad(t_color, tc, 0).rgb;
	return vec4<f32>(color, 1.0);
}