use std::{num::NonZeroU32, collections::{hash_map::{Values, Iter}, HashMap}};

pub struct FramebufferDescriptor {
    pub(super) color_attachments : Vec<wgpu::TextureView>,
    pub(super) depth_stencil_attachment : Option<wgpu::TextureView>,
    pub(super) clear_color : bool,
    pub(super) clear_depth : bool
}

pub enum RenderPassDescriptor<'a> {
    Graphics {
        source : &'a str,
        push_constant_ranges : &'a [wgpu::PushConstantRange],
        targets : Option<&'a [wgpu::ColorTargetState]>,
        primitive_state : wgpu::PrimitiveState,
        outputs_depth : bool,
        multisample_state : wgpu::MultisampleState,
        multiview : Option<NonZeroU32>
    },
    Compute {
        source : &'a str,
        bind_group_layouts : &'a [&'a wgpu::BindGroupLayout],
        push_constant_ranges : &'a [wgpu::PushConstantRange],
    }
}


/*
 * A render pass encapsulates everything needed for pipeline setup.
 * For now, ignore all the compute pass/render item stuff in this file since that will mostly
 * come in handy later if I want to do some fancy stuff in the end of this class. Most of it will
 * have to wait until the render graph is implemented.
 */
pub enum RenderPass {
    Graphics {
        shader : wgpu::ShaderModule,
        pipeline_layout : wgpu::PipelineLayout,
        render_pipeline : wgpu::RenderPipeline
    },
    Compute {
        shader : wgpu::ShaderModule,
        pipeline_layout : wgpu::PipelineLayout,
        compute_pipeline : wgpu::ComputePipeline
    }
}

/*
 * Ignore this push constant stuff since I've kind of forgotten about it and it's just more work.
 * It's just a way to store small amounts of data in a faster to access way. For now in our game, 
 * any uniforms will just be stored in a uniform buffer and accessed through a bind group.
 */
pub struct PushConstantData<'a> {
    stages : wgpu::ShaderStages,
    offset : u32,
    data : &'a [u8]
}

/*
 * A RenderItem stores all state for a draw call (or in the future, a compute dispatch call)
 */
#[derive(Clone)]
pub enum RenderItem<'a> {
    Graphics {
        pass_name : &'a str,
        framebuffer_name : &'a str,
        num_elements : u32,
        vertex_buffers : Vec<wgpu::BufferSlice<'a>>,
        index_buffer : Option<wgpu::BufferSlice<'a>>,
        index_format : wgpu::IndexFormat,
        bind_group : Vec<&'a wgpu::BindGroup>
    },
    Compute {
        pass_name : &'a str,
        bind_group : &'a [&'a wgpu::BindGroup],
        push_constants : &'a [PushConstantData<'a>]
    },
    Custom {
        pass_name : &'a str,
    }
}

/*
 * kinda ugly but whatevs
 * Encapsulates a list of RenderItems organized by framebuffer and render pass. 
 * It just does a bit of work when adding render items to organize everything properly.
 * Otherwise it doesn't care to organize further into vertex buffer or bind group bindings. 
 */
pub struct RenderJob<'a> {
    graphics_items : HashMap<String, HashMap<String, Vec<RenderItem<'a>>>>,
    compute_items : HashMap<String, Vec<RenderItem<'a>>>
}

impl<'a> RenderJob<'a> {
    pub fn new() -> Self {
        RenderJob { 
            graphics_items: HashMap::new(), 
            compute_items: HashMap::new() 
        }
    }

    pub fn add_item(&mut self, item : RenderItem<'a>) {
        match item {
            RenderItem::Graphics { 
                pass_name, 
                framebuffer_name, 
                ..
            } => {
                self.graphics_items
                    .entry(String::from(framebuffer_name))
                    .or_default()
                    .entry(String::from(pass_name))
                    .or_default()
                    .push(item);
            }
            RenderItem::Compute { 
                pass_name,
                ..
            } => {
                self.compute_items
                    .entry(String::from(pass_name))
                    .or_default()
                    .push(item);
            }
            RenderItem::Custom { .. } => {
                panic!("custom items not supported yet");
            }
        }
    }

    pub(super) fn compute_iter(&self) -> Values<String, Vec<RenderItem>> {
        self.compute_items.values()
    }

    pub(super) fn graphics_iter(&self) -> Iter<String, HashMap<String, Vec<RenderItem<'a>>>> {
        self.graphics_items.iter()
    }
}