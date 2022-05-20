pub struct PostProcessTechnique {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: material::Material,
}

impl ShadeDirectTechnique {
    const PASS_NAME: &'static str = "postprocess";
    const FRAMEBUFFER_NAME: &'static str = "surface";
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

        let material = material::MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "shade_direct", 0, false)
            .produce();

        Self {
            vertex_buffer,
            index_buffer,
            material,
        }
    }
}

impl Technique for ShadeDirectTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let bind_groups = self.material.bind_groups(context.iteration);

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: 6,
            vertex_buffers: vec![self.vertex_buffer.slice(..)],
            index_buffer: Some(self.index_buffer.slice(..)),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}
