use crate::drawable::technique::UILayerTechnique;
use crate::drawable::{Drawable, UIDrawable};
use crate::renderer::{render_job, Renderer};
use crate::resources::glyph_cache::GlyphCache;
use crate::resources::ResourceManager;
use glam::Vec2;

pub struct StringDrawable {
    ui_drawable: UIDrawable,
    glyph_cache: GlyphCache,
}

impl StringDrawable {
    pub fn new(font_name: &str, point_size: f32) -> StringDrawable {
        StringDrawable {
            ui_drawable: UIDrawable { layers: vec![] },
            glyph_cache: GlyphCache::new(font_name, point_size),
        }
    }

    // create a new UIDrawable based on the requested content and position
    pub fn set(
        &mut self,
        content: &str,
        mut screen_pos: Vec2,
        renderer: &Renderer,
        resource_manager: &mut ResourceManager,
    ) {
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
