use glam::Vec2;

use crate::drawable::technique::UILayerTechnique;
use crate::drawable::UIDrawable;
use crate::renderer::Renderer;
use crate::resources::glyph_cache::FontSelection;
use crate::resources::ResourceManager;

#[derive(Copy, Clone)]
pub enum StringAlignment {
    LEFT,
    RIGHT,
    CENTERED,
}

#[derive(Clone)]
pub struct StringBuilder {
    font_selection: FontSelection,
    screen_position: Vec2,
    alignment: StringAlignment,
    content: String,
}

// builder-pattern structure for creating UIDrawables that represent a rendered string
impl StringBuilder {
    pub fn new(font_selection: FontSelection) -> StringBuilder {
        StringBuilder {
            font_selection,
            alignment: StringAlignment::LEFT,
            screen_position: Vec2::new(0.0, 0.0),
            content: String::from(""),
        }
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = String::from(content);
        self
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.screen_position = Vec2::new(x, y);
        self
    }

    pub fn alignment(mut self, alignment: StringAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn build_drawable(
        self,
        renderer: &Renderer,
        resource_manager: &mut ResourceManager,
    ) -> UIDrawable {
        let mut screen_pos = self.screen_position.clone();

        let glyph_cache = resource_manager.get_glyph_cache(self.font_selection);

        // first, grab all glyphs
        let total_string_width: f32 = self
            .content
            .chars()
            .map(|char| {
                glyph_cache
                    .get_glyph(char, renderer)
                    .get_advance_surface_offset(renderer)
                    .x
            })
            .sum();

        // generate starting position depending on alignment settings
        match self.alignment {
            StringAlignment::LEFT => {}
            StringAlignment::RIGHT => screen_pos.x -= total_string_width,
            StringAlignment::CENTERED => screen_pos.x -= total_string_width / 2.0,
        }

        // get UILayerTechniques for each glyph
        let layers: Vec<UILayerTechnique> = self
            .content
            .chars()
            .map(|char| {
                let glyph = glyph_cache.get_glyph(char, renderer);
                let render_position = screen_pos - glyph.get_origin_surface_offset(renderer);

                // increment the screen position by the glyph advance
                screen_pos += glyph.get_advance_surface_offset(renderer);

                UILayerTechnique::new(
                    renderer,
                    render_position,
                    glyph.get_bounds_surface_offset(renderer),
                    glyph.texture_offset,
                    glyph.texture_size,
                    &glyph.texture,
                )
            })
            .collect();

        UIDrawable { layers }
    }
}
