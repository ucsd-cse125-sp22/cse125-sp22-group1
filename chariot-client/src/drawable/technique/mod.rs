use std::ops::Bound;

pub mod bloom;
pub mod composite;
pub mod downsample;
pub mod geometry;
pub mod hibl;
pub mod probe;
pub mod shade;
pub mod shadow;
pub mod simple_fsq;
pub mod skybox;
pub mod ui_layer;

pub use bloom::*;
pub use composite::*;
pub use downsample::*;
pub use geometry::*;
pub use hibl::*;
pub use probe::*;
pub use shade::*;
pub use shadow::*;
pub use simple_fsq::*;
pub use skybox::*;
pub use ui_layer::*;

use crate::drawable::*;
use crate::renderer::*;

pub trait Technique {
    const PASS_NAME: &'static str;
    fn register(renderer: &mut Renderer);
    fn update_once(renderer: &Renderer, context: &RenderContext) {}
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a>;
}
