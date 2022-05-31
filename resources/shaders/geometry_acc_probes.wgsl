struct ViewInfo {
	inv_view: mat4x4<f32>;
	inv_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var t_depth: texture_2d<f32>;

[[group(0), binding(1)]]
var t_color: texture_2d<f32>;

[[group(1), binding(0)]]
var<uniform> view: ViewInfo;

fn world_pos_from_depth(tex_coord: vec2<f32>, depth: f32, inv_view: mat4x4<f32>, inv_proj: mat4x4<f32>) -> vec3<f32> {
	let clip_space_pos = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_pos_h = inv_proj * clip_space_pos;
	let view_space_pos = vec4<f32>(view_space_pos_h.xyz / view_space_pos_h.w, 1.0);
	let world_space_pos = inv_view * view_space_pos;
	return world_space_pos.xyz;
}

fn oct_wrap(v: vec2<f32>) -> vec2<f32>
{
	var res: vec2<f32> = (1.0 - abs(v.yx));

	if (v.x < 0.0) {
		res.x = res.x * -1.0;
	}

	if (v.y < 0.0) {
		res.y = res.y * -1.0;
	}

	return res;
    //return (1.0 - abs(v.yx)) * sign(v.xy);
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
	[[builtin(position)]] position: vec4<f32>;
	[[location(1)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput{
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let tc = vec2<i32>(i32(vertex_index) % surface_size.x, i32(vertex_index) / surface_size.x);
	let tcn = vec2<f32>(tc) / surface_sizef;

	let probe_size = 16;
	let probe_center = probe_size / 2;
	let probe_centerf = f32(probe_center);
	let probe_2d_idx = tc / probe_size;
	let probe_tc = probe_2d_idx * probe_size + probe_center;
	let probe_tcn = vec2<f32>(probe_tc) / surface_sizef;

	let cur_probe_depth = textureLoad(t_depth, probe_tc, 0).r;
	let cur_probe_world_pos = world_pos_from_depth(probe_tcn, cur_probe_depth, view.inv_view, view.inv_proj);

	let sample_color = textureLoad(t_color, tc, 0).rgb;
	let sample_depth = textureLoad(t_depth, tc, 0).r;
	let sample_world_pos = world_pos_from_depth(tcn, sample_depth, view.inv_view, view.inv_proj);

	let new_sample_dir = normalize(sample_world_pos - cur_probe_world_pos);
	let new_sample_depth = length(sample_world_pos - cur_probe_world_pos);
	let new_oct_coords_n = oct_encode(new_sample_dir.xzy) * 1.01; //1.00001; magic number here to counteract rounding error
	let new_oct_coords = vec2<i32>(new_oct_coords_n * probe_centerf);
	let reproject_tc = probe_tc + new_oct_coords;

	let max_depth = 100.0;
	let depth_out = min(new_sample_depth / max_depth, 1.0);

	let tc_out = reproject_tc + vec2<i32>(1,1);
	let tcn_out = vec2<f32>(tc_out) / surface_sizef;
	let ndc_out = vec2<f32>(tcn_out.x, 1.0 - tcn_out.y) * 2.0 - 1.0;
	
	var out: VertexOutput;
	out.position = vec4<f32>(ndc_out, depth_out, 1.0);
	out.color = sample_color;
	return out;
}

struct FramebufferData {
	[[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FramebufferData {
	var out: FramebufferData;
	out.color = vec4<f32>(in.color, 1.0);
	return out;
}