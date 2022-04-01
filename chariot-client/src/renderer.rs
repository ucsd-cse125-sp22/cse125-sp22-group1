use core::panic;
use std::{collections::{HashMap, hash_map::{Iter, Values}}, string::String, 
    borrow::Cow, num::NonZeroU32
};

use winit::dpi::PhysicalSize;
pub struct Context {
    window : winit::window::Window,
    instance : wgpu::Instance,
    surface : wgpu::Surface,
    adapter : wgpu::Adapter
}

impl Context {
    pub fn new(event_loop : &winit::event_loop::EventLoop<()>) -> Self {
        let window = winit::window::Window::new(&event_loop).unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            }))
            .expect("Failed to find an appropriate adapter");

        Context {
            window,
            instance,
            surface,
            adapter
        }
    }
}

pub struct FramebufferDescriptor {
    color_attachments : Vec<wgpu::TextureView>,
    depth_stencil_attachment : Option<wgpu::TextureView>,
    clear_color : bool,
    clear_depth : bool
}

pub enum RenderPassDescriptor<'a> {
    Graphics {
        source : &'a str,
        vertex_buffer_layouts : &'a [wgpu::VertexBufferLayout<'a>],
        bind_group_layouts : &'a [&'a wgpu::BindGroupLayout],
        push_constant_ranges : &'a [wgpu::PushConstantRange],
        targets : Option<&'a [wgpu::ColorTargetState]>,
        primitive_state : wgpu::PrimitiveState,
        depth_stencil_state : Option<wgpu::DepthStencilState>,
        multisample_state : wgpu::MultisampleState,
        multiview : Option<NonZeroU32>
    },
    Compute {
        source : &'a str,
        bind_group_layouts : &'a [&'a wgpu::BindGroupLayout],
        push_constant_ranges : &'a [wgpu::PushConstantRange],
    }
}

enum RenderPass {
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

pub struct PushConstantData<'a> {
    stages : wgpu::ShaderStages,
    offset : u32,
    data : &'a [u8]
}

#[derive(Clone, Copy)]
pub enum RenderItem<'a> {
    Graphics {
        pass_name : &'a str,
        framebuffer_name : &'a str,
        num_vertices : u32,
        vertex_buffers : &'a [wgpu::BufferSlice<'a>],
        index_buffer : Option<wgpu::BufferSlice<'a>>,
        index_format : wgpu::IndexFormat,
        bind_group : &'a [&'a wgpu::BindGroup],
        push_constants : &'a [PushConstantData<'a>]
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

fn render_item_pass_name(render_item : RenderItem) -> &str{
    match render_item {
        RenderItem::Graphics { 
            pass_name, 
            .. 
        } => {
            pass_name
        },
        RenderItem::Compute { 
            pass_name, 
            .. 
        } => {
            pass_name
        },
        RenderItem::Custom { 
            pass_name 
        } => {
            pass_name
        }
    }
}

fn render_item_framebuffer_name(render_item : RenderItem) -> Option<&str>{
    match render_item {
        RenderItem::Graphics {
            framebuffer_name,
            .. 
        } => {
            Some(framebuffer_name)
        },
        RenderItem::Compute {
            .. 
        } => {
            None
        },
        RenderItem::Custom { 
            ..
        } => {
            None
        }
    }
}

// kinda ugly but whatevs
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

    fn compute_iter(&self) -> Values<String, Vec<RenderItem>> {
        self.compute_items.values()
    }

    fn graphics_iter(&self) -> Iter<String, HashMap<String, Vec<RenderItem<'a>>>> {
        self.graphics_items.iter()
    }
}

pub struct Renderer {
    context : Context,
    pub device : wgpu::Device,
    queue : wgpu::Queue,
    passes : HashMap<String, RenderPass>,
    framebuffers : HashMap<String, FramebufferDescriptor>,
    surface_format : wgpu::TextureFormat
}

