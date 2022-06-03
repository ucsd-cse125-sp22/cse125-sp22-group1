struct LightData {
    view: mat4x4<f32>;
	proj: mat4x4<f32>;
};

struct ModelData {
    model: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: LightData;

[[group(1), binding(0)]]
var<uniform> model: ModelData;

struct VertexOutput {
	[[builtin(position)]] pos: vec4<f32>;
	[[location(0)]] depth: f32;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>) -> VertexOutput {
	let local_pos = view.view * model.model * vec4<f32>(position, 1.0);

	var out: VertexOutput;
	out.pos = view.proj * local_pos;
	out.depth = -local_pos.z;
	return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec2<f32> {
    return vec2<f32>(in.depth, in.depth * in.depth);
}