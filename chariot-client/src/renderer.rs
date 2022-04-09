use core::panic;
use std::{
    borrow::Cow,
    collections::{
        hash_map::{Iter, Values},
        HashMap,
    },
    num::NonZeroU32,
    string::String,
    sync::atomic::{AtomicUsize, Ordering},
};

use naga;
use wgpu::util::DeviceExt;
use wgpu::BindGroupLayout;
use winit::dpi::PhysicalSize;

use crate::drawable::StaticMeshDrawable;

pub struct Context {
    window: winit::window::Window,
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
}

impl Context {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = winit::window::Window::new(&event_loop).unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
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
            adapter,
        }
    }
}

pub struct FramebufferDescriptor {
    color_attachments: Vec<wgpu::TextureView>,
    depth_stencil_attachment: Option<wgpu::TextureView>,
    clear_color: bool,
    clear_depth: bool,
}

pub enum RenderPassDescriptor<'a> {
    Graphics {
        source: &'a str,
        push_constant_ranges: &'a [wgpu::PushConstantRange],
        targets: Option<&'a [wgpu::ColorTargetState]>,
        primitive_state: wgpu::PrimitiveState,
        depth_stencil_state: Option<wgpu::DepthStencilState>,
        multisample_state: wgpu::MultisampleState,
        multiview: Option<NonZeroU32>,
    },
    Compute {
        source: &'a str,
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
        push_constant_ranges: &'a [wgpu::PushConstantRange],
    },
}

enum RenderPass {
    Graphics {
        shader: wgpu::ShaderModule,
        pipeline_layout: wgpu::PipelineLayout,
        render_pipeline: wgpu::RenderPipeline,
    },
    Compute {
        shader: wgpu::ShaderModule,
        pipeline_layout: wgpu::PipelineLayout,
        compute_pipeline: wgpu::ComputePipeline,
    },
}

pub struct PushConstantData<'a> {
    stages: wgpu::ShaderStages,
    offset: u32,
    data: &'a [u8],
}

#[derive(Clone)]
pub enum RenderItem<'a> {
    Graphics {
        pass_name: &'a str,
        framebuffer_name: &'a str,
        num_elements: u32,
        vertex_buffers: Vec<wgpu::BufferSlice<'a>>,
        index_buffer: Option<wgpu::BufferSlice<'a>>,
        index_format: wgpu::IndexFormat,
        bind_group: Vec<&'a wgpu::BindGroup>,
    },
    Compute {
        pass_name: &'a str,
        bind_group: &'a [&'a wgpu::BindGroup],
        push_constants: &'a [PushConstantData<'a>],
    },
    Custom {
        pass_name: &'a str,
    },
}

fn render_item_pass_name(render_item: RenderItem) -> &str {
    match render_item {
        RenderItem::Graphics { pass_name, .. } => pass_name,
        RenderItem::Compute { pass_name, .. } => pass_name,
        RenderItem::Custom { pass_name } => pass_name,
    }
}

fn render_item_framebuffer_name(render_item: RenderItem) -> Option<&str> {
    match render_item {
        RenderItem::Graphics {
            framebuffer_name, ..
        } => Some(framebuffer_name),
        RenderItem::Compute { .. } => None,
        RenderItem::Custom { .. } => None,
    }
}

// kinda ugly but whatevs
pub struct RenderJob<'a> {
    graphics_items: HashMap<String, HashMap<String, Vec<RenderItem<'a>>>>,
    compute_items: HashMap<String, Vec<RenderItem<'a>>>,
}

impl<'a> RenderJob<'a> {
    pub fn new() -> Self {
        RenderJob {
            graphics_items: HashMap::new(),
            compute_items: HashMap::new(),
        }
    }

