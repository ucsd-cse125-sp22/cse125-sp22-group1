/*
 * The Context struct contains the data for both the window and the wgpu context.
 * It mostly just makes it easier to initialize everything in one function.
 * TODO: Maybe in the future I'll add initial width and height paramters.
 */

use crate::assets::ui::ICON;
use chariot_core::GLOBAL_CONFIG;
use image::ImageFormat;
use std::io::Cursor;
use winit::window::{Fullscreen, Icon};
use Fullscreen::Borderless;

#[allow(dead_code)] // instance is just here to be kept alive
pub struct Context {
    pub(crate) window: winit::window::Window,
    pub(super) instance: wgpu::Instance,
    pub(super) surface: wgpu::Surface,
    pub(super) adapter: wgpu::Adapter,
    is_fullscreen: bool,
}

impl Context {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let window = winit::window::Window::new(&event_loop).unwrap();

        window.set_inner_size(size);
        window.set_resizable(false);

        // set title and icon
        window.set_title("Chairiot");
        let img = image::load(Cursor::new(ICON), ImageFormat::Png)
            .expect("couldn't load embedded icon")
            .into_rgba8();

        let width = img.width();
        let height = img.height();
        window.set_window_icon(Some(Icon::from_rgba(img.to_vec(), width, height).unwrap()));

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        let mut context = Context {
            window,
            instance,
            surface,
            adapter,
            is_fullscreen: false,
        };

        if GLOBAL_CONFIG.start_fullscreen {
            context.toggle_fullscreen();
        }

        context
    }

    // this does a lot for playability
    pub fn _capture_cursor(&self) {
        self.window.set_cursor_visible(false);
        let _ = self.window.set_cursor_grab(true);
    }

    pub fn toggle_fullscreen(&mut self) {
        self.is_fullscreen = !self.is_fullscreen;
        if self.is_fullscreen {
            self.window
                .set_fullscreen(Some(Borderless(self.window.current_monitor())));
        } else {
            self.window.set_fullscreen(None);
        }
    }
}
