struct Locals {
    scale: f32;
};

[[group(0), binding(0)]]
var<uniform> local: Locals;

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>,) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position * local.scale, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}