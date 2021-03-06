use glam::UVec2;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicUsize, Ordering},
};
use wgpu::{Extent3d, ImageCopyTexture, ImageDataLayout};

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

pub mod context;
mod reflection;
pub mod render_job;
pub mod util;

use context::*;
use reflection::shader_metadata;
use render_job::*;

/*
 * The renderer handles the state setup and transitions when sending commands to the GPU.
 * The idea is to submit a graph of RenderItems (encapsulated in a RenderJob) and let the renderer generate
 * a list of commands for the GPU to process for a frame, like this:
 *
 * GPU command							| RenderItem field	| value
 * --------------------------------------------------------------------------------
 * bind framebuffer						| framebuffer_name	| "surface"				|
 *		bind pipeline					| pass_name			| "shade_pbr"	 		|	Draw Item 1
 * 			bind index bufffer 1		| index_buffer		| inds for model 1		|
 *			bind vertex buffers 1		| vertex_buffers	| verts for model 1		|
 *				bind uniform set 1.1	| bind_group[0]		| mvp buffer			|
 *				bind uniform set 1.2	| bind_group[1]		| material data buffer	|
 *					draw()
 * These next two are skipped because they are the same as the above
 * bind framebuffer	(SKIPPED)			| framebuffer_name	| "surface"				|
 *		bind pipeline (SKIPPED)			| pass_name			| "shade_pbr"	 		|   Draw Item 2
 * 			bind index bufffer 2		| index_buffer		| inds for model 1		|
 *			bind vertex buffers 2		| vertex_buffers	| verts for model 1		|
 *				bind uniform set 2.1	| bind_group[0]		| mvp buffer			|
 *				bind uniform set 2.2	| bind_group[1]		| material data buffer	|
 *					draw()
 *			...
 * Some things to note:
 * - Out of laziness, it is only the framebuffers and pipelines that are not re-bound if the previous item was the same.
 * If the index/vertex buffer and uniform buffers are the same as the previous they are re-bound anyways.
 * - WGPU calls bind_framebuffer begin_render_pass. Kind of the same thing but not entirely. It also calls uniform sets bind groups.
 */

pub struct FramebufferDescriptor {
    pub color_attachments: Vec<wgpu::TextureView>,
    pub depth_stencil_attachment: Option<wgpu::TextureView>,
    pub clear_color: Option<wgpu::Color>,
    pub clear_depth: bool,
}

pub struct Renderer {
    pub context: Context,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    passes: HashMap<String, RenderPass>,
    framebuffers: HashMap<String, FramebufferDescriptor>,
    bind_group_layouts: HashMap<String, Vec<wgpu::BindGroupLayout>>,
    surface_format: wgpu::TextureFormat,
    depth_texture: wgpu::Texture,
}