impl Renderer {
    pub fn new(context : Context) -> Self {
        let (device, queue) = pollster::block_on(context.adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(context.adapter.limits()),
            },
            None,
        ))
        .expect("Failed to create device");

        let size = context.window.inner_size();
        let surface_format = context.surface.get_preferred_format(&context.adapter).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        context.surface.configure(&device, &config);

        let passes = HashMap::new();
        let framebuffers = HashMap::new();
        Renderer{
            context,
            device,
            queue,
            passes,
            framebuffers,
            surface_format
        }
    }

    pub fn register_framebuffer(&mut self, name : &str, framebuffer_desc : FramebufferDescriptor) {
        self.framebuffers.insert(String::from(name), framebuffer_desc);
    }

    // TODO: add index buffer layout
    pub fn register_pass(&mut self, name : &str, render_pass_desc : &RenderPassDescriptor) {
        match render_pass_desc {
            RenderPassDescriptor::Graphics{
                source,
                vertex_buffer_layouts,
                bind_group_layouts,
                push_constant_ranges,
                targets,
                primitive_state,
                depth_stencil_state,
                multisample_state,
                multiview
            } => {
                let shader = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
                });

                let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: bind_group_layouts,
                    push_constant_ranges: push_constant_ranges,
                });

                let surface_target : &[wgpu::ColorTargetState] = &[self.surface_format.into()];
                let target_formats = targets.unwrap_or(surface_target);

                let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(name),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: vertex_buffer_layouts,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: target_formats,
                    }),
                    primitive: *primitive_state,
                    depth_stencil: depth_stencil_state.clone(),
                    multisample: *multisample_state,
                    multiview: *multiview,
                });

                self.passes.insert(
                    String::from(name),
                    RenderPass::Graphics{
                        shader : shader,
                        pipeline_layout : pipeline_layout,
                        render_pipeline : render_pipeline
                    }
                );
            }
            RenderPassDescriptor::Compute { 
                source, 
                bind_group_layouts, 
                push_constant_ranges 
            } => {
                let shader = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
                });

                let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: bind_group_layouts,
                    push_constant_ranges: push_constant_ranges,
                });

                let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label : Some(name),
                    layout : Some(&pipeline_layout),
                    module : &shader,
                    entry_point : "main"
                });

                self.passes.insert(
                    String::from(name), 
                    RenderPass::Compute { 
                        shader: shader, 
                        pipeline_layout: pipeline_layout, 
                        compute_pipeline: compute_pipeline 
                    }
                );
            }
        }
    }

    pub fn render(&mut self, render_job : &RenderJob) {
        let frame = self.context.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.framebuffers.insert(String::from("surface"), FramebufferDescriptor { 
                color_attachments: vec![view], 
                depth_stencil_attachment: None, 
                clear_color: true, 
                clear_depth: true
            });
            // TODO: surface_with_depth
        }
        
        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        for framebuffer_passes in render_job.graphics_iter() {
            let framebuffer_desc = self.framebuffers.get(&String::from(framebuffer_passes.0))
                .expect("Unable to find frambuffer requested");

            let mut color_attachments = Vec::new();
            for color_tex_view in framebuffer_desc.color_attachments.iter() {
                color_attachments.push(wgpu::RenderPassColorAttachment{
                    view : &color_tex_view,
                    resolve_target : None,
                    ops : wgpu::Operations {
                        load : if framebuffer_desc.clear_color { 
                            wgpu::LoadOp::Clear(wgpu::Color::BLACK) 
                        } else { 
                            wgpu::LoadOp::Load 
                        },
                        store : true
                    }
                });
            }

            let depth_stencil_attachment = match &framebuffer_desc.depth_stencil_attachment {
                Some(view) => {
                    Some(wgpu::RenderPassDepthStencilAttachment {
                        view : &view,
                        depth_ops : Some(wgpu::Operations {
                            load : if framebuffer_desc.clear_depth {
                                wgpu::LoadOp::Clear(0.0)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store : true
                        }),
                        stencil_ops : None
                    })
                }
                None => None
            };

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &color_attachments,
                depth_stencil_attachment: depth_stencil_attachment
            });

            for pass_items in framebuffer_passes.1.iter() {
                let render_pass = self.passes.get(&String::from(pass_items.0))
                    .expect("Unable to find render pass requested");

                let graphics_pipeline = match render_pass {
                    RenderPass::Graphics { render_pipeline, .. } => {
                        render_pipeline
                    }
                    _ => {
                        panic!("Unable to execute compute pass when framebuffer is bound");
                    }
                };

                rpass.set_pipeline(graphics_pipeline);
                
                for render_item in pass_items.1.iter() {
                    if let RenderItem::Graphics { 
                        pass_name : _, 
                        framebuffer_name : _, 
                        num_vertices,
                        vertex_buffers, 
                        index_buffer, 
                        index_format, 
                        bind_group, 
                        push_constants : _ 
                    } = render_item {
                        for buffer_slot_pair in vertex_buffers.iter().zip(0..vertex_buffers.len()) {
                            rpass.set_vertex_buffer(
                                u32::try_from(buffer_slot_pair.1).unwrap(), 
                                *buffer_slot_pair.0
                            );
                        }

                        if let Some(buffer_slice) = index_buffer {
                            rpass.set_index_buffer(*buffer_slice, *index_format)
                        }

                        for bind_group_idx_pair in bind_group.iter().zip(0..bind_group.len()) {
                            rpass.set_bind_group(
                                u32::try_from(bind_group_idx_pair.1).unwrap(), 
                                bind_group_idx_pair.0, 
                                &[]
                            );
                        }

                        // TODO: push constants

                        match index_buffer {
                            Some(_) => rpass.draw_indexed(0..*num_vertices, 0, 0..1),
                            None => rpass.draw(0..*num_vertices, 0..1)
                        }
                    }
                    else {
                        panic!("Unable to execute non-graphics render item when framebuffer is bound");
                    }
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn handle_surface_resize(&mut self, size : PhysicalSize<u32>) {

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        self.context.surface.configure(&self.device, &config);
    }
}