    pub fn add_item(&mut self, item: RenderItem<'a>) {
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
            RenderItem::Compute { pass_name, .. } => {
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

fn scalar_uint_val(scalar: &naga::ScalarValue) -> u64 {
    match scalar {
        naga::ScalarValue::Sint(val) => u64::try_from(*val).unwrap(),
        naga::ScalarValue::Uint(val) => *val,
        _ => panic!("ahhh"),
    }
}

fn constant_size_val(module: &naga::Module, const_handle: naga::Handle<naga::Constant>) -> u64 {
    let constant = module.constants.try_get(const_handle).unwrap();
    match constant.inner {
        naga::ConstantInner::Scalar { width, value } => scalar_uint_val(&value),
        _ => panic!("bad index"),
    }
}

fn to_wgpu_tex_sample_type(scalar_kind: naga::ScalarKind) -> wgpu::TextureSampleType {
    match scalar_kind {
        naga::ScalarKind::Sint => wgpu::TextureSampleType::Sint,
        naga::ScalarKind::Uint => wgpu::TextureSampleType::Uint,
        naga::ScalarKind::Float => wgpu::TextureSampleType::Float { filterable: true },
        naga::ScalarKind::Bool => panic!("bool texture????"),
    }
}

fn to_wgpu_tex_dimension(dim: naga::ImageDimension) -> wgpu::TextureViewDimension {
    match dim {
        naga::ImageDimension::D1 => wgpu::TextureViewDimension::D1,
        naga::ImageDimension::D2 => wgpu::TextureViewDimension::D2,
        naga::ImageDimension::D3 => wgpu::TextureViewDimension::D3,
        naga::ImageDimension::Cube => wgpu::TextureViewDimension::Cube,
    }
}

fn to_wgpu_tex_access(access: naga::StorageAccess) -> wgpu::StorageTextureAccess {
    match access {
        naga::StorageAccess::LOAD => wgpu::StorageTextureAccess::ReadOnly,
        naga::StorageAccess::STORE => wgpu::StorageTextureAccess::WriteOnly,
        _ => wgpu::StorageTextureAccess::ReadWrite,
    }
}

fn to_wgpu_format(format: naga::StorageFormat) -> wgpu::TextureFormat {
    match format {
        naga::StorageFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        naga::StorageFormat::R8Snorm => wgpu::TextureFormat::R8Snorm,
        naga::StorageFormat::R8Uint => wgpu::TextureFormat::R8Uint,
        naga::StorageFormat::R8Sint => wgpu::TextureFormat::R8Sint,
        naga::StorageFormat::R16Uint => wgpu::TextureFormat::R16Uint,
        naga::StorageFormat::R16Sint => wgpu::TextureFormat::R16Sint,
        naga::StorageFormat::R16Float => wgpu::TextureFormat::R16Float,
        naga::StorageFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
        naga::StorageFormat::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
        naga::StorageFormat::Rg8Uint => wgpu::TextureFormat::Rg8Uint,
        naga::StorageFormat::Rg8Sint => wgpu::TextureFormat::Rg8Sint,
        naga::StorageFormat::R32Uint => wgpu::TextureFormat::R32Uint,
        naga::StorageFormat::R32Sint => wgpu::TextureFormat::R32Sint,
        naga::StorageFormat::R32Float => wgpu::TextureFormat::R32Float,
        naga::StorageFormat::Rg16Uint => wgpu::TextureFormat::Rg16Uint,
        naga::StorageFormat::Rg16Sint => wgpu::TextureFormat::Rg16Sint,
        naga::StorageFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
        naga::StorageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        naga::StorageFormat::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
        naga::StorageFormat::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
        naga::StorageFormat::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
        naga::StorageFormat::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
        naga::StorageFormat::Rg11b10Float => wgpu::TextureFormat::Rg11b10Float,
        naga::StorageFormat::Rg32Uint => wgpu::TextureFormat::Rg32Uint,
        naga::StorageFormat::Rg32Sint => wgpu::TextureFormat::Rg32Sint,
        naga::StorageFormat::Rg32Float => wgpu::TextureFormat::Rg32Float,
        naga::StorageFormat::Rgba16Uint => wgpu::TextureFormat::Rgba16Uint,
        naga::StorageFormat::Rgba16Sint => wgpu::TextureFormat::Rgba16Sint,
        naga::StorageFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        naga::StorageFormat::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
        naga::StorageFormat::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
        naga::StorageFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
    }
}

fn to_wgpu_binding_type(module: &naga::Module, naga_type: &naga::TypeInner) -> wgpu::BindingType {
    match naga_type {
        naga::TypeInner::Scalar { kind, width } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(*width as u64),
        },
        naga::TypeInner::Vector { size, kind, width } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(u64::from(*width) * u64::from(*size as u8)),
        },
        naga::TypeInner::Matrix {
            columns,
            rows,
            width,
        } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(
                u64::from(*width) * u64::from(*rows as u8) * u64::from(*columns as u8),
            ),
        },
        naga::TypeInner::Pointer { base, class } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false }, // TODO: read only?
            has_dynamic_offset: false,
            min_binding_size: None, // TODO: is this correct?
        },
        naga::TypeInner::ValuePointer {
            size,
            kind,
            width,
            class,
        } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None, // TODO: is this correct?
        },
        naga::TypeInner::Array { base, size, stride } => match size {
            naga::ArraySize::Constant(sz_handle) => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    constant_size_val(&module, *sz_handle) * u64::from(*stride),
                ),
            },
            naga::ArraySize::Dynamic => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO: not sure about this...
            },
        },
        naga::TypeInner::Struct {
            members: _,
            span: _,
        } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        naga::TypeInner::Image {
            dim,
            arrayed,
            class,
        } => match class {
            naga::ImageClass::Sampled { kind, multi } => wgpu::BindingType::Texture {
                sample_type: to_wgpu_tex_sample_type(*kind), // TODO: tex arrays
                view_dimension: to_wgpu_tex_dimension(*dim),
                multisampled: *multi,
            },
            naga::ImageClass::Storage { format, access } => wgpu::BindingType::StorageTexture {
                access: to_wgpu_tex_access(*access),
                format: to_wgpu_format(*format),
                view_dimension: to_wgpu_tex_dimension(*dim),
            },
            naga::ImageClass::Depth { multi } => panic!("depth not supported yet"),
        },
        naga::TypeInner::Sampler { comparison } => wgpu::BindingType::Sampler(if *comparison {
            wgpu::SamplerBindingType::Comparison
        } else {
            wgpu::SamplerBindingType::Filtering
        }),
        _ => panic!("uniform type not supportedd"),
    }
}

