use std::{
    collections::{BTreeMap, HashMap},
    result::Result,
};

/*
 * This file contains the function shader_metadata (at the bottom), which parses a shader using the naga library,
 * a part of the wgpu project.
 *
 * wgpu requires a lot of information upfront about a program which can be inferred from the shader code. For example,
 * some info on the layout of the vertex buffer and the uniform types are all in the shader code. It can be annoying to
 * change all your uniform data in the wgpu pipeline setup every time you change or add a uniform in a shader, so this
 * makes everything easier.
 *
 * Right now it's just getting vertex buffer and uniform buffer information, but I might also add framebuffer info
 * at some time as well.
 */

fn scalar_uint_val(scalar: &naga::ScalarValue) -> Result<u64, &'static str> {
    match scalar {
        naga::ScalarValue::Sint(val) => Ok(u64::try_from(*val).unwrap()),
        naga::ScalarValue::Uint(val) => Ok(*val),
        _ => Err("Scalar type cannot be used as index"),
    }
}

fn constant_size_val(
    module: &naga::Module,
    const_handle: naga::Handle<naga::Constant>,
) -> Result<u64, &'static str> {
    let constant = module.constants.try_get(const_handle).unwrap();
    match constant.inner {
        naga::ConstantInner::Scalar { width, value } => scalar_uint_val(&value),
        _ => Err("Type cannot be used as index"),
    }
}

fn to_wgpu_tex_sample_type(
    scalar_kind: naga::ScalarKind,
) -> Result<wgpu::TextureSampleType, &'static str> {
    match scalar_kind {
        naga::ScalarKind::Sint => Ok(wgpu::TextureSampleType::Sint),
        naga::ScalarKind::Uint => Ok(wgpu::TextureSampleType::Uint),
        naga::ScalarKind::Float => Ok(wgpu::TextureSampleType::Float { filterable: true }),
        naga::ScalarKind::Bool => Err("bool texture????"),
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

fn to_wgpu_binding_type(
    module: &naga::Module,
    naga_type: &naga::TypeInner,
) -> Result<wgpu::BindingType, &'static str> {
    match naga_type {
        naga::TypeInner::Scalar { kind, width } => Ok(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(*width as u64),
        }),
        naga::TypeInner::Vector { size, kind, width } => Ok(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(u64::from(*width) * u64::from(*size as u8)),
        }),
        naga::TypeInner::Matrix {
            columns,
            rows,
            width,
        } => Ok(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(
                u64::from(*width) * u64::from(*rows as u8) * u64::from(*columns as u8),
            ),
        }),
        naga::TypeInner::Pointer { base, class } => Ok(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false }, // TODO: read only?
            has_dynamic_offset: false,
            min_binding_size: None, // TODO: is this correct?
        }),
        naga::TypeInner::ValuePointer {
            size,
            kind,
            width,
            class,
        } => Ok(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None, // TODO: is this correct?
        }),
        naga::TypeInner::Array { base, size, stride } => match size {
            naga::ArraySize::Constant(sz_handle) => Ok(wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    constant_size_val(&module, *sz_handle)? * u64::from(*stride),
                ),
            }),
            naga::ArraySize::Dynamic => Ok(wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None, // TODO: not sure about this...
            }),
        },
        naga::TypeInner::Struct {
            members: _,
            span: _,
        } => Ok(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }),
        naga::TypeInner::Image {
            dim,
            arrayed,
            class,
        } => match class {
            naga::ImageClass::Sampled { kind, multi } => Ok(wgpu::BindingType::Texture {
                sample_type: to_wgpu_tex_sample_type(*kind)?, // TODO: tex arrays
                view_dimension: to_wgpu_tex_dimension(*dim),
                multisampled: *multi,
            }),
            naga::ImageClass::Storage { format, access } => Ok(wgpu::BindingType::StorageTexture {
                access: to_wgpu_tex_access(*access),
                format: to_wgpu_format(*format),
                view_dimension: to_wgpu_tex_dimension(*dim),
            }),
            naga::ImageClass::Depth { multi } => Err("Depth not supported yet"),
        },
        naga::TypeInner::Sampler { comparison } => Ok(wgpu::BindingType::Sampler(if *comparison {
            wgpu::SamplerBindingType::Comparison
        } else {
            wgpu::SamplerBindingType::Filtering
        })),
        _ => Err("Uniform type not supportedd"),
    }
}

fn vertex_type_size(ty: &naga::TypeInner) -> Result<u64, &'static str> {
    match ty {
        naga::TypeInner::Scalar { kind, width } => Ok(u64::from(*width)),
        naga::TypeInner::Vector { size, kind, width } => {
            Ok(u64::from(*width) * u64::from(*size as u8))
        }
        _ => Err("Invalid vertex type"),
    }
}

/*
 * The wgpu macro wgpu::vertex_attr_array![location => type] returns an array but I just want to get the single
 * value out of it, so there is a bit of sketchy unwrapping going on here
 */
