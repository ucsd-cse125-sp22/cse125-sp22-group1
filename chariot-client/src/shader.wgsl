struct Locals {
    model: mat4x4<f32>;
	proj_view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> local: Locals;

struct VertexOutput {
	[[builtin(position)]] position : vec4<f32>;
	[[location(0)]] normal : vec3<f32>;
	[[location(1)]] tex_coords : vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>, [[location(1)]] normal: vec3<f32>, [[location(2)]] tex_coords: vec2<f32>) -> VertexOutput {
	var out : VertexOutput;
	out.position =  local.proj_view * local.model * vec4<f32>(position, 1.0);
	out.normal = normal;
	out.tex_coords = tex_coords;
    return out;
}

[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var s_diffuse: sampler;

struct FramebufferData {
	[[location(0)]] color: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FramebufferData {
    //return vec4<f32>((in.normal.xyz + 1.0) * 0.5, 1.0);
	let tc_transformed = vec2<f32>(in.tex_coords.x, in.tex_coords.y);

	var data : FramebufferData;
	data.color = textureSample(t_diffuse, s_diffuse, tc_transformed);
	data.normal = vec4<f32>(in.normal, 1.0);
	return data;
}