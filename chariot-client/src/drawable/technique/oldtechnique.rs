use crate::renderer::*;
use crate::resources::material::MaterialBuilder;
use crate::resources::*;
use wgpu::util::DeviceExt;

pub trait Technique {
    fn render_item<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderItem<'a>;
}

pub(super) struct TransformUniform<const NUM_ELEMS: usize> {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<const NUM_ELEMS: usize> TransformUniform<NUM_ELEMS> {
    pub fn new(renderer: &Renderer, pass_name: &str, group: u32) -> Self {
        let uniform_init = [glam::Mat4::IDENTITY; NUM_ELEMS];
        let xform_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buf"),
                contents: unsafe {
                    core::slice::from_raw_parts(
                        uniform_init.as_ptr() as *const u8,
                        std::mem::size_of::<[glam::Mat4; NUM_ELEMS]>(),
                    )
                },
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::MAP_WRITE,
            });

        let xform_bind_group =
            renderer.create_bind_group(pass_name, group, &[(0, xform_buffer.as_entire_binding())]);

        return TransformUniform {
            buffer: xform_buffer,
            bind_group: xform_bind_group,
        };
    }

    pub fn update(&self, renderer: &Renderer, data: &[glam::Mat4; NUM_ELEMS]) {
        renderer.write_buffer(&self.buffer, data);
    }
}

pub struct GeometryDrawTechnique {
    material: MaterialHandle,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub(super) mvp_xform: TransformUniform<3>,
}

impl GeometryDrawTechnique {
    const PASS_NAME: &'static str = "geometry";
    const FRAMEBUFFER_NAME: &'static str = "geometry_out";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        material: MaterialHandle,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
    ) -> Self {
        Self {
            material: material,
            static_mesh: static_mesh,
            submesh_idx: submesh_idx,
            mvp_xform: TransformUniform::new(renderer, Self::PASS_NAME, 0),
        }
    }
}

impl Technique for GeometryDrawTechnique {
    fn render_item<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderItem<'a> {
        let static_mesh = resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");
        let material = resources
            .materials
            .get(&self.material)
            .expect("invalid material handle");

        let vertex_buffers_with_ranges = static_mesh
            .vertex_buffers
            .iter()
            .zip(static_mesh.submeshes[self.submesh_idx].vertex_ranges.iter());

        let mut bind_group_refs = vec![&self.mvp_xform.bind_group];
        bind_group_refs.extend(material.bind_groups.values());
        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: Self::FRAMEBUFFER_NAME,
            num_elements: static_mesh.submeshes[self.submesh_idx].num_elements,
            vertex_buffers: vertex_buffers_with_ranges
                .map(|(buffer, range)| buffer.slice(*range))
                .collect::<Vec<wgpu::BufferSlice>>(),
            index_buffer: match &static_mesh.index_buffer {
                Some(buffer) => {
                    Some(buffer.slice(static_mesh.submeshes[self.submesh_idx].index_range.unwrap()))
                }
                None => None,
            },
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}

pub struct ShadowDrawTechnique {
    pass_name: String,
    framebuffer_name: String,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub(super) mvp_xform: TransformUniform<1>,
}

impl ShadowDrawTechnique {
    pub fn new(
        renderer: &Renderer,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
        pass_name: &str,
        framebuffer_name: &str,
    ) -> Self {
        Self {
            pass_name: pass_name.to_string(),
            framebuffer_name: framebuffer_name.to_string(),
            static_mesh: static_mesh,
            submesh_idx: submesh_idx,
            mvp_xform: TransformUniform::new(renderer, pass_name, 0),
        }
    }
}

impl Technique for ShadowDrawTechnique {
    fn render_item<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderItem<'a> {
        let static_mesh = resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");

        let vertex_buffers_with_ranges = static_mesh
            .vertex_buffers
            .iter()
            .zip(static_mesh.submeshes[self.submesh_idx].vertex_ranges.iter());

        let bind_group_refs = vec![&self.mvp_xform.bind_group];

        render_job::RenderItem::Graphics {
            pass_name: self.pass_name.as_str(),
            framebuffer_name: self.framebuffer_name.as_str(),
            num_elements: static_mesh.submeshes[self.submesh_idx].num_elements,
            vertex_buffers: vertex_buffers_with_ranges
                .map(|(buffer, range)| buffer.slice(*range))
                .collect::<Vec<wgpu::BufferSlice>>(),
            index_buffer: match &static_mesh.index_buffer {
                Some(buffer) => {
                    Some(buffer.slice(static_mesh.submeshes[self.submesh_idx].index_range.unwrap()))
                }
                None => None,
            },
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}

pub struct ShadeDirectTechnique {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: material::Material,
    view_xform: TransformUniform<2>,
    light_xform: TransformUniform<1>,
}

impl ShadeDirectTechnique {
    const PASS_NAME: &'static str = "shade_direct";
    const FRAMEBUFFER_NAME: &'static str = "shade_direct_out";

