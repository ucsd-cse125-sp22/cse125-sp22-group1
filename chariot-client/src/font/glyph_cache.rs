use std::collections::HashMap;

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::source::SystemSource;
use glam::UVec2;
use pathfinder_geometry::transform2d::Transform2F;

use crate::renderer::Renderer;
use crate::resources::{ResourceManager, TextureHandle};

struct Glyph {
    texture_handle: TextureHandle,
    offset: UVec2,
    size: UVec2,
}

struct GlyphCache<const POINT_SIZE: u32> {
    // since glyph cache is doing all the rendering, it needs the font and rendering options
    font: Font,
    hinting_options: HintingOptions,
    cache: HashMap<char, Glyph>,
    current_texture: Option<TextureHandle>,
    // the size of the wgpu texture in pixels
    texture_size: UVec2,
    // should always point to an offset that HASN'T BEEN USED YET
    current_offset: UVec2,
}

impl<const POINT_SIZE: u32> GlyphCache<POINT_SIZE> {
    // creates a new GlyphCache for the named font
    pub fn new(font_name: &str) -> GlyphCache<POINT_SIZE> {
        let font = SystemSource::new()
            .select_by_postscript_name(font_name)
            .expect("could not find requested font on the system")
            .load()
            .expect("could not load font despite finding it");
        GlyphCache {
            font,
            hinting_options: HintingOptions::Full(POINT_SIZE as f32),
            cache: HashMap::new(),
            current_texture: None,
            // this size should contain the entire ascii table (256 monospace glyphs)
            // even if its not enough, more will be allocated if necessary
            texture_size: UVec2::new(POINT_SIZE * 16, POINT_SIZE * 16),
            current_offset: UVec2::ZERO,
        }
    }

    // fetches the glyph that corresponds with the given character
    // if the glyph hasn't been rasterized yet, that will happen now
    // i find it annoying that I have to pass the renderer AND resource_manager into this function but here we are
    pub fn get_glyph(
        &mut self,
        character: char,
        renderer: &Renderer,
        resource_manager: &mut ResourceManager,
    ) -> &Glyph {
        // why do I gotta check the cache this way?
        // ...long story https://stackoverflow.com/questions/42879098/why-are-borrows-of-struct-members-allowed-in-mut-self-but-not-of-self-to-immut
        if self.cache.contains_key(&character) {
            return self.cache.get(&character).unwrap();
        }

        return self.raster_glyph(character, renderer, resource_manager);
    }

    fn raster_glyph(
        &mut self,
        character: char,
        renderer: &Renderer,
        resource_manager: &mut ResourceManager,
    ) -> &Glyph {
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
                POINT_SIZE as f32,
                Transform2F::default(),
                self.hinting_options,
                RasterizationOptions::SubpixelAa,
            )
            .expect("couldn't determine raster bounds for glyph");
        let glyph_size = UVec2::new(bounds.width() as u32, bounds.height() as u32);

        // render glyph to temp canvas
        let mut canvas = Canvas::new(bounds.size(), Format::Rgba32);
        self.font
            .rasterize_glyph(
                &mut canvas,
                glyph_id,
                POINT_SIZE as f32,
                Transform2F::from_translation(-bounds.origin().to_f32()),
                self.hinting_options,
                RasterizationOptions::SubpixelAa,
            )
            .expect("failed to raster glyph");

        // check if we're going to overflow the wgpu texture, first horizontally
        if self.current_offset.x + glyph_size.x > self.texture_size.x {
            // if so wrap to new line
            self.current_offset = UVec2::new(0, self.current_offset.y + POINT_SIZE);

            // okay now check if we've overflow vertically, if so, we need to allocate a new texture
            if self.current_offset.y + glyph_size.y > self.texture_size.y {
                self.current_offset = UVec2::ZERO;
                self.current_texture = None;
            }
        }

        // create a new wgpu texture if we need to
        let texture_handle = self.current_texture.unwrap_or_else(|| {
            resource_manager.register_texture(renderer.create_texture2d(
                "glyph cache texture",
                winit::dpi::PhysicalSize::<u32> {
                    width: self.texture_size.x,
                    height: self.texture_size.y,
                },
                wgpu::TextureFormat::Rgba8Uint,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            ))
        });

        // form glyph struct
        let glyph = Glyph {
            texture_handle,
            offset: self.current_offset,
            size: glyph_size,
        };

        // copy glyph to wgpu texture
        let texture = resource_manager
            .textures
            .get(&texture_handle)
            .expect("couldn't get texture we just created for glyph cache");
        renderer.write_texture2d(
            texture,
            self.current_offset,
            canvas.pixels.as_slice(),
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
