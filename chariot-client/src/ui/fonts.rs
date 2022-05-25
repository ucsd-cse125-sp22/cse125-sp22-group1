use crate::resources::glyph_cache::{FontSelection, FontSource};

pub const PRIMARY_FONT: FontSelection = FontSelection {
    source: FontSource::SystemFont {
        font_name: "ArialMT",
    },
    point_size: 32,
};

pub const PLACEMENT_FONT: FontSelection = FontSelection {
    source: FontSource::FileFont {
        file_path: "PressStart2P-Regular.ttf",
    },
    point_size: 38,
};
