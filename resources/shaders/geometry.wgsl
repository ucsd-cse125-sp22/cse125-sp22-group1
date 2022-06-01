struct ViewData {
	proj_view: mat4x4<f32>;
};

struct ModelData {
    model: mat4x4<f32>;
	normal_to_view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: ViewData;

[[group(1), binding(0)]]
var<uniform> model: ModelData;

fn oct_wrap(v: vec2<f32>) -> vec2<f32>
{
    return (1.0 - abs(v.yx)) * sign(v.xy);
}

// outputs in (-1, 1)
// https://knarkowicz.wordpress.com/2014/04/16/octahedron-normal-vector-encoding/
fn oct_encode(n: vec3<f32>) -> vec2<f32> {
	var nn = n / (abs(n.x) + abs(n.y) + abs(n.z));
	if (nn.z < 0.0) {
		let wrapped = oct_wrap(nn.xy);
		nn.x = wrapped.x;
		nn.y = wrapped.y;
	}

	return nn.xy;
}

struct VertexOutput {
	[[builtin(position)]] position : vec4<f32>;
	[[location(0)]] normal : vec2<f32>;
	[[location(1)]] tex_coords : vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>, [[location(1)]] normal: vec3<f32>, [[location(2)]] tex_coords: vec2<f32>) -> VertexOutput {
	let view_normal = normalize((model.normal_to_view * vec4<f32>(normal, 0.0)).xyz);
	let oct_normal = oct_encode(view_normal);

	var out : VertexOutput;
	out.position = view.proj_view * model.model * vec4<f32>(position, 1.0);
	out.normal = oct_normal * 0.5 + 0.5;
	out.tex_coords = tex_coords;
    return out;
}

struct MaterialInfo {
	id: u32;
};

[[group(2), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(2), binding(1)]]
var s_diffuse: sampler;
[[group(2), binding(2)]]
var<uniform> material: MaterialInfo;

struct FramebufferData {
	[[location(0)]] color: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
};

fn srgb_to_linear(x: f32) -> f32{
	if (x <= 0.0) {
		return 0.0;
	} else if (x >= 1.0) {
		return 1.0;
	} else if (x < 0.04045) {
		return x / 12.92;
	} else {
		return pow((x + 0.055) / 1.055, 2.4);
	}
}

fn srgb_to_linear_color(x: vec4<f32>) -> vec4<f32> {
	return vec4<f32>(
		srgb_to_linear(x.r), srgb_to_linear(x.g), 
		srgb_to_linear(x.b), srgb_to_linear(x.a)
	);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FramebufferData {
    //return vec4<f32>((in.normal.xyz + 1.0) * 0.5, 1.0);
	let tc_transformed = vec2<f32>(in.tex_coords.x, in.tex_coords.y);
	let srgb_color = textureSample(t_diffuse, s_diffuse, tc_transformed);

	var data : FramebufferData;
	data.color = srgb_color;
	data.normal = vec4<f32>(in.normal, 0.0, 1.0);
	return data;
}