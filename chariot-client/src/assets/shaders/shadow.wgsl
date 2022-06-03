struct LightData {
    view_proj: mat4x4<f32>;
};

struct ModelData {
    model: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: LightData;

[[group(1), binding(0)]]
var<uniform> model: ModelData;

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
	return  view.view_proj * model.model * vec4<f32>(position, 1.0);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) {
    return;
}