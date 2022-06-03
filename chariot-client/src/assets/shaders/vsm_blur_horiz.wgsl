[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_depth: texture_2d<f32>;

fn gauss(x: f32, s: f32) -> f32 {
	let pi: f32 = 3.14159265359;
	return exp(-(x * x) / (2.0 * s * s)) / sqrt(2.0 * pi * s * s);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec2<f32> {
	let tc = vec2<i32>(in.xy);

	let shadow_size = textureDimensions(t_depth);
	let shadow_sizef = vec2<f32>(shadow_size);
	
	let s: f32 = 6.0;
	let support: i32 = 11;
	let middle = support / 2;

	var weight_sum = 0.0;
	var depth_depth2_sum = vec2<f32>(0.0, 0.0);
	for(var i: i32 = 0; i < support; i = i + 1) {
		let x = i - middle;
		let xf = f32(x);
		let weight = gauss(xf, s);

		let sample_tc = tc + vec2<i32>(x, 0);
		let depth_depth2 = textureLoad(t_depth, sample_tc, 0).rg;

		depth_depth2_sum = depth_depth2_sum + depth_depth2 * weight;
		weight_sum = weight_sum + weight;
	}
    
	let depth_depth2_avg = depth_depth2_sum / weight_sum;

	return depth_depth2_avg;
}