use std::collections::HashMap;

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::source::SystemSource;
use glam::Vec2;
use pathfinder_geometry::transform2d::Transform2F;

struct Glyph {
    texture: wgpu::Texture,
    offset: Vec2,
    size: Vec2
}

struct GlyphCache {
    // since glyph cache is doing all the rendering, it needs the font and rendering options
    font: Font,
    point_size: f32,
    hinting_options: HintingOptions,
    cache: HashMap<char, Glyph>,
}

impl GlyphCache {
    // creates a new GlyphCache for the named font
    pub fn new(font_name: &str, point_size: f32) -> GlyphCache {
        let font = SystemSource::new()
            .select_by_postscript_name(font_name)
            .expect("could not find requested font on the system")
            .load()
            .expect("could not load font despite finding it");
        GlyphCache {
            font,
            point_size,
            hinting_options: HintingOptions::Full(point_size),
            cache: HashMap::new(),
        }
    }

    // fetches the glyph that corresponds with the given character
    // if the glyph hasn't been rasterized yet, that will happen now
    // the fact this works without lifetime nonsense is astonishing
    pub fn get_glyph(&self, character: char) -> &Glyph {
        self.cache.get(&character).unwrap_or_else(|| { self.raster_glyph(character) })
    }

    fn raster_glyph(&self, character: char) -> &Glyph {

        // fetch the glyph_id for this character
        // rather than CRASH, we should render the "unrecognized character" glyph
        let glyph_id = self.font.glyph_for_char(character)
            .expect("glyph for character unavailable");

        // fetch the bounds of the glyph raster
        let bounds = self.font.raster_bounds(
            glyph_id,
            self.point_size,
            Transform2F::default(),
            self.hinting_options,
            RasterizationOptions::GrayscaleAa,
        ).expect("couldn't determine raster bounds for glyph");

        // render glyph to temp canvas
        let mut canvas = Canvas::new(bounds.size(), Format::A8);
        self.font.rasterize_glyph(
            &mut canvas,
            glyph_id,
            self.point_size,
            Transform2F::from_translation(-bounds.origin().to_f32()),
            self.hinting_options,
            RasterizationOptions::GrayscaleAa,
        );

        // copy glyph to wgpu texture

        // form glyph struct

        // add to cache for future retrievals
    }
}