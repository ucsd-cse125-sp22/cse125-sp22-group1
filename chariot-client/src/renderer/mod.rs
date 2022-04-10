use std::{
    borrow::Cow,
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

pub mod context;
mod reflection;
pub mod render_job;

use context::*;
use reflection::shader_metadata;
use render_job::*;

/*
 * The renderer handles the state management and transitions when sending commands to the GPU.
 * The idea is to submit a list of RenderItems (encapsulated in a RenderJob) and let the renderer generate
 * a list of commands for the GPU to process for a frame like this
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
 * These next two are skipped becase they are the same as the above
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
 * - At some point it would be nice to allow a drawable to submit render items for multiple passes so we could get a
 * deferred rendering setup, but for now it's just forward rendering. In other comments I'll call the
 * "submit multiple render_items for a drawable feature" the render graph.
 * - WGPU calls bind_framebuffer begin_render_pass. Kind of the same thing but not entirely. It also calls uniform sets bind groups.
 */

pub struct Renderer {
    context: Context,
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
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        context.surface.configure(&device, &config);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth buffer_tex"),
            size: wgpu::Extent3d {
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

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
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

                let pipeline_layout =
                    self.device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &self
                                .bind_group_layouts
                                .get(name)
                                .unwrap()
                                .iter()
                                .collect::<Vec<&wgpu::BindGroupLayout>>(),
                            push_constant_ranges: push_constant_ranges,
                        });

                let surface_target: &[wgpu::ColorTargetState] = &[self.surface_format.into()];
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
                            depth_stencil: if *outputs_depth {
                                Some(wgpu::DepthStencilState {
                                    format: Self::DEPTH_FORMAT,
                                    depth_write_enabled: true,
                                    depth_compare: wgpu::CompareFunction::Less, // 1.
                                    stencil: wgpu::StencilState::default(),     // 2.
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
        data: &[wgpu::BindingResource],
    ) -> wgpu::BindGroup {
        let bind_group_entries = data
            .iter()
            .enumerate()
            .map(|(idx, resource)| wgpu::BindGroupEntry {
                binding: u32::try_from(idx).unwrap(),
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
                .expect("invalid pass name")
                .get(group_num as usize)
                .expect("invalid group num"),
            entries: &bind_group_entries,
        })
    }

    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor, data: &[u8]) -> wgpu::Texture {
        self.device
            .create_texture_with_data(&self.queue, desc, data)
        // TODO: mipmapping
    }

    pub fn write_buffer<T>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        self.queue.write_buffer(buffer, 0, unsafe {
            core::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                std::mem::size_of::<T>() * data.len(),
            )
        });
    }

    pub fn render(&mut self, render_job: &RenderJob) {
        let frame = self
            .context
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        // kind of sketch to re-set this every frame
        {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.framebuffers.insert(
                String::from("surface"),
                FramebufferDescriptor {
                    color_attachments: vec![view],
                    depth_stencil_attachment: Some(
                        self.depth_texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                    clear_color: true,
                    clear_depth: true,
                },
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for framebuffer_passes in render_job.graphics_iter() {
            let framebuffer_desc = self
                .framebuffers
                .get(&String::from(framebuffer_passes.0))
                .expect("Unable to find frambuffer requested");

            let mut color_attachments = Vec::new();
            for color_tex_view in framebuffer_desc.color_attachments.iter() {
                color_attachments.push(wgpu::RenderPassColorAttachment {
                    view: &color_tex_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if framebuffer_desc.clear_color {
                            wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: true,
                    },
                });
            }

            let depth_stencil_attachment = match &framebuffer_desc.depth_stencil_attachment {
                Some(view) => Some(wgpu::RenderPassDepthStencilAttachment {
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
                }),
                None => None,
            };

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &color_attachments,
                depth_stencil_attachment: depth_stencil_attachment,
            });

            for pass_items in framebuffer_passes.1.iter() {
                let render_pass = self
                    .passes
                    .get(&String::from(pass_items.0))
                    .expect("Unable to find render pass requested");

                let graphics_pipeline = match render_pass {
                    RenderPass::Graphics {
                        render_pipeline, ..
                    } => render_pipeline,
                    _ => {
                        panic!("Unable to execute compute pass when framebuffer is bound");
                    }
                };

                rpass.set_pipeline(graphics_pipeline);

                for render_item in pass_items.1.iter() {
                    if let RenderItem::Graphics {
                        pass_name: _,
                        framebuffer_name: _,
                        num_elements,
                        vertex_buffers,
                        index_buffer,
                        index_format,
                        bind_group,
                    } = render_item
                    {
                        for (idx, buffer) in vertex_buffers.iter().enumerate() {
                            rpass.set_vertex_buffer(u32::try_from(idx).unwrap(), *buffer);
                        }

                        if let Some(buffer_slice) = index_buffer {
                            rpass.set_index_buffer(*buffer_slice, *index_format)
                        }

                        for (idx, bind_group) in bind_group.iter().enumerate() {
                            rpass.set_bind_group(u32::try_from(idx).unwrap(), bind_group, &[]);
                        }

                        // TODO: push constants

                        match index_buffer {
                            Some(_) => rpass.draw_indexed(0..*num_elements, 0, 0..1),
                            None => rpass.draw(0..*num_elements, 0..1),
                        }
                    } else {
                        panic!(
                            "Unable to execute non-graphics render item when framebuffer is bound"
                        );
                    }
                }
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
            size: wgpu::Extent3d {
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
}