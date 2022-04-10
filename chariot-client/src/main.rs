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

    let material_handle = application.resources.import_material(
        &mut application.renderer,
        include_str!("shader.wgsl"),
        "boring",
    );

    let import_result = application.resources.import_gltf(
        &application.renderer,
        "models/FlightHelmet/FlightHelmet.gltf",
    );

    if import_result.is_ok() {
        for static_mesh_handle in import_result.unwrap().2.iter() {
            let drawable = drawable::StaticMeshDrawable::new(
                &application.renderer,
                &application.resources,
                material_handle,
                *static_mesh_handle,
                0,
            );
            application.drawables.push(drawable);
        }
    }

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
}
