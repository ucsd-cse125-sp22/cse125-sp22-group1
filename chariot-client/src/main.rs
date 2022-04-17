use chariot_core::GLOBAL_CONFIG;
use game::GameClient;
use specs::{Builder, WorldExt};
use std::mem;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::ControlFlow,
};

mod application;
mod client_events;
mod drawable;
mod game;
mod renderer;
mod resources;

use crate::client_events::Watching;

fn main() {
    // at some point, networking PoC:
    // let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    // let game_client = game::GameClient::new(ip_addr);
    // game_client.ping();

    let ip_addr = "".to_string();
    let mut game = GameClient::new(ip_addr);

    let event_loop = winit::event_loop::EventLoop::new();
    let context = renderer::context::Context::new(&event_loop);
    let renderer = renderer::Renderer::new(context);
    let mut application = application::Application::new(renderer, game);

    let mut mouse_pos = PhysicalPosition::<f64> { x: -1.0, y: -1.0 };
    let mut first_run = false;

    // Example of main loop deferring to elsewhere
    event_loop.run(move |event, _, control_flow| {
        // TRIGGER EVENTS
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
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(key) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => application.on_key_down(key),
                        ElementState::Released => application.on_key_up(key),
                    }
                }
            } // call application.on_keyboard_input()
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        device_id,
                        button,
                        state,
                        modifiers,
                    },
                ..
            } => match button {
                MouseButton::Left => application.on_left_mouse(mouse_pos.x, mouse_pos.y, state),
                MouseButton::Right => application.on_right_mouse(mouse_pos.x, mouse_pos.y, state),
                _ => println!("Unknown mouse input received!"),
            }, // call application.on_mouse_input()
            Event::WindowEvent {
                event:
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        modifiers,
                    },
                ..
            } => {
                mouse_pos = position;
                application.on_mouse_move(position.x, position.y);
            } // call application.on_mouse_moved()
            _ => {}
        }

        // HANDLE GAME LOGIC

        // Trigger Pre-init
        // Call update()
        application.update();
        // Trigger Post-update
        // Call draw()
        // Trigger post-draw

        // Trigger Cleanup

        // NEXT ITERATION
    });

    /*let material_handle = application.resources.import_material(
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
    */
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
