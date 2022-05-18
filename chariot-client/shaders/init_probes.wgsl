struct ModelData {
    model: mat4x4<f32>;
	normal_to_global: mat4x4<f32>;
	inv_view: mat4x4<f32>;
	inv_proj: mat4x4<f32>;
};

struct LightInfo {
	view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

[[group(0), binding(1)]]
var t_normal: texture_2d<f32>;

[[group(0), binding(2)]]
var t_depth: texture_2d<f32>;

[[group(0), binding(3)]]
var t_shadow: texture_2d<f32>;

[[group(1), binding(0)]]
var<uniform> mvp: ModelData;

[[group(2), binding(0)]]
var<uniform> light: LightInfo;

fn pcg_hash(input: u32) -> u32 {
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn world_pos_from_depth(tex_coord: vec2<f32>, depth: f32) -> vec3<f32> {
	let clip_space_pos = vec4<f32>(tex_coord.x * 2.0 - 1.0, (1.0 - tex_coord.y) * 2.0 - 1.0, depth, 1.0);
	let view_space_pos_h = mvp.inv_proj * clip_space_pos;
	let view_space_pos = vec4<f32>(view_space_pos_h.xyz / view_space_pos_h.w, 1.0);
	let world_space_pos = mvp.inv_view * view_space_pos;
	return world_space_pos.xyz;
}

fn world_pos_to_light_pos(world_pos: vec3<f32>) -> vec3<f32> {
	let light_pos_h = light.view_proj * vec4<f32>(world_pos, 1.0);
	let light_pos_ndc = light_pos_h.xyz / light_pos_h.w;
	let light_pos = vec3<f32>(light_pos_ndc.x * 0.5 + 0.5, 1.0 - (light_pos_ndc.y * 0.5 + 0.5), light_pos_ndc.z);
	return light_pos;
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
fn vs_main([[builtin(vertex_index)]] vertex_index: u32, [[location(0)]] position: vec3<f32>, [[location(1)]] normal: vec3<f32>, [[location(2)]] color: vec3<f32>) -> VertexOutput{
	let world_pos = (mvp.model * vec4<f32>(position, 1.0)).xyz;
	let world_normal = normalize(mvp.normal_to_global * vec4<f32>(normal, 0.0)).xyz;
	let world_pos_i32 = bitcast<vec3<i32>>(world_pos);
	// TODO: improve, change every frame, etc
	let world_pos_hash_input = world_pos_i32.x * 73856093 + world_pos_i32.y * 19349669 + world_pos_i32.z * 83492791 + i32(vertex_index) * 521;
	let world_pos_hash = pcg_hash(u32(world_pos_hash_input));

	let shadow_size = textureDimensions(t_shadow);
	let shadow_sizef = vec2<f32>(shadow_size);
	let light_pos = world_pos_to_light_pos(world_pos);
	var light_depth: f32 = 100.0;
	if (light_pos.x > 0.0 && light_pos.x < 1.0 && light_pos.y > 0.0 && light_pos.y < 1.0) {
		light_depth = textureLoad(t_shadow, vec2<i32>(light_pos.xy * shadow_sizef), 0).r;
	}

	var shadow = vec3<f32>(1.0); 
	if (light_pos.z > light_depth - 0.00004) {
		shadow = vec3<f32>(0.0); 
	}

	let probe_size = 32;
	let probe_center = probe_size / 2;
	let probe_centerf = f32(probe_center);
	let color_size = textureDimensions(t_color);
	let color_sizef = vec2<f32>(color_size);
	let probes_grid_size = color_size / vec2<i32>(probe_size, probe_size);

	let probe_index = i32(world_pos_hash % u32(probes_grid_size.x * probes_grid_size.y));
	let probe_2d_index = vec2<i32>(probe_index % probes_grid_size.x, probe_index / probes_grid_size.x);
	let tc = probe_2d_index * vec2<i32>(probe_size, probe_size) + vec2<i32>(probe_center, probe_center);
	let tcn = vec2<f32>(tc) / color_sizef;

	let depth = textureLoad(t_depth, tc, 0).r;
	let probe_pos = world_pos_from_depth(tcn, depth);
	let probe_normal = textureLoad(t_normal, tc, 0).xyz * 2.0 - 1.0;
	//let probe_frame = calc_ONB(probe_normal);

	let dist = length(position - probe_pos);
	let probe_to_surfel = normalize(position - probe_pos);
	let oct_coords_n = oct_encode(probe_to_surfel.xzy);
	let oct_coords = vec2<i32>(oct_coords_n * vec2<f32>(probe_centerf, probe_centerf));
	let screen_coords = vec2<f32>(tc + oct_coords) / color_sizef;
	let ndc_coords = vec2<f32>(screen_coords.x, 1.0 - screen_coords.y) * 2.0 - 1.0;
	
	let light_dir = vec3<f32>(-0.5, -1.0, 0.5);
	let diffuse = max(dot(world_normal, -light_dir), 0.0);
	let ambient = vec3<f32>(0.06);
	let shade = color * ((shadow * diffuse)); 
	// div by dist is a hack: compromise between no div (equal area weighting) and dist^2 (area differential weighted);

	let max_depth = 100.0;
	let depth_out = min(dist / max_depth, 1.0);

	var out: VertexOutput;
	out.position = vec4<f32>(ndc_coords, depth_out, 1.0);
	out.color = shade;
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