struct MVPData {
    mvp: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> mvp: MVPData;

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
	return  mvp.mvp * vec4<f32>(position, 1.0);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) {
    return;
}