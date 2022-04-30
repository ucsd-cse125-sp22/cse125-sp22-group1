[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_forward: texture_2d<f32>;

[[group(0), binding(1)]]
var t_normal: texture_2d<f32>;

struct VarianceInfo {
	count: i32;
	mean: vec3<f32>;
	m2: vec3<f32>;
};

fn acc_var(acc: VarianceInfo, new: vec3<f32>) -> VarianceInfo {
	var res: VarianceInfo;
	res.count = acc.count + 1;

	let vcount = vec3<f32>(f32(res.count));
	let delta = new - acc.mean;
	res.mean = acc.mean + (delta / vcount);
	let delta2 = new - res.mean;
	res.m2 = acc.m2 + delta * delta2;

	return res;
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let tc = vec2<i32>(in.xy);

	var acc : VarianceInfo;
	acc.count = 0;
	acc.mean = vec3<f32>(0.0);
	acc.m2 = vec3<f32>(0.0);

	var tmp : vec3<f32>;
	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, -1), 0).xyz;
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0, -1), 0).xyz;
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1, -1), 0).xyz;
	acc = acc_var(acc, tmp);

	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, 0), 0).xyz;
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0,  0), 0).xyz;
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1,  0), 0).xyz;
	acc = acc_var(acc, tmp);
	
	tmp = textureLoad(t_normal, tc + vec2<i32>(-1, 1), 0).xyz;
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(0,  1), 0).xyz;
	acc = acc_var(acc, tmp);
	tmp = textureLoad(t_normal, tc + vec2<i32>(1,  1), 0).xyz;
	acc = acc_var(acc, tmp);

	let variance = length(acc.m2 / vec3<f32>(f32(acc.count)));

	let light_dir = vec3<f32>(1.0, 1.0, 0.0);
	let normal = textureLoad(t_normal, tc + vec2<i32>(0,  0), 0).xyz;
	let color = textureLoad(t_forward, tc, 0);
	let diffuse = max(dot(normal, light_dir), 0.0) * color;
	return vec4<f32>(variance) + diffuse;
}
