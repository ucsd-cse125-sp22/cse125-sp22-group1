use include_flate::flate;

flate!(pub static FORWARD: str from "../resources/shaders/forward.wgsl");
flate!(pub static PARTICLE: str from "../resources/shaders/particle.wgsl");
flate!(pub static POST_PROCESS: str from "../resources/shaders/postprocess.wgsl");
flate!(pub static SHADOW: str from "../resources/shaders/shadow.wgsl");
// flate!(pub static SKYBOX: str from "../resources/shaders/skybox.wgsl");
flate!(pub static UI: str from "../resources/shaders/ui.wgsl");
