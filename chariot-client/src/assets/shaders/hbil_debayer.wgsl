[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_hibl_color: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let surface_size = textureDimensions(t_hibl_color);
	let surface_sizef = vec2<f32>(surface_size);

	let tc = vec2<i32>(in.xy);
	let hibl_tc = tc * vec2<i32>(4, 4);

	// simple 4x4 average
	var total_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(0, 0), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(0, 1), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(0, 2), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(0, 3), 0);

	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(1, 0), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(1, 1), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(1, 2), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(1, 3), 0);

	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(2, 0), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(2, 1), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(2, 2), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(2, 3), 0);

	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(3, 0), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(3, 1), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(3, 2), 0);
	total_color = total_color + textureLoad(t_hibl_color, hibl_tc + vec2<i32>(3, 3), 0);

	let avg_color = total_color / 16.0;

	return avg_color;
}