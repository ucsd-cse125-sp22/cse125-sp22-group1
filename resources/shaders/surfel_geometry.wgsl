struct ModelData {
    model: mat4x4<f32>;
	proj_view: mat4x4<f32>;
	normal_to_global: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> mvp: ModelData;

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] normal: vec3<f32>;
	[[location(1)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>, [[location(1)]] normal: vec3<f32>, [[location(2)]] color: vec3<f32>) -> VertexOutput{
	var out: VertexOutput;
	out.position =  mvp.proj_view * mvp.model * vec4<f32>(position, 1.0);
	out.normal = normal;
	out.color = color;
	return out;
}

struct FramebufferData {
	[[location(0)]] color: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FramebufferData {
    var data : FramebufferData;
	data.color = vec4<f32>(in.color, 1.0);
	data.normal = vec4<f32>(in.normal, 0.0) * 0.5 + 0.5;
	return data;
}