fn vertex_type_size(ty: &naga::TypeInner) -> u64 {
    match ty {
        naga::TypeInner::Scalar { kind, width } => u64::from(*width),
        naga::TypeInner::Vector { size, kind, width } => u64::from(*width) * u64::from(*size as u8),
        _ => panic!("invalid vertex type"),
    }
}

fn wgpu_vertex_type_attr(location: u32, ty: &naga::TypeInner) -> wgpu::VertexAttribute {
    *match ty {
        naga::TypeInner::Scalar { kind, width } => match kind {
            naga::ScalarKind::Sint => match width {
                4 => wgpu::vertex_attr_array![location => Sint32],
                _ => panic!(),
            },
            naga::ScalarKind::Uint => match width {
                4 => wgpu::vertex_attr_array![location => Uint32],
                _ => panic!(),
            },
            naga::ScalarKind::Float => match width {
                4 => wgpu::vertex_attr_array![location => Float32],
                8 => wgpu::vertex_attr_array![location => Float64],
                _ => panic!(),
            },
            _ => panic!(),
        },
        naga::TypeInner::Vector { size, kind, width } => match *size as u8 {
            2 => match kind {
                naga::ScalarKind::Sint => match width {
                    1 => wgpu::vertex_attr_array![location => Sint8x2],
                    2 => wgpu::vertex_attr_array![location => Sint16x2],
                    4 => wgpu::vertex_attr_array![location => Sint32x2],
                    _ => panic!(),
                },
                naga::ScalarKind::Uint => match width {
                    1 => wgpu::vertex_attr_array![location => Uint8x2],
                    2 => wgpu::vertex_attr_array![location => Uint16x2],
                    4 => wgpu::vertex_attr_array![location => Uint32x2],
                    _ => panic!(),
                },
                naga::ScalarKind::Float => match width {
                    2 => wgpu::vertex_attr_array![location => Float16x2],
                    4 => wgpu::vertex_attr_array![location => Float32x2],
                    8 => wgpu::vertex_attr_array![location => Float64x2],
                    _ => panic!(),
                },
                _ => panic!(),
            },
            3 => match kind {
                naga::ScalarKind::Sint => match width {
                    4 => wgpu::vertex_attr_array![location => Sint32x3],
                    _ => panic!(),
                },
                naga::ScalarKind::Uint => match width {
                    4 => wgpu::vertex_attr_array![location => Uint32x3],
                    _ => panic!(),
                },
                naga::ScalarKind::Float => match width {
                    4 => wgpu::vertex_attr_array![location => Float32x3],
                    8 => wgpu::vertex_attr_array![location => Float64x3],
                    _ => panic!(),
                },
                _ => panic!(),
            },
            4 => match kind {
                naga::ScalarKind::Sint => match width {
                    1 => wgpu::vertex_attr_array![location => Sint8x4],
                    2 => wgpu::vertex_attr_array![location => Sint16x4],
                    4 => wgpu::vertex_attr_array![location => Sint32x4],
                    _ => panic!(),
                },
                naga::ScalarKind::Uint => match width {
                    1 => wgpu::vertex_attr_array![location => Uint8x4],
                    2 => wgpu::vertex_attr_array![location => Uint16x4],
                    4 => wgpu::vertex_attr_array![location => Uint32x4],
                    _ => panic!(),
                },
                naga::ScalarKind::Float => match width {
                    2 => wgpu::vertex_attr_array![location => Float16x4],
                    4 => wgpu::vertex_attr_array![location => Float32x4],
                    8 => wgpu::vertex_attr_array![location => Float64x4],
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        },
        _ => panic!(),
    }
    .last()
    .unwrap()
}

fn has_location_binding(arg: &naga::FunctionArgument) -> bool {
    match &arg.binding {
        Some(binding) => match binding {
            naga::Binding::Location {
                location: _,
                interpolation: _,
                sampling: _,
            } => true,
            naga::Binding::BuiltIn(_) => false,
        },
        None => false,
    }
}

struct ShaderMetadata {
    bind_group_layouts: HashMap<u32, Vec<wgpu::BindGroupLayoutEntry>>,
    vertex_attributes: Vec<wgpu::VertexAttribute>,
}

fn shader_metadata(source: &str) -> ShaderMetadata {
    let mut bind_group_layouts = HashMap::<u32, Vec<wgpu::BindGroupLayoutEntry>>::new();
    let naga_module = naga::front::wgsl::parse_str(source).unwrap();
    for (_, global_var) in naga_module.global_variables.iter() {
        if let Some(binding) = &global_var.binding {
            let binding_type = naga_module.types.get_handle(global_var.ty).unwrap();
            bind_group_layouts
                .entry(binding.group)
                .or_default()
                .push(wgpu::BindGroupLayoutEntry {
                    binding: binding.binding,
                    visibility: wgpu::ShaderStages::all(),
                    ty: to_wgpu_binding_type(&naga_module, &binding_type.inner),
                    count: None, // TODO: arrays not supported yet
                });
        }
    }

    let mut vertex_attrs = Vec::new();
    let maybe_vs_fun = naga_module
        .entry_points
        .iter()
        .filter(|ep| ep.name.eq(&String::from("vs_main")))
        .last();

    if let Some(vs_fun_pair) = maybe_vs_fun {
        let vs_fun = &vs_fun_pair.function;
        let loc_binding_iter = vs_fun
            .arguments
            .iter()
            .filter(|arg| has_location_binding(arg));
        for arg in loc_binding_iter {
            let arg_binding = arg.binding.as_ref().unwrap();
            let arg_type = naga_module.types.get_handle(arg.ty).unwrap();
            if let naga::Binding::Location {
                location,
                interpolation: _,
                sampling: _,
            } = arg_binding
            {
                vertex_attrs.push(wgpu_vertex_type_attr(*location, &arg_type.inner));
            };
        }
    }

    ShaderMetadata {
        bind_group_layouts: bind_group_layouts,
        vertex_attributes: vertex_attrs,
    }
}

pub struct Renderer {
    context: Context,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    passes: HashMap<String, RenderPass>,
    framebuffers: HashMap<String, FramebufferDescriptor>,
    bind_group_layouts: HashMap<String, Vec<wgpu::BindGroupLayout>>,
    surface_format: wgpu::TextureFormat,
}

impl Renderer {
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
                depth_stencil_state,
                multisample_state,
                multiview,
            } => {
                let shader_metadata = shader_metadata(source);
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
                            depth_stencil: depth_stencil_state.clone(),
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

    pub fn render(&mut self, render_job: &RenderJob) {
        let frame = self
            .context
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.framebuffers.insert(
                String::from("surface"),
                FramebufferDescriptor {
                    color_attachments: vec![view],
                    depth_stencil_attachment: None,
                    clear_color: true,
                    clear_depth: true,
                },
            );
            // TODO: surface_with_depth
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
                            wgpu::LoadOp::Clear(0.0)
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
    }
}
