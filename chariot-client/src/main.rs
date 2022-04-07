use std::mem;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow}
};
use chariot_core::GLOBAL_CONFIG;

mod game;
mod renderer;
mod drawable;
mod application;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Local {
    scale : f32
}

fn vec2(x : f32, y : f32) -> Vertex {
    Vertex{position : [x, y]}
}

// for anyone skimming this: vertex buffers are created & set up in code but not actually used by the shader (yet),
// index buffers are also created but binding is borked for now
// also, don't try building for wasm because I don't think that works yet either
fn main() {
    // at some point, networking PoC:
    // let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    // let game_client = game::GameClient::new(ip_addr);
    // game_client.ping();

    let event_loop = winit::event_loop::EventLoop::new();
    let context = renderer::Context::new(&event_loop);
    let renderer = renderer::Renderer::new(context);
	let mut application = application::Application::new(renderer);

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                application.renderer.handle_surface_resize(size);
            }
            Event::RedrawRequested(_) => {
                application.render();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
			Event::WindowEvent { 
				event: WindowEvent::KeyboardInput { 
					device_id, 
					input, 
					is_synthetic 
				},
				..
			} => println!("keyboard input!!"), // call application.on_keyboard_input()
			Event::WindowEvent { 
				event: WindowEvent::MouseInput { 
					device_id, 
					state, 
					button, 
					modifiers 
				},
				..
			} => println!("mouse input!!"), // call application.on_mouse_input()
			Event::WindowEvent { 
				event: WindowEvent::CursorMoved { 
					device_id, 
					position, 
					modifiers 
				},
				..
			} => println!("mouse moved!!"), // call application.on_mouse_moved()
            _ => {}
        }
    });
}
