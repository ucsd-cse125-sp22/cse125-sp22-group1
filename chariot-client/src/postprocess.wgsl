[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 1.0);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
