struct ViewInfo {
	prev_inv_view: mat4x4<f32>;
	prev_inv_proj: mat4x4<f32>;
	inv_view: mat4x4<f32>;
	inv_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var t_depth: texture_2d<f32>;

[[group(0), binding(1)]]
var t_prev_depth: texture_2d<f32>;

[[group(0), binding(2)]]
var t_prev_probes_color: texture_2d<f32>;

[[group(0), binding(3)]]
var t_prev_probes_depth: texture_2d<f32>;

[[group(0), binding(4)]]
var s_probes: sampler;


[[group(1), binding(0)]]
var<uniform> view: ViewInfo;

fn world_pos_from_depth(tex_coord: vec2<f32>, depth: f32, inv_view: mat4x4<f32>, inv_proj: mat4x4<f32>) -> vec3<f32> {
	let clip_space_pos = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_pos_h = inv_proj * clip_space_pos;
	let view_space_pos = vec4<f32>(view_space_pos_h.xyz / view_space_pos_h.w, 1.0);
	let world_space_pos = inv_view * view_space_pos;
	return world_space_pos.xyz;
}

fn oct_decode(f: vec2<f32>) -> vec3<f32> {
    // https://twitter.com/Stubbesaurus/status/937994790553227264
    var n = vec3<f32>(f.x, f.y, 1.0 - abs(f.x) - abs(f.y));
    let t = clamp(-n.z, 0.0, 1.0);

	if (n.x >= 0.0) {
		n.x = n.x - t;
	} else {
		n.x = n.x + t;
	}

	if (n.y >= 0.0) {
		n.y = n.y - t;
	} else {
		n.y = n.y + t;
	}

    return normalize(n);
}

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
	[[builtin(position)]] position: vec4<f32>;
	[[location(1)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput{
	let surface_size = textureDimensions(t_depth);
	let surface_sizef = vec2<f32>(surface_size);

	let tc = vec2<i32>(i32(vertex_index) % surface_size.x, i32(vertex_index) / surface_size.x);

	let probe_size = 8;
	let probe_center = probe_size / 2;
	let probe_centerf = f32(probe_center);
	let probe_2d_idx = tc / probe_size;
	let probe_tc = probe_2d_idx * probe_size + probe_center;
	let probe_tcn = vec2<f32>(probe_tc) / surface_sizef;
	
	let prev_probe_depth = textureLoad(t_prev_depth, probe_tc, 0).r;
	let prev_probe_world_pos = world_pos_from_depth(probe_tcn, prev_probe_depth, view.prev_inv_view, view.prev_inv_proj);
	let cur_probe_depth = textureLoad(t_depth, probe_tc, 0).r;
	let cur_probe_world_pos = world_pos_from_depth(probe_tcn, cur_probe_depth, view.inv_view, view.inv_proj);

	let max_depth = 100.0;
	let prev_oct_coords = tc - (probe_2d_idx * probe_size);
	let prev_oct_coords_n = vec2<f32>(prev_oct_coords) / probe_centerf;
	let prev_sample_dir = oct_decode(prev_oct_coords_n);
	let prev_sample_depth = textureLoad(t_prev_probes_depth, tc, 0).r * max_depth;
	let prev_sample_color = textureLoad(t_prev_probes_color, tc, 0).rgb;
	let sample_world_pos = prev_probe_world_pos + prev_sample_dir * prev_sample_depth;

	let cur_sample_dir = normalize(sample_world_pos - cur_probe_world_pos);
	let cur_sample_depth = length(sample_world_pos - cur_probe_world_pos);
	let cur_oct_coords_n = oct_encode(cur_sample_dir);
	let cur_oct_coords = vec2<i32>(cur_oct_coords_n * probe_centerf);
	let reproject_tc = probe_tc + cur_oct_coords;
	let reproject_tcn = vec2<f32>(reproject_tc) / surface_sizef;
	let reproject_ndc = vec2<f32>(reproject_tcn.x, 1.0 - reproject_tcn.y) * 2.0 - 1.0;

	let depth_out = min(cur_sample_depth / max_depth, 1.0);

	var out: VertexOutput;
	out.position = vec4<f32>(reproject_ndc, depth_out, 1.0);
	out.color = prev_sample_color;
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