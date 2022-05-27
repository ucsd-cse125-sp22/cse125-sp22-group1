use crate::resources::glyph_cache::{FontSelection, FontSource};
use include_flate::flate;
use lazy_static::lazy_static;

flate!(static PRESS_START: [u8] from "src/assets/fonts/PressStart2P-Regular.ttf");

// we have to use lazy static here because we don't actually know the address of PRESS_START in advance
lazy_static! {
    pub static ref PLACEMENT_FONT_SELECTION: FontSelection = FontSelection {
        source: FontSource::EmbeddedFont { data: &PRESS_START },
        point_size: 38,
    };
}

pub const PRIMARY_FONT: FontSelection = FontSelection {
    source: FontSource::SystemFont {
        font_name: "ArialMT",
    },
    point_size: 32,
};
