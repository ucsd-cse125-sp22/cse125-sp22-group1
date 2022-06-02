use include_flate::flate;

flate!(pub static GEOMETRY: str from "src/assets/shaders/geometry.wgsl");
flate!(pub static PARTICLE: str from "src/assets/shaders/particle.wgsl");
flate!(pub static SHADE_DIRECT: str from "src/assets/shaders/shade_direct.wgsl");
flate!(pub static SHADOW: str from "src/assets/shaders/shadow.wgsl");
flate!(pub static SKYBOX: str from "src/assets/shaders/skybox.wgsl");
flate!(pub static UI: str from "src/assets/shaders/ui.wgsl");

// bloom stuff
flate!(pub static DOWNSAMPLE_BLOOM: str from "src/assets/shaders/downsample_bloom.wgsl");
flate!(pub static KAWASE_BLUR_DOWN: str from "src/assets/shaders/kawase_blur_down.wgsl");
flate!(pub static KAWASE_BLUR_UP: str from "src/assets/shaders/kawase_blur_up.wgsl");
flate!(pub static COMPOSITE_BLOOM: str from "src/assets/shaders/composite_bloom.wgsl");

// hbil stuff (more simillar to hbao for now)
flate!(pub static HBIL: str from "src/assets/shaders/hbil.wgsl");
flate!(pub static HBIL_DEBAYER: str from "src/assets/shaders/hbil_debayer.wgsl");

// probes (unused for now)
flate!(pub static INIT_PROBES: str from "src/assets/shaders/init_probes.wgsl");
flate!(pub static GEOMETRY_ACC_PROBES: str from "src/assets/shaders/geometry_acc_probes.wgsl");
flate!(pub static TEMPORAL_ACC_PROBES: str from "src/assets/shaders/geometry_acc_probes.wgsl");

// util
flate!(pub static DOWNSAMPLE_MITCHELL: str from "src/assets/shaders/downsample_mitchell.wgsl");
flate!(pub static SIMPLE_FSQ: str from "src/assets/shaders/simple_fsq.wgsl");
flate!(pub static SURFEL_GEOMETRY: str from "src/assets/shaders/surfel_geometry.wgsl");
