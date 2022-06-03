use crate::resources::glyph_cache::{FontSelection, FontSource};
use include_flate::flate;
use lazy_static::lazy_static;

flate!(static PRESS_START_FONT_DATA: [u8] from "src/assets/fonts/PressStart2P-Regular.ttf");

// we have to use lazy static here because we don't actually know the address of PRESS_START in advance
lazy_static! {
    pub static ref PLACEMENT_TEXT_FONT: FontSelection = FontSelection {
        source: FontSource::EmbeddedFont {
            data: &PRESS_START_FONT_DATA
        },
        point_size: 38,
    };
    pub static ref LAP_TEXT_FONT: FontSelection = FontSelection {
        source: FontSource::EmbeddedFont {
            data: &PRESS_START_FONT_DATA
        },
        point_size: 28,
    };
}

pub const PRIMARY_FONT: FontSelection = FontSelection {
    source: FontSource::SystemFont {
        font_name: "ArialMT",
    },
    point_size: 32,
};