fn wgpu_vertex_type_attr(
    location: u32,
    ty: &naga::TypeInner,
) -> Result<wgpu::VertexAttribute, &'static str> {
    Ok(*match ty {
        naga::TypeInner::Scalar { kind, width } => match kind {
            naga::ScalarKind::Sint => match width {
                4 => Ok(wgpu::vertex_attr_array![location => Sint32]),
                _ => Err("Unsupported scalar int width"),
            },
            naga::ScalarKind::Uint => match width {
                4 => Ok(wgpu::vertex_attr_array![location => Uint32]),
                _ => Err("Unsupported scalar uint width"),
            },
            naga::ScalarKind::Float => match width {
                4 => Ok(wgpu::vertex_attr_array![location => Float32]),
                8 => Ok(wgpu::vertex_attr_array![location => Float64]),
                _ => Err("Unsupported scalar float width"),
            },
            _ => Err("Unsupported scalar type"),
        },
        naga::TypeInner::Vector { size, kind, width } => match *size as u8 {
            2 => match kind {
                naga::ScalarKind::Sint => match width {
                    1 => Ok(wgpu::vertex_attr_array![location => Sint8x2]),
                    2 => Ok(wgpu::vertex_attr_array![location => Sint16x2]),
                    4 => Ok(wgpu::vertex_attr_array![location => Sint32x2]),
                    _ => Err("Unsupported vec2 int width"),
                },
                naga::ScalarKind::Uint => match width {
                    1 => Ok(wgpu::vertex_attr_array![location => Uint8x2]),
                    2 => Ok(wgpu::vertex_attr_array![location => Uint16x2]),
                    4 => Ok(wgpu::vertex_attr_array![location => Uint32x2]),
                    _ => Err("Unsupported vec2 uint width"),
                },
                naga::ScalarKind::Float => match width {
                    2 => Ok(wgpu::vertex_attr_array![location => Float16x2]),
                    4 => Ok(wgpu::vertex_attr_array![location => Float32x2]),
                    8 => Ok(wgpu::vertex_attr_array![location => Float64x2]),
                    _ => Err("Unsupported vec2 float width"),
                },
                _ => Err("Unsupported vec2 component type"),
            },
            3 => match kind {
                naga::ScalarKind::Sint => match width {
                    4 => Ok(wgpu::vertex_attr_array![location => Sint32x3]),
                    _ => Err("Unsupported vec3 int width"),
                },
                naga::ScalarKind::Uint => match width {
                    4 => Ok(wgpu::vertex_attr_array![location => Uint32x3]),
                    _ => Err("Unsupported vec3 uint width"),
                },
                naga::ScalarKind::Float => match width {
                    4 => Ok(wgpu::vertex_attr_array![location => Float32x3]),
                    8 => Ok(wgpu::vertex_attr_array![location => Float64x3]),
                    _ => Err("Unsupported vec3 float width"),
                },
                _ => Err("Unsupported vec3 component type"),
            },
            4 => match kind {
                naga::ScalarKind::Sint => match width {
                    1 => Ok(wgpu::vertex_attr_array![location => Sint8x4]),
                    2 => Ok(wgpu::vertex_attr_array![location => Sint16x4]),
                    4 => Ok(wgpu::vertex_attr_array![location => Sint32x4]),
                    _ => Err("Unsupported vec4 int width"),
                },
                naga::ScalarKind::Uint => match width {
                    1 => Ok(wgpu::vertex_attr_array![location => Uint8x4]),
                    2 => Ok(wgpu::vertex_attr_array![location => Uint16x4]),
                    4 => Ok(wgpu::vertex_attr_array![location => Uint32x4]),
                    _ => Err("Unsupported vec4 uint width"),
                },
                naga::ScalarKind::Float => match width {
                    2 => Ok(wgpu::vertex_attr_array![location => Float16x4]),
                    4 => Ok(wgpu::vertex_attr_array![location => Float32x4]),
                    8 => Ok(wgpu::vertex_attr_array![location => Float64x4]),
                    _ => Err("Unsupported vec4 float width"),
                },
                _ => Err("Unsupported vec4 component type"),
            },
            _ => Err("Unsupported number of components in vec"),
        },
        _ => Err("Unsupported vertex attrib type"),
    }?
    .last()
    .unwrap())
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

pub struct ShaderMetadata {
    pub(super) bind_group_layouts: BTreeMap<u32, Vec<wgpu::BindGroupLayoutEntry>>, // Tree needed here so it is iterated in order
    pub(super) vertex_attributes: Vec<wgpu::VertexAttribute>,
}

pub fn shader_metadata(source: &str) -> Result<ShaderMetadata, &'static str> {
    let mut bind_group_layouts = BTreeMap::<u32, Vec<wgpu::BindGroupLayoutEntry>>::new();
    let naga_module = naga::front::wgsl::parse_str(source).unwrap();
    for (_, global_var) in naga_module.global_variables.iter() {
        if let Some(binding) = &global_var.binding {
            let binding_type = naga_module.types.get_handle(global_var.ty).unwrap();
            let entry = wgpu::BindGroupLayoutEntry {
                binding: binding.binding,
                visibility: wgpu::ShaderStages::all(),
                ty: to_wgpu_binding_type(&naga_module, &binding_type.inner)?,
                count: None, // TODO: arrays not supported yet
            };
            bind_group_layouts
                .entry(binding.group)
                .or_default()
                .push(entry);
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
                vertex_attrs.push(wgpu_vertex_type_attr(*location, &arg_type.inner)?);
            };
        }
    }

    Ok(ShaderMetadata {
        bind_group_layouts: bind_group_layouts,
        vertex_attributes: vertex_attrs,
    })
}
