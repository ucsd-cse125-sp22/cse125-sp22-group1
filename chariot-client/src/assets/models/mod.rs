use chariot_core::player::choices::Chair;
use include_flate::flate;

flate!(pub static BEANBAG: [u8] from "src/assets/models/beanbag.glb");
flate!(pub static ERGONOMIC: [u8] from "src/assets/models/ergonomic.glb");
flate!(pub static FOLDING_CHAIR: [u8] from "src/assets/models/foldingchair.glb");
// flate!(pub static POWERUP: [u8] from "src/assets/models/powerup.glb");
flate!(pub static RECLINER: [u8] from "src/assets/models/recliner.glb");
flate!(pub static SWIVEL: [u8] from "src/assets/models/swivel.glb");
// flate!(pub static WET_FLOOR_SIGN: [u8] from "src/assets/models/wetfloorsign.glb");

pub fn get_chair_data(chair: Chair) -> &'static [u8] {
    match chair {
        Chair::Swivel => &SWIVEL,
        Chair::Recliner => &RECLINER,
        Chair::Ergonomic => &ERGONOMIC,
        Chair::Beanbag => &BEANBAG,
        Chair::Folding => &FOLDING_CHAIR,
    }
}
