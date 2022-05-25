use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::source::SystemSource;
use glam::{UVec2, Vec2};
use pathfinder_geometry::transform2d::Transform2F;
use wgpu::Texture;

use crate::renderer::Renderer;

pub struct Glyph {
    pub texture: Rc<Texture>,
    // 0.0 - 1.0 offset on the texture of the top left corner
    pub texture_offset: Vec2,
    // 0.0 - 1.0 offset of the bottom-right corner of the texture - offset
    pub texture_size: Vec2,
    // the size of the rasterable part of the glyph IN PIXELS
    bounds: Vec2,
    origin: Vec2,
    advance: Vec2,
}

impl Glyph {
    // returns a UILayerTechnique-friendly 0.0 - 1.0 screen offset that represents
    // the horizontal and vertical offset of this glyph
    pub fn get_origin_surface_offset(&self, renderer: &Renderer) -> Vec2 {
        let screen_size = renderer.surface_size();
        self.origin / Vec2::new(screen_size.width as f32, screen_size.height as f32)
    }

    // returns a UILayerTechnique-friendly 0.0 - 1.0 screen offset that represents
    // the horizontal and vertical layout size of this glyph
    pub fn get_advance_surface_offset(&self, renderer: &Renderer) -> Vec2 {
        let screen_size = renderer.surface_size();
        self.advance / Vec2::new(screen_size.width as f32, screen_size.height as f32)
    }

    // returns a UILayerTechnique-friendly 0.0 - 1.0 screen offset that represents
    // the horizontal and vertical size of the underlying glyph texture
    pub fn get_bounds_surface_offset(&self, renderer: &Renderer) -> Vec2 {
        let screen_size = renderer.surface_size();
        self.bounds / Vec2::new(screen_size.width as f32, screen_size.height as f32)
    }
}

pub struct GlyphCache {
    // since glyph cache is doing all the rendering, it needs the font and rendering options
    font: Font,
    point_size: f32,
    hinting_options: HintingOptions,
    cache: HashMap<char, Glyph>,
    current_texture: Option<Rc<Texture>>,
    // the size of the wgpu texture in pixels
    texture_size: UVec2,
    // should always point to an offset that HASN'T BEEN USED YET
    current_offset: UVec2,
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
            current_texture: None,
            // this size should contain the entire ascii table (256 monospace glyphs)
            // even if its not enough, more will be allocated if necessary
            texture_size: UVec2::new(point_size as u32 * 16, point_size as u32 * 16),
            current_offset: UVec2::ZERO,
        }
    }

    // fetches the glyph that corresponds with the given character
    // if the glyph hasn't been rasterized yet, that will happen now
    // i find it annoying that I have to pass the renderer AND resource_manager into this function but here we are
    pub fn get_glyph(&mut self, character: char, renderer: &Renderer) -> &Glyph {
        // why do I gotta check the cache this way?
        // ...long story https://stackoverflow.com/questions/42879098/why-are-borrows-of-struct-members-allowed-in-mut-self-but-not-of-self-to-immut
        if self.cache.contains_key(&character) {
            return self.cache.get(&character).unwrap();
        }

        return self.raster_glyph(character, renderer);
    }

    fn raster_glyph(&mut self, character: char, renderer: &Renderer) -> &Glyph {
        // fetch the glyph_id for this character
        // TODO: rather than CRASH, we should render the "unrecognized character" glyph
        let glyph_id = self
            .font
            .glyph_for_char(character)
            .expect("glyph for character unavailable");

        // fetch the bounds of the glyph raster
        let bounds = self
            .font
            .raster_bounds(
                glyph_id,
                self.point_size,
                Transform2F::default(),
                self.hinting_options,
                RasterizationOptions::SubpixelAa,
            )
            .expect("couldn't determine raster bounds for glyph");
        let glyph_size = UVec2::new(bounds.width() as u32, bounds.height() as u32);

        // check if we're going to overflow the wgpu texture, first horizontally
        if self.current_offset.x + glyph_size.x > self.texture_size.x {
            // if so wrap to new line
            self.current_offset = UVec2::new(0, self.current_offset.y + self.point_size as u32);

            // okay now check if we've overflow vertically, if so, we need to allocate a new texture
            if self.current_offset.y + glyph_size.y > self.texture_size.y {
                self.current_offset = UVec2::ZERO;
                self.current_texture = None;
            }
        }

        // create a new wgpu texture if we need to
        let texture = self.current_texture.clone().unwrap_or_else(|| {
            Rc::new(renderer.create_texture2d(
                "glyph cache texture",
                winit::dpi::PhysicalSize::<u32> {
                    width: self.texture_size.x,
                    height: self.texture_size.y,
                },
                wgpu::TextureFormat::Rgba8Unorm,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            ))
        });

        // get glyph layout information
        let metrics = self.font.metrics();
        let advance = self
            .font
            .advance(glyph_id)
            .expect("couldn't get glyph advance");
        let type_bounds = self
            .font
            .typographic_bounds(glyph_id)
            .expect("couldn't get glyph origin");

        // to convert from grid_coords to pixel coords, we'll need to use...MATH
        // https://freetype.org/freetype2/docs/glyphs/glyphs-2.html
        let origin_pixel = (Vec2::new(
            type_bounds.origin_x(),
            // height gets involved because freetype is in the 1st quadrant while our renderer is in the 4th
            type_bounds.origin_y() + type_bounds.height(),
        ) * self.point_size)
            / metrics.units_per_em as f32;
        let advance_pixel =
            (Vec2::new(advance.x(), advance.y()) * self.point_size) / metrics.units_per_em as f32;

        // form glyph struct
        let glyph = Glyph {
            texture,
            texture_offset: self.current_offset.as_vec2() / self.texture_size.as_vec2(),
            texture_size: glyph_size.as_vec2() / self.texture_size.as_vec2(),
            bounds: Vec2::new(bounds.width() as f32, bounds.height() as f32),
            origin: origin_pixel,
            advance: advance_pixel,
        };

        // render glyph to temp canvas
        let mut canvas = Canvas::new(bounds.size(), Format::Rgba32);
        self.font
            .rasterize_glyph(
                &mut canvas,
                glyph_id,
                self.point_size,
                Transform2F::from_translation(-bounds.origin().to_f32()),
                self.hinting_options,
                RasterizationOptions::SubpixelAa,
            )
            .expect("failed to raster glyph");

        // some platforms (cough Windows) do not give us transparency,
        // so we gotta handle transparency the old fashioned way
        let texture_data: Vec<u8> = canvas
            .pixels
            .iter()
            .enumerate()
            .map(|(i, pixel)| {
                if (i + 1) % 4 == 0 {
                    // take the average value of the three color values
                    ((canvas.pixels[i - 1] as u32
                        + canvas.pixels[i - 2] as u32
                        + canvas.pixels[i - 3] as u32)
                        / 3) as u8
                } else {
                    *pixel
                }
            })
            .collect();

        // copy glyph to wgpu texture
        renderer.write_texture2d(
            &glyph.texture,
            self.current_offset,
            texture_data.as_slice(),
            glyph_size,
            4,
        );

        // compute next texture offset
        self.current_offset =
            UVec2::new(self.current_offset.x + glyph_size.x, self.current_offset.y);

        // add to cache for future retrievals
        self.cache.insert(character, glyph);

        // return glyph
        return self
            .cache
            .get(&character)
            .expect("failed to return new glyph");
    }
}
