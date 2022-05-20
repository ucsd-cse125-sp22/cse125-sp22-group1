use std::collections::HashMap;

use crate::renderer::Renderer;

/*
 * A material encapsulates the render pass it should be a part of and the resources it should bind.
 */
#[derive(Default)]
#[allow(dead_code)] // Some instance vars of the material are currently unused, but may be in the future
pub struct Material {
    pub pass_name: String,
    pub bind_groups: HashMap<u32, wgpu::BindGroup>,

    pub buffers: Vec<wgpu::Buffer>,
    textures: Vec<wgpu::TextureView>,
    samplers: Vec<wgpu::Sampler>,
}

// Helper struct for building materials
enum MatResourceIdx {
    Buffer(usize),
    Texture(usize),
    Sampler(usize),
}
pub struct MaterialBuilder<'a> {
    pass_name: &'a str,
    renderer: &'a Renderer,
    bind_group_resources: HashMap<u32, HashMap<u32, MatResourceIdx>>,

    buffers: Vec<wgpu::Buffer>,
    textures: Vec<wgpu::TextureView>,
    samplers: Vec<wgpu::Sampler>,
}

impl<'a> MaterialBuilder<'a> {
    pub fn new(renderer: &'a Renderer, pass_name: &'a str) -> Self {
        MaterialBuilder {
            pass_name,
            renderer,
            bind_group_resources: HashMap::new(),
            buffers: Vec::new(),
            textures: Vec::new(),
            samplers: Vec::new(),
        }
    }

    pub fn buffer_resource(&mut self, group: u32, binding: u32, buffer: wgpu::Buffer) -> &mut Self {
        self.buffers.push(buffer);

        self.bind_group_resources
            .entry(group)
            .or_default()
            .insert(binding, MatResourceIdx::Buffer(self.buffers.len() - 1));
        self
    }

    pub fn texture_resource(
        &mut self,
        group: u32,
        binding: u32,
        texture: wgpu::TextureView,
    ) -> &mut Self {
        self.textures.push(texture);

        self.bind_group_resources
            .entry(group)
            .or_default()
            .insert(binding, MatResourceIdx::Texture(self.textures.len() - 1));
        self
    }

    pub fn sampler_resource(
        &mut self,
        group: u32,
        binding: u32,
        sampler: wgpu::Sampler,
    ) -> &mut Self {
        self.samplers.push(sampler);

        self.bind_group_resources
            .entry(group)
            .or_default()
            .insert(binding, MatResourceIdx::Sampler(self.samplers.len() - 1));
        self
    }

    pub fn produce(&mut self) -> Material {
        let lookup_binding_resource =
            |(binding, resource_idx): (&u32, &MatResourceIdx)| match resource_idx {
                MatResourceIdx::Buffer(idx) => (*binding, self.buffers[*idx].as_entire_binding()),
                MatResourceIdx::Texture(idx) => (
                    *binding,
                    wgpu::BindingResource::TextureView(&self.textures[*idx]),
                ),
                MatResourceIdx::Sampler(idx) => (
                    *binding,
                    wgpu::BindingResource::Sampler(&self.samplers[*idx]),
                ),
            };

        let create_bind_group = |(group, resource_map): (&u32, &HashMap<u32, MatResourceIdx>)| {
            let binding_resources = resource_map
                .iter()
                .map(lookup_binding_resource)
                .collect::<Vec<(u32, wgpu::BindingResource)>>();
            (
                *group,
                self.renderer
                    .create_bind_group(self.pass_name, *group, &binding_resources),
            )
        };

        let bind_groups = self
            .bind_group_resources
            .iter()
            .map(create_bind_group)
            .collect::<HashMap<u32, wgpu::BindGroup>>();

        Material {
            pass_name: String::from(self.pass_name),
            bind_groups: bind_groups,
            buffers: std::mem::take(&mut self.buffers),
            textures: std::mem::take(&mut self.textures),
            samplers: std::mem::take(&mut self.samplers),
        }
    }
}
