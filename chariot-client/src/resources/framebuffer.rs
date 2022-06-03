use crate::renderer::{FramebufferDescriptor, Renderer};

use super::{Handle, ResourceManager, TextureHandle};

impl ResourceManager {
    fn create_framebuffer_textures(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        storage: bool,
        is_alt: bool,
    ) -> Vec<TextureHandle> {
        let usages = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | if storage {
                wgpu::TextureUsages::STORAGE_BINDING
            } else {
                wgpu::TextureUsages::empty()
            };

        let color_textures: Vec<wgpu::Texture> = formats
            .iter()
            .enumerate()
            .map(|(idx, format)| {
                renderer.create_texture2d(
                    format!("{}_tex_{}", name, idx).as_str(),
                    size,
                    *format,
                    usages,
                )
            })
            .collect();

        let color_handles: Vec<TextureHandle> = (0..color_textures.len())
            .map(|_idx| TextureHandle::unique())
            .collect();

        let depth_handle = TextureHandle::unique();

        let desc = FramebufferDescriptor {
            color_attachments: color_textures
                .iter()
                .map(|tex| tex.create_view(&wgpu::TextureViewDescriptor::default()))
                .collect(),
            depth_stencil_attachment: None,
            clear_color: clear_color,
            clear_depth: false,
        };

        let register_name = format!("{}.{}", name, if is_alt { 1 } else { 0 });
        renderer.register_framebuffer(register_name.as_str(), desc);

        self.textures.extend(
            color_handles
                .clone()
                .into_iter()
                .chain([depth_handle].into_iter())
                .zip(color_textures.into_iter()),
        );

        color_handles
    }

    fn create_depth_framebuffer_textures(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        clear_depth: bool,
        storage: bool,
        is_alt: bool,
    ) -> (TextureHandle, Vec<TextureHandle>) {
        let depth_texture = renderer.create_texture2d(
            format!("{}_tex_depth", name).as_str(),
            size,
            Renderer::DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let usages = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | if storage {
                wgpu::TextureUsages::STORAGE_BINDING
            } else {
                wgpu::TextureUsages::empty()
            };

        let color_textures: Vec<wgpu::Texture> = formats
            .iter()
            .enumerate()
            .map(|(idx, format)| {
                renderer.create_texture2d(
                    format!("{}_tex_{}", name, idx).as_str(),
                    size,
                    *format,
                    usages,
                )
            })
            .collect();

        let color_handles: Vec<TextureHandle> = (0..color_textures.len())
            .map(|_idx| TextureHandle::unique())
            .collect();

        let depth_handle = TextureHandle::unique();

        let desc = FramebufferDescriptor {
            color_attachments: color_textures
                .iter()
                .map(|tex| tex.create_view(&wgpu::TextureViewDescriptor::default()))
                .collect(),
            depth_stencil_attachment: Some(
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            ),
            clear_color: clear_color,
            clear_depth: clear_depth,
        };

        let register_name = format!("{}.{}", name, if is_alt { 1 } else { 0 });
        renderer.register_framebuffer(register_name.as_str(), desc);

        self.textures.extend(
            color_handles
                .clone()
                .into_iter()
                .chain([depth_handle].into_iter())
                .zip(
                    color_textures
                        .into_iter()
                        .chain([depth_texture].into_iter()),
                ),
        );

        (depth_handle, color_handles)
    }

    pub fn register_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        storage: bool,
        save_prev: bool,
    ) {
        let color_handles = self.create_framebuffer_textures(
            name,
            renderer,
            size,
            formats,
            clear_color,
            storage,
            false,
        );

        self.framebuffers
            .entry(name.to_string())
            .or_default()
            .extend(color_handles.iter());

        if save_prev {
            let alt_color_handles = self.create_framebuffer_textures(
                name,
                renderer,
                size,
                formats,
                clear_color,
                storage,
                true,
            );

            self.alt_framebuffers
                .entry(name.to_string())
                .or_default()
                .extend(alt_color_handles.iter());
        }
    }

    pub fn register_depth_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        clear_depth: bool,
        storage: bool,
        save_prev: bool,
    ) {
        let (depth_handle, color_handles) = self.create_depth_framebuffer_textures(
            name,
            renderer,
            size,
            formats,
            clear_color,
            clear_depth,
            storage,
            false,
        );

        self.framebuffers
            .entry(name.to_string())
            .or_default()
            .extend(color_handles.iter().chain([depth_handle].iter()));

        if save_prev {
            let (alt_depth_handle, alt_color_handles) = self.create_depth_framebuffer_textures(
                name,
                renderer,
                size,
                formats,
                clear_color,
                clear_depth,
                storage,
                true,
            );

            self.alt_framebuffers
                .entry(name.to_string())
                .or_default()
                .extend(alt_color_handles.iter().chain([alt_depth_handle].iter()));
        }
    }

    pub fn register_surface_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        storage: bool,
        save_prev: bool,
    ) {
        let surface_size = renderer.surface_size();
        self.register_framebuffer(
            name,
            renderer,
            surface_size,
            formats,
            clear_color,
            storage,
            save_prev,
        )
    }

    pub fn register_depth_surface_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        clear_depth: bool,
        storage: bool,
        save_prev: bool,
    ) {
        let surface_size = renderer.surface_size();
        self.register_depth_framebuffer(
            name,
            renderer,
            surface_size,
            formats,
            clear_color,
            clear_depth,
            storage,
            save_prev,
        )
    }

    pub fn framebuffer_tex(&self, name: &str, index: usize, alt: bool) -> Option<&wgpu::Texture> {
        let handle = if !alt {
            self.framebuffers.get(&name.to_string())?.get(index)?
        } else {
            self.alt_framebuffers.get(&name.to_string())?.get(index)?
        };
        self.textures.get(&handle)
    }

    pub fn framebuffer_name(&self, name: &str, alt: bool) -> String {
        let get_alt = alt && self.alt_framebuffers.contains_key(&name.to_string());
        format!("{}.{}", name, if get_alt { 1 } else { 0 })
    }
}
