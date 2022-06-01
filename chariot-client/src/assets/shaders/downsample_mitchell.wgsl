[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32>{
	return vec4<f32>(position, 0.0, 1.0);
}

[[group(0), binding(0)]]
var t_color: texture_2d<f32>;

fn mitchell_filter(offset: vec2<i32>) -> f32
{
	let offsetf = vec2<f32>(offset);
	let x = length(offsetf);
    let ax = abs(x);

    if (ax < 1.0) {
        return (16.0 + ax*ax*(21.0 * ax - 36.0))/18.0;
	} else if (ax < 2.0) {
        return (32.0 + ax*(-60.0 + ax*(36.0 - 7.0*ax)))/18.0;
	}

    return 0.0;
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
	let tc = vec2<i32>(in.xy) * vec2<i32>(2, 2); // 2x downsample

	let dir_axis1 = vec2<i32>(-1, 0);
	let dir_axis2 = vec2<i32>(0, 1);
	let dir_axis3 = vec2<i32>(1, 0);
	let dir_axis4 = vec2<i32>(0, -1);

	let dir_diag1 = vec2<i32>(-1, 1);
	let dir_diag2 = vec2<i32>(1, 1);
	let dir_diag3 = vec2<i32>(1, -1);
	let dir_diag4 = vec2<i32>(-1, -1);

	let dir_axis5 = vec2<i32>(-2, 0);
	let dir_axis6 = vec2<i32>(0, 2);
	let dir_axis7 = vec2<i32>(2, 0);
	let dir_axis8 = vec2<i32>(0, -2);

	let coef_center = mitchell_filter(vec2<i32>(0, 0));

	let coef_axis1 = mitchell_filter(dir_axis1);
	let coef_axis2 = mitchell_filter(dir_axis2);
	let coef_axis3 = mitchell_filter(dir_axis3);
	let coef_axis4 = mitchell_filter(dir_axis4);

	let coef_diag1 = mitchell_filter(dir_diag1);
	let coef_diag2 = mitchell_filter(dir_diag2);
	let coef_diag3 = mitchell_filter(dir_diag3);
	let coef_diag4 = mitchell_filter(dir_diag4);

	let coef_axis5 = mitchell_filter(dir_axis5);
	let coef_axis6 = mitchell_filter(dir_axis6);
	let coef_axis7 = mitchell_filter(dir_axis7);
	let coef_axis8 = mitchell_filter(dir_axis8);

	let coef_total = coef_center + coef_axis1 + coef_axis2 + coef_axis3 + coef_axis4 + 
		coef_diag1 + coef_diag2 + coef_diag3 + coef_diag4 + 
		coef_axis5 + coef_axis6 + coef_axis7 + coef_axis8;
	
	let filter_scale = 1.0 / coef_total;

	// 0, -0.032, 0, 0.284, 0.496, 0.284, 0, -0.032, 0
	var color = textureLoad(t_color, tc + vec2<i32>(0, 0), 0) * coef_center;

	color = color + textureLoad(t_color, tc + dir_axis1, 0) * coef_axis1;
	color = color + textureLoad(t_color, tc + dir_axis2, 0) * coef_axis2;
	color = color + textureLoad(t_color, tc + dir_axis3, 0) * coef_axis3;
	color = color + textureLoad(t_color, tc + dir_axis4, 0) * coef_axis4;

	color = color + textureLoad(t_color, tc + dir_diag1, 0) * coef_diag1;
	color = color + textureLoad(t_color, tc + dir_diag2, 0) * coef_axis2;
	color = color + textureLoad(t_color, tc + dir_diag3, 0) * coef_axis3;
	color = color + textureLoad(t_color, tc + dir_diag4, 0) * coef_axis4;

	color = color + textureLoad(t_color, tc + dir_axis5, 0) * coef_axis5;
	color = color + textureLoad(t_color, tc + dir_axis6, 0) * coef_axis6;
	color = color + textureLoad(t_color, tc + dir_axis7, 0) * coef_axis7;
	color = color + textureLoad(t_color, tc + dir_axis8, 0) * coef_axis8;

	return color * filter_scale;
}