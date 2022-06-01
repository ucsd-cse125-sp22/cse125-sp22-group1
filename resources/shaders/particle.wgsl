struct ViewData {
	proj_view: mat4x4<f32>;
};

struct ModelData {
    model: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: ViewData;

[[group(1), binding(0)]]
var<uniform> model: ModelData;

struct VertexOutput {
	[[builtin(position)]] position : vec4<f32>;
	[[location(1)]] tex_coords : vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>, [[location(1)]] tex_coords: vec2<f32>) -> VertexOutput {
	var out : VertexOutput;
	out.position = view.proj_view * model.model * vec4<f32>(position, 0.0, 1.0);
	out.tex_coords = tex_coords;
	out.position.z = out.position.z - 0.04; // depth offset hack to make fire appear in front of car
    return out;
}

[[group(2), binding(0)]]
var texture: texture_2d<f32>;

struct FramebufferData {
	[[location(0)]] color: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FramebufferData {//[[location(0)]] vec4<f32> {
	let diffuse_size = textureDimensions(texture);
	let diffuse_sizef = vec2<f32>(diffuse_size);
	let tc = vec2<i32>(in.tex_coords * diffuse_sizef);

	let color = textureLoad(texture, tc, 0);
	let light_dir = vec3<f32>(-0.5, -1.0, 0.5);

	var data: FramebufferData;
	data.color = color;
	data.normal = vec4<f32>(0.0, 0.0, 0.9, color.a);

	return data;
	//return color;
}