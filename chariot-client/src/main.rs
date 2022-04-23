use chariot_core::GLOBAL_CONFIG;
use specs::{Builder, WorldExt};
use std::mem;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

mod application;
mod drawable;
mod game;
mod renderer;
mod resources;

fn main() {
    // at some point, networking PoC:
    // let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    // let game_client = game::GameClient::new(ip_addr);
    // game_client.ping();

    let event_loop = winit::event_loop::EventLoop::new();
    let context = renderer::context::Context::new(&event_loop);
    let renderer = renderer::Renderer::new(context);
    let mut application = application::Application::new(renderer);

    /*let material_handle = application.resources.import_material(
        &mut application.renderer,
        include_str!("shader.wgsl"),
        "boring",
    );*/

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => application.renderer.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                application.renderer.handle_surface_resize(size);
            }
            Event::RedrawRequested(_) => {
                application.update(); // Is this the right location?
                application.render();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    },
                ..
            } => println!("keyboard input!!"), // call application.on_keyboard_input()
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        modifiers,
                    },
                ..
            } => println!("mouse input!!"), // call application.on_mouse_input()
            Event::WindowEvent {
                event:
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        modifiers,
                    },
                ..
            } => println!("mouse moved!!"), // call application.on_mouse_moved()
            _ => {}
        }
    });
    /*
    let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    let mut game_client = game::GameClient::new(ip_addr);

    // temporary code until we establish an actual game loop
    game_client.ping();
    game_client.sync_outgoing();
    loop {
        game_client.sync_incoming();
        game_client.process_incoming_packets();
    }
    // end temporary code
    */
}
