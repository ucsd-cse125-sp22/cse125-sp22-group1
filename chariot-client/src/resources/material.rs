use std::collections::HashMap;

use crate::renderer::Renderer;
use crate::resources::ResourceManager;

/*
 * A material encapsulates the render pass it should be a part of and the resources it should bind.
 */
#[derive(Default)]
#[allow(dead_code)] // Some instance vars of the material are currently unused, but may be in the future
pub struct Material {
    pub pass_name: String,
    bind_groups: HashMap<u32, wgpu::BindGroup>,
    alt_bind_groups: HashMap<u32, wgpu::BindGroup>,

    pub buffers: Vec<wgpu::Buffer>,
    textures: Vec<wgpu::TextureView>,
    samplers: Vec<wgpu::Sampler>,
}

impl Material {
    pub fn bind_groups(&self, iteration: u32) -> Vec<&wgpu::BindGroup> {
        let use_alt = iteration % 2 == 1;
        self.bind_groups
            .iter()
            .map(|(group, bind_group)| {
                if self.alt_bind_groups.contains_key(group) && use_alt {
                    self.alt_bind_groups.get(group).unwrap()
                } else {
                    bind_group
                }
            })
            .collect()
    }
}

// Helper struct for building materials
enum MatResourceIdx {
    Buffer(usize),
    Texture(usize),
    FbTexture(usize, usize),
    Sampler(usize),
}

impl MatResourceIdx {
    fn is_fb_texture_with_alt(&self, _: &ResourceManager) -> bool {
        if let MatResourceIdx::FbTexture(idx, alt_idx) = self {
            idx != alt_idx
        } else {
            false
        }
    }
}
pub struct MaterialBuilder<'a> {
    pass_name: &'a str,
    renderer: &'a Renderer,
    resources: Option<&'a ResourceManager>,
    bind_group_resources: HashMap<u32, HashMap<u32, MatResourceIdx>>,

    buffers: Vec<wgpu::Buffer>,
    textures: Vec<wgpu::TextureView>,
    samplers: Vec<wgpu::Sampler>,
}

impl<'a> MaterialBuilder<'a> {
    pub fn new(renderer: &'a Renderer, resources: &'a ResourceManager, pass_name: &'a str) -> Self {
        MaterialBuilder {
            pass_name,
            renderer,
            resources: Some(resources),
            bind_group_resources: HashMap::new(),
            buffers: Vec::new(),
            textures: Vec::new(),
            samplers: Vec::new(),
        }
    }

    pub fn new_no_fb(renderer: &'a Renderer, pass_name: &'a str) -> Self {
        MaterialBuilder {
            pass_name,
            renderer,
            resources: None,
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

    pub fn framebuffer_texture_resource(
        &mut self,
        group: u32,
        binding: u32,
        fb_name: &'a str,
        idx: usize,
        prev: bool,
    ) -> &mut Self {
        let fb_tex_view = self
            .resources
            .expect("Must create builder with resource manager to access framebuffer textures")
            .framebuffer_tex(fb_name, idx, false)
            .expect(format!("Invalid framebuffer tex: ({}, {})", fb_name, idx).as_str())
            .create_view(&wgpu::TextureViewDescriptor::default());

        let tex_id = self.textures.len();
        self.textures.push(fb_tex_view);

        let (res_id, alt_res_id) = if let Some(alt_fb_tex) =
            self.resources.unwrap().framebuffer_tex(fb_name, idx, true)
        {
            let alt_fb_tex_view = alt_fb_tex.create_view(&wgpu::TextureViewDescriptor::default());
            let alt_tex_id = self.textures.len();
            self.textures.push(alt_fb_tex_view);

            if !prev {
                (tex_id, alt_tex_id)
            } else {
                (alt_tex_id, tex_id)
            }
        } else {
            (tex_id, tex_id)
        };

        self.bind_group_resources
            .entry(group)
            .or_default()
            .insert(binding, MatResourceIdx::FbTexture(res_id, alt_res_id));
        self
    }

    pub fn sampler_resource<'b>(
        &'b mut self,
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
        let lookup_binding_resource_with_alt =
            |(binding, resource_idx, alt): (&u32, &MatResourceIdx, bool)| match resource_idx {
                MatResourceIdx::Buffer(idx) => (*binding, self.buffers[*idx].as_entire_binding()),
                MatResourceIdx::Texture(idx) => (
                    *binding,
                    wgpu::BindingResource::TextureView(&self.textures[*idx]),
                ),
                MatResourceIdx::Sampler(idx) => (
                    *binding,
                    wgpu::BindingResource::Sampler(&self.samplers[*idx]),
                ),
                MatResourceIdx::FbTexture(idx, alt_idx) => (
                    *binding,
                    wgpu::BindingResource::TextureView(if !alt {
                        &self.textures[*idx]
                    } else {
                        &self.textures[*alt_idx]
                    }),
                ),
            };

        let lookup_binding_resource = |(binding, resource_idx): (&u32, &MatResourceIdx)| {
            lookup_binding_resource_with_alt((binding, resource_idx, false))
        };

        let lookup_alt_binding_resource = |(binding, resource_idx): (&u32, &MatResourceIdx)| {
            lookup_binding_resource_with_alt((binding, resource_idx, true))
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

        let create_alt_bind_group =
            |(group, resource_map): (&u32, &HashMap<u32, MatResourceIdx>)| {
                let binding_resources = resource_map
                    .iter()
                    .map(lookup_alt_binding_resource)
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

        let alt_bind_groups = self
            .bind_group_resources
            .iter()
            .filter(|(_, resource_map)| {
                resource_map.iter().any(|(_, resource_idx)| {
                    self.resources.is_some()
                        && resource_idx.is_fb_texture_with_alt(self.resources.unwrap())
                })
            })
            .map(create_alt_bind_group)
            .collect::<HashMap<u32, wgpu::BindGroup>>();

        Material {
            pass_name: String::from(self.pass_name),
            bind_groups: bind_groups,
            alt_bind_groups: alt_bind_groups,
            buffers: std::mem::take(&mut self.buffers),
            textures: std::mem::take(&mut self.textures),
            samplers: std::mem::take(&mut self.samplers),
        }
    }
}
