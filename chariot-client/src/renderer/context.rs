/*
 * The Context struct contains the data for both the window and the wgpu context.
 * It mostly just makes it easier to initialize everything in one function.
 * TODO: Maybe in the future I'll add initial width and height paramters.
 */

#[allow(dead_code)] // instance is just here to be kept alive
pub struct Context {
    pub(super) window: winit::window::Window,
    pub(super) instance: wgpu::Instance,
    pub(super) surface: wgpu::Surface,
    pub(super) adapter: wgpu::Adapter,
}

impl Context {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let window = winit::window::Window::new(&event_loop).unwrap();
        //window.set_inner_size(size);
        window.set_resizable(false);

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
