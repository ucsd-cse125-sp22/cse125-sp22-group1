struct Locals {
    model: mat4x4<f32>;
	view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> local: Locals;

struct VertexOutput {
	[[builtin(position)]] position : vec4<f32>;
	[[location(0)]] normal : vec3<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>, [[location(1)]] normal: vec3<f32>,) -> VertexOutput {
	var out : VertexOutput;
	out.position = local.view_proj * local.model * vec4<f32>(position, 1.0);
	out.normal = normal;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>((in.normal.xyz + 1.0) * 0.5, 1.0);
}