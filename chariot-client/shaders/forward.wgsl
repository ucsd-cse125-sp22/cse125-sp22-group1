struct ModelData {
    model: mat4x4<f32>;
	proj_view: mat4x4<f32>;
	normal_to_local: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> mvp: ModelData;

struct VertexOutput {
	[[builtin(position)]] position : vec4<f32>;
	[[location(0)]] normal : vec3<f32>;
	[[location(1)]] tex_coords : vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>, [[location(1)]] normal: vec3<f32>, [[location(2)]] tex_coords: vec2<f32>) -> VertexOutput {
	var out : VertexOutput;
	out.position =  mvp.proj_view * mvp.model * vec4<f32>(position, 1.0);
	out.normal = normal; //normalize((mvp.normal_to_local * vec4<f32>(normal, 0.0)).xyz);
	out.tex_coords = tex_coords;
    return out;
}

struct MaterialInfo {
	id: u32;
};

[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var s_diffuse: sampler;
[[group(1), binding(2)]]
var<uniform> material: MaterialInfo;

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
	data.normal = vec4<f32>(in.normal, f32(material.id) * 5.0);
	return data;
}