impl Renderer {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(context: Context) -> Self {
        let (device, queue) = pollster::block_on(
            context.adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(context.adapter.limits()),
                },
                None,
            ),
        )
        .expect("Failed to create device");

        let size = context.window.inner_size();
        let surface_format = context
            .surface
            .get_preferred_format(&context.adapter)
            .expect("unable to get preferred surface format initializing renderer");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        context.surface.configure(&device, &config);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_buffer_tex"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let passes = HashMap::new();
        let framebuffers = HashMap::new();
        let bind_group_layouts = HashMap::new();
        Renderer {
            context,
            device,
            queue,
            passes,
            framebuffers,
            bind_group_layouts,
            surface_format,
            depth_texture,
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> glam::Vec2 {
        let wind_size = self.context.window.inner_size();
        glam::vec2(
            (x as f32) / (wind_size.width as f32),
            (y as f32) / (wind_size.height as f32),
        )
    }

    pub fn pixel_x(&self, x: u32) -> f32 {
        (x as f32) / (self.context.window.inner_size().width as f32)
    }

    pub fn pixel_y(&self, y: u32) -> f32 {
        (y as f32) / (self.context.window.inner_size().height as f32)
    }

    pub fn _pixel_scale(&self, coord: (u32, u32)) -> glam::Vec2 {
        let width = 1280.0;
        let height = 720.0;
        glam::vec2((coord.0 as f32) / width, (coord.1 as f32) / height)
    }

    // request the operating system redraw the window contents via winit
    // this triggers the RedrawRequested event which then calls this render() again
    pub fn request_redraw(&self) {
        self.context.window.request_redraw();
    }

    pub fn register_framebuffer(&mut self, name: &str, framebuffer_desc: FramebufferDescriptor) {
        self.framebuffers
            .insert(String::from(name), framebuffer_desc);
    }

    // TODO: add index buffer layout
    pub fn register_pass(&mut self, name: &str, render_pass_desc: &RenderPassDescriptor) {
        match render_pass_desc {
            RenderPassDescriptor::Graphics {
                source,
                push_constant_ranges,
                targets,
                primitive_state,
                tests_depth,
                outputs_depth,
                multisample_state,
                multiview,
            } => {
                let shader_metadata = shader_metadata(source).expect(
                    format!("Error extracting metadata from shader for pass {}", name).as_str(),
                );
                let shader = self
                    .device
                    .create_shader_module(&wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
                    });

                let bind_group_layouts = shader_metadata
                    .bind_group_layouts
                    .iter()
                    .map(|(group_num, entries)| {
                        self.device
                            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                                label: Some(
                                    format!("{}_bind_group_layout_{}", name, group_num).as_str(),
                                ),
                                entries: &entries,
                            })
                    })
                    .collect::<Vec<wgpu::BindGroupLayout>>();
                self.bind_group_layouts
                    .insert(name.to_string(), bind_group_layouts);

                let vertex_buffer_layouts = shader_metadata
                    .vertex_attributes
                    .iter()
                    .map(|attrib| wgpu::VertexBufferLayout {
                        array_stride: attrib.format.size(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: std::slice::from_ref(attrib),
                    })
                    .collect::<Vec<wgpu::VertexBufferLayout>>();

                let bind_group_layouts = &self
                    .bind_group_layouts
                    .get(name)
                    .unwrap()
                    .iter()
                    .collect::<Vec<&wgpu::BindGroupLayout>>();

                let pipeline_layout =
                    self.device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &bind_group_layouts,
                            push_constant_ranges,
                        });

                let surface_color_state = wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                };
                let surface_target: &[wgpu::ColorTargetState] = &[surface_color_state];
                let target_formats = targets.unwrap_or(surface_target);

                let render_pipeline =
                    self.device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some(name),
                            layout: Some(&pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &shader,
                                entry_point: "vs_main",
                                buffers: &vertex_buffer_layouts,
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader,
                                entry_point: "fs_main",
                                targets: target_formats,
                            }),
                            primitive: *primitive_state,
                            depth_stencil: if *tests_depth || *outputs_depth {
                                Some(wgpu::DepthStencilState {
                                    format: Self::DEPTH_FORMAT,
                                    depth_write_enabled: *outputs_depth,
                                    depth_compare: if *tests_depth {
                                        wgpu::CompareFunction::Less
                                    } else {
                                        wgpu::CompareFunction::Always
                                    }, // 1.
                                    stencil: wgpu::StencilState::default(), // 2.
                                    bias: wgpu::DepthBiasState::default(),
                                })
                            } else {
                                None
                            },
                            multisample: *multisample_state,
                            multiview: *multiview,
                        });

                self.passes.insert(
                    String::from(name),
                    RenderPass::Graphics {
                        shader: shader,
                        pipeline_layout: pipeline_layout,
                        render_pipeline: render_pipeline,
                    },
                );
            }
            RenderPassDescriptor::Compute {
                source,
                bind_group_layouts,
                push_constant_ranges,
            } => {
                let shader = self
                    .device
                    .create_shader_module(&wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
                    });

                let pipeline_layout =
                    self.device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: bind_group_layouts,
                            push_constant_ranges: push_constant_ranges,
                        });

                let compute_pipeline =
                    self.device
                        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                            label: Some(name),
                            layout: Some(&pipeline_layout),
                            module: &shader,
                            entry_point: "main",
                        });

                self.passes.insert(
                    String::from(name),
                    RenderPass::Compute {
                        shader: shader,
                        pipeline_layout: pipeline_layout,
                        compute_pipeline: compute_pipeline,
                    },
                );
            }
        }
    }

    pub fn create_bind_group(
        &self,
        pass_name: &str,
        group_num: u32,
        data: &[(u32, wgpu::BindingResource)],
    ) -> wgpu::BindGroup {
        let bind_group_entries = data
            .iter()
            .map(|(binding, resource)| wgpu::BindGroupEntry {
                binding: *binding,
                resource: resource.clone(),
            })
            .collect::<Vec<wgpu::BindGroupEntry>>();

        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(
                format!(
                    "{}_bind_group_{}",
                    pass_name,
                    COUNTER.fetch_add(1, Ordering::Relaxed)
                )
                .as_str(),
            ),
            layout: self
                .bind_group_layouts
                .get(pass_name)
                .expect(format!("invalid pass name: {}", pass_name).as_str())
                .get(group_num as usize)
                .expect(format!("invalid group num: {}", group_num).as_str()),
            entries: &bind_group_entries,
        })
    }

    pub fn create_texture2d_init(
        &self,
        name: &str,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        data: &[u8],
    ) -> wgpu::Texture {
        self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some(name),
                size: Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
            },
            data,
        )
        // TODO: mipmapping
    }

    pub fn create_texture2d(
        &self,
        name: &str,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        })
    }

    // write pixel data into an existing texture
    pub fn write_texture2d(
        &self,
        texture: &wgpu::Texture,
        texture_offset: UVec2,
        data: &[u8],
        // the width and height of the data in pixels
        data_dimensions: UVec2,
        // depends on the underlying texture format
        bytes_per_block: u32,
    ) {
        self.queue.write_texture(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                // offset into the texture by the requested amount
                origin: wgpu::Origin3d {
                    x: texture_offset.x,
                    y: texture_offset.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(bytes_per_block * data_dimensions.x),
                rows_per_image: std::num::NonZeroU32::new(data_dimensions.y),
            },
            Extent3d {
                width: data_dimensions.x,
                height: data_dimensions.y,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn write_buffer<T>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        self.queue.write_buffer(buffer, 0, unsafe {
            core::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                std::mem::size_of::<T>() * data.len(),
            )
        });
    }

    fn new_wgpu_render_pass<'a>(
        framebuffer_name: &str,
        framebuffers: &'a HashMap<String, FramebufferDescriptor>,
        encoder: &'a mut wgpu::CommandEncoder,
        force_no_clear: bool,
    ) -> wgpu::RenderPass<'a> {
        let framebuffer_desc = framebuffers
            .get(&String::from(framebuffer_name))
            .expect(format!("Unable to find framebuffer requested: {}", framebuffer_name).as_str());

        let mut color_attachments = Vec::new();
        for color_tex_view in framebuffer_desc.color_attachments.iter() {
            color_attachments.push(wgpu::RenderPassColorAttachment {
                view: &color_tex_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if let Some(color) = framebuffer_desc.clear_color {
                        if force_no_clear {
                            wgpu::LoadOp::Load
                        } else {
                            wgpu::LoadOp::Clear(color)
                        }
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: true,
                },
            });
        }

        let depth_stencil_attachment =
            framebuffer_desc
                .depth_stencil_attachment
                .as_ref()
                .map(|view| wgpu::RenderPassDepthStencilAttachment {
                    view: &view,
                    depth_ops: Some(wgpu::Operations {
                        load: if framebuffer_desc.clear_depth {
                            wgpu::LoadOp::Clear(1.0)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: true,
                    }),
                    stencil_ops: None,
                });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: depth_stencil_attachment,
        })
    }

    fn is_same_framebuffer(next: Option<&(&str, &[RenderItem])>, fb_name: &str) -> bool {
        next.map(|(_, is)| render_item_framebuffer_name(is.first().unwrap()))
            .filter(|on| on.is_some())
            .map(|on| on.unwrap())
            .filter(|n| *n == fb_name)
            .is_some()
    }

    fn encode_graphics_pass<'a>(
        wgpu_rpass: &mut wgpu::RenderPass<'a>,
        pass_name: &str,
        passes: &'a HashMap<String, RenderPass>,
        items: &[RenderItem<'a>],
    ) {
        let render_pass = passes
            .get(&String::from(pass_name))
            .expect("Unable to find render pass requested");

        let graphics_pipeline = pass_render_pipeline(render_pass).unwrap();

        wgpu_rpass.set_pipeline(graphics_pipeline);

        for render_item in items.iter() {
            let (_, _, num_elements, vertex_buffers, index_buffer, index_format, bind_group) =
                try_unpack_graphics_item(render_item).unwrap();
            {
                for (idx, buffer) in vertex_buffers.iter().enumerate() {
                    wgpu_rpass.set_vertex_buffer(u32::try_from(idx).unwrap(), *buffer);
                }

                if let Some(buffer_slice) = index_buffer {
                    wgpu_rpass.set_index_buffer(*buffer_slice, index_format)
                }

                for (idx, bind_group) in bind_group.iter().enumerate() {
                    wgpu_rpass.set_bind_group(u32::try_from(idx).unwrap(), bind_group, &[]);
                }

                // TODO: push constants

                match index_buffer {
                    Some(_) => wgpu_rpass.draw_indexed(0..num_elements, 0, 0..1),
                    None => wgpu_rpass.draw(0..num_elements, 0..1),
                }
            }
        }
    }

    pub fn render(&mut self, render_job: &RenderJob) {
        let frame = self
            .context
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        // kind of sketch to re-set this every frame
        {
            let surface_view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.framebuffers.insert(
                String::from("surface"),
                FramebufferDescriptor {
                    color_attachments: vec![surface_view],
                    depth_stencil_attachment: Some(
                        self.depth_texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                    clear_color: Some(wgpu::Color::BLACK),
                    clear_depth: true,
                },
            );

            let surface_nodepth_view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.framebuffers.insert(
                String::from("surface_nodepth"),
                FramebufferDescriptor {
                    color_attachments: vec![surface_nodepth_view],
                    depth_stencil_attachment: None,
                    clear_color: Some(wgpu::Color::BLACK),
                    clear_depth: false,
                },
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut cleared_framebuffers = HashSet::<&str>::new();
        let mut job_iter = render_job.iter_bfs().peekable();
        while job_iter.peek().is_some() {
            let (pass_name, items) = job_iter.next().unwrap();
            let pass = self
                .passes
                .get(pass_name)
                .expect("Invalid pass name in graph");
            match pass {
                RenderPass::Graphics {
                    shader: _,
                    pipeline_layout: _,
                    render_pipeline: _,
                } => {
                    let fb_name = render_item_framebuffer_name(&items[0]).unwrap();
                    let force_no_clear = cleared_framebuffers.contains(fb_name);
                    let mut wgpu_rpass = Renderer::new_wgpu_render_pass(
                        fb_name,
                        &self.framebuffers,
                        &mut encoder,
                        force_no_clear,
                    );
                    Renderer::encode_graphics_pass(&mut wgpu_rpass, pass_name, &self.passes, items);
                    while Renderer::is_same_framebuffer(job_iter.peek(), fb_name) {
                        let (pass_name, items) = job_iter.next().unwrap();
                        Renderer::encode_graphics_pass(
                            &mut wgpu_rpass,
                            pass_name,
                            &self.passes,
                            items,
                        );
                    }

                    cleared_framebuffers.insert(fb_name);
                }
                _ => (),
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn handle_surface_resize(&mut self, size: PhysicalSize<u32>) {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        self.context.surface.configure(&self.device, &config);

        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth buffer_tex"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
    }

    pub fn surface_size(&self) -> PhysicalSize<u32> {
        self.context.window.inner_size()
    }
}
