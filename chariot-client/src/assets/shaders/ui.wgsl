
struct VertexOutput {
	[[builtin(position)]] position : vec4<f32>;
	[[location(1)]] tex_coords : vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>, [[location(1)]] tex_coords: vec2<f32>) -> VertexOutput {
	var out : VertexOutput;
	out.position =  vec4<f32>(position, 0.0, 1.0);
	out.tex_coords = tex_coords;
    return out;
}

struct colorHolder {
    color: vec4<f32>;
};

[[group(0), binding(0)]]
var texture: texture_2d<f32>;

[[group(0), binding(1)]]
var<uniform> tint: colorHolder;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	let diffuse_size = textureDimensions(texture);
	let diffuse_sizef = vec2<f32>(diffuse_size);
	let tc = vec2<i32>(in.tex_coords * diffuse_sizef);

	let color = textureLoad(texture, tc, 0) * tint.color;

	return color;
}