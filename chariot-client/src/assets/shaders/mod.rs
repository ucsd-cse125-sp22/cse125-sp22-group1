use include_flate::flate;

flate!(pub static FORWARD: str from "src/assets/shaders/forward.wgsl");
flate!(pub static PARTICLE: str from "src/assets/shaders/particle.wgsl");
flate!(pub static POST_PROCESS: str from "src/assets/shaders/postprocess.wgsl");
flate!(pub static SHADOW: str from "src/assets/shaders/shadow.wgsl");
// flate!(pub static SKYBOX: str from "src/assets/shaders/skybox.wgsl");
flate!(pub static UI: str from "src/assets/shaders/ui.wgsl");