    pub fn new(renderer: &Renderer, resources: &ResourceManager) -> Self {
        let verts_data: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let inds_data: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fsq_verts"),
                contents: bytemuck::cast_slice(&verts_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fsq_inds"),
                contents: bytemuck::cast_slice(&inds_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        // I don't like the idea of making new views all the time but it's too built into the design now; maybe later I'll fix it
        let load_tex = |name: &str, idx: usize| {
            resources
                .framebuffer_tex(name, idx)
                .expect(
                    format!(
                        "FSQTechnique ({}) requires geometry_out framebuffer to be registered",
                        Self::PASS_NAME
                    )
                    .as_str(),
                )
                .create_view(&wgpu::TextureViewDescriptor::default())
        };

        let color_tex_view = load_tex("geometry_out", 0);
        let normal_tex_view = load_tex("geometry_out", 1);
        let depth_tex_view = load_tex("geometry_out", 2);
        let shadow_tex_view = load_tex("shadow_out1", 0);

        let view_xform = TransformUniform::<2>::new(renderer, Self::PASS_NAME, 1);
        let light_xform = TransformUniform::<1>::new(renderer, Self::PASS_NAME, 2);

        let material = material::MaterialBuilder::new(renderer, Self::PASS_NAME)
            .texture_resource(0, 0, color_tex_view)
            .texture_resource(0, 1, normal_tex_view)
            .texture_resource(0, 2, depth_tex_view)
            .texture_resource(0, 3, shadow_tex_view)
            .produce();

        Self {
            vertex_buffer,
            index_buffer,
            material,
            view_xform,
            light_xform,
        }
    }

    pub fn update_view_data(&self, renderer: &Renderer, view: glam::Mat4, proj: glam::Mat4) {
        let inv_view = view.inverse();
        let inv_proj = proj.inverse();
        self.view_xform.update(renderer, &[inv_view, inv_proj]);
    }

    pub fn update_light_data(&self, renderer: &Renderer, view: glam::Mat4, proj: glam::Mat4) {
        let view_proj = proj * view;
        self.light_xform.update(renderer, &[view_proj]);
    }
}

impl Technique for ShadeDirectTechnique {
    fn render_item<'a>(&'a self, _: &'a ResourceManager) -> render_job::RenderItem<'a> {
        let bind_groups = self
            .material
            .bind_groups
            .values()
            .chain(std::slice::from_ref(&self.view_xform.bind_group).iter())
            .chain(std::slice::from_ref(&self.light_xform.bind_group).iter())
            .collect();

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: Self::FRAMEBUFFER_NAME,
            num_elements: 6,
            vertex_buffers: vec![self.vertex_buffer.slice(..)],
            index_buffer: Some(self.index_buffer.slice(..)),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}

pub struct PostprocessTechnique {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: material::Material,
}

impl PostprocessTechnique {
    const PASS_NAME: &'static str = "postprocess";
    const FRAMEBUFFER_NAME: &'static str = "surface_nodepth";

    pub fn new(renderer: &Renderer, resources: &ResourceManager) -> Self {
        let verts_data: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let inds_data: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fsq_verts"),
                contents: bytemuck::cast_slice(&verts_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fsq_inds"),
                contents: bytemuck::cast_slice(&inds_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        // I don't like the idea of making new views all the time but it's too built into the design now; maybe later I'll fix it
        let load_tex = |name: &str, idx: usize| {
            resources
                .framebuffer_tex(name, idx)
                .expect(
                    format!(
                        "FSQTechnique ({}) requires shade_direct_out framebuffer to be registered",
                        Self::PASS_NAME
                    )
                    .as_str(),
                )
                .create_view(&wgpu::TextureViewDescriptor::default())
        };

        let color_tex_view = load_tex("shade_direct_out", 0);
        let depth_tex_view = load_tex("geometry_out", 2);
        let particles_color_tex_view = load_tex("particles_out", 0);
        let particles_depth_tex_view = load_tex("particles_out", 1);

        let material = material::MaterialBuilder::new(renderer, Self::PASS_NAME)
            .texture_resource(0, 0, color_tex_view)
            .texture_resource(0, 1, depth_tex_view)
            .texture_resource(0, 2, particles_color_tex_view)
            .texture_resource(0, 3, particles_depth_tex_view)
            .produce();

        Self {
            vertex_buffer,
            index_buffer,
            material,
        }
    }
}

impl Technique for PostprocessTechnique {
    fn render_item<'a>(&'a self, _: &'a ResourceManager) -> render_job::RenderItem<'a> {
        let bind_groups = self.material.bind_groups.values().collect();

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: Self::FRAMEBUFFER_NAME,
            num_elements: 6,
            vertex_buffers: vec![self.vertex_buffer.slice(..)],
            index_buffer: Some(self.index_buffer.slice(..)),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}

pub struct UILayerTechnique {
    pub vertex_buffer: wgpu::Buffer,
    texcoord_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: material::Material,
    pub tint_color: [f32; 4],
    pub pos: glam::Vec2,
    pub size: glam::Vec2,
}

impl UILayerTechnique {
    const PASS_NAME: &'static str = "ui";
    const FRAMEBUFFER_NAME: &'static str = "surface_nodepth";

    pub fn create_verts_data(pos: glam::Vec2, size: glam::Vec2) -> [[f32; 2]; 4] {
        let pos_ndc = glam::vec2(pos.x, 1.0 - pos.y) * 2.0 - 1.0;
        let size_ndc = glam::vec2(size.x, -size.y) * 2.0;
        [
            [pos_ndc.x, pos_ndc.y],
            [pos_ndc.x + size_ndc.x, pos_ndc.y],
            [pos_ndc.x + size_ndc.x, pos_ndc.y + size_ndc.y],
            [pos_ndc.x, pos_ndc.y + size_ndc.y],
        ]
    }

    pub fn update_pos(&mut self, renderer: &Renderer, pos: glam::Vec2) {
        let raw_data = UILayerTechnique::create_verts_data(pos, self.size);
        let verts_data: &[u8] = bytemuck::cast_slice(&raw_data);
        renderer.write_buffer(&self.vertex_buffer, verts_data);
        self.pos = pos;
    }

    pub fn update_size(&mut self, renderer: &Renderer, size: glam::Vec2) {
        let raw_data = UILayerTechnique::create_verts_data(self.pos, size);
        let verts_data: &[u8] = bytemuck::cast_slice(&raw_data);
        renderer.write_buffer(&self.vertex_buffer, verts_data);
        self.size = size;
    }

    pub fn update_color(&mut self, renderer: &Renderer, color: [f32; 4]) {
        let raw_data: &[u8] = bytemuck::cast_slice(&color);
        self.material.bind_groups.get(&0).unwrap();
    }

    pub fn new(
        renderer: &Renderer,
        pos: glam::Vec2,
        size: glam::Vec2,
        tc_pos: glam::Vec2,
        tc_size: glam::Vec2,
        texture: &wgpu::Texture,
    ) -> Self {
        let raw_verts_data = UILayerTechnique::create_verts_data(pos, size);
        let verts_data: &[u8] = bytemuck::cast_slice(&raw_verts_data);
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ui_verts"),
                contents: bytemuck::cast_slice(&verts_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let texcoord_data: [[f32; 2]; 4] = [
            [tc_pos.x, tc_pos.y],
            [tc_pos.x + tc_size.x, tc_pos.y],
            [tc_pos.x + tc_size.x, tc_pos.y + tc_size.y],
            [tc_pos.x, tc_pos.y + tc_size.y],
        ];
        let inds_data: [u16; 6] = [0, 2, 1, 0, 3, 2];

        let texcoord_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ui_texcoords"),
                    contents: bytemuck::cast_slice(&texcoord_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ui_inds"),
                contents: bytemuck::cast_slice(&inds_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        let material = MaterialBuilder::new(renderer, Self::PASS_NAME)
            .texture_resource(
                0,
                0,
                texture.create_view(&wgpu::TextureViewDescriptor::default()),
            )
            .buffer_resource(
                0,
                1,
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("ui_color"),
                        contents: bytemuck::cast_slice(&([1.0, 1.0, 1.0, 1.0 ] as [f32; 4])),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
            )
            .produce();

        Self {
            vertex_buffer,
            texcoord_buffer,
            index_buffer,
            material,
            tint_color: [1.0, 1.0, 1.0, 1.0],
            pos,
            size,
        }
    }
}

impl Technique for UILayerTechnique {
    fn render_item<'a>(&'a self, _: &'a ResourceManager) -> render_job::RenderItem<'a> {
        let bind_groups = self.material.bind_groups.values().collect();

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: Self::FRAMEBUFFER_NAME,
            num_elements: 6,
            vertex_buffers: vec![self.vertex_buffer.slice(..), self.texcoord_buffer.slice(..)],
            index_buffer: Some(self.index_buffer.slice(..)),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}

