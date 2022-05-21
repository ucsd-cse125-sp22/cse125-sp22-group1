use crate::drawable::technique::UILayerTechnique;
use crate::drawable::{Drawable, UIDrawable};
use crate::renderer::{render_job, Renderer};
use crate::resources::glyph_cache::GlyphCache;
use crate::resources::ResourceManager;
use glam::Vec2;

pub struct StringDrawable {
    ui_drawable: UIDrawable,
    glyph_cache: GlyphCache,
    pub should_draw: bool,
    pub screen_position: Vec2,
    pub center_text: bool,
}

impl StringDrawable {
    pub fn new(
        font_name: &str,
        point_size: f32,
        screen_position: Vec2,
        should_draw: bool,
    ) -> StringDrawable {
        StringDrawable {
            ui_drawable: UIDrawable { layers: vec![] },
            glyph_cache: GlyphCache::new(font_name, point_size),
            should_draw,
            screen_position,
            center_text: false,
        }
    }

    // create a new UIDrawable based on the requested content and position
    // screen_pos needs to be floats from 0.0 -> 1.0 because thats what the Renderer expects
    pub fn set(
        &mut self,
        content: &str,
        renderer: &Renderer,
        resource_manager: &mut ResourceManager,
    ) {
        let mut screen_pos = self.screen_position.clone();

        if self.center_text {
            println!("screen_pos_x {}", screen_pos[0]);
            // to center the screen position:
            let letter_width = 11.0 / 741.0;
            let word_width: f32 = (content.len() as f32 * letter_width);
            screen_pos[0] -= (word_width / 2.0);
            println!(
                "letter_width: {}, total letters: {}",
                letter_width,
                content.len()
            );
            println!("word_width: {}", word_width);
            println!("screen pos now: {}", screen_pos[0]);
        }

        // get UILayerTechniques for each glyph
        let layers: Vec<UILayerTechnique> = content
            .chars()
            .map(|char| {
                let glyph = self.glyph_cache.get_glyph(char, renderer, resource_manager);
                let render_position = screen_pos - glyph.get_origin_surface_offset(renderer);

                // increment the screen position by the glyph advance
                screen_pos += glyph.get_advance_surface_offset(renderer);

                UILayerTechnique::new(
                    renderer,
                    render_position,
                    glyph.get_bounds_surface_offset(renderer),
                    glyph.texture_offset,
                    glyph.texture_size,
                    glyph.get_texture(resource_manager),
                )
            })
            .collect();

        // set UIDrawable to new item
        self.ui_drawable = UIDrawable { layers }
    }
}

impl Drawable for StringDrawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a> {
        self.ui_drawable.render_graph(resources)
    }
}
