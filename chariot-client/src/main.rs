use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::ControlFlow,
};

use game::GameClient;

use chariot_core::GLOBAL_CONFIG;

mod application;
mod drawable;
mod game;
mod renderer;
mod resources;

fn main() {
    let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
    let mut game_client = game::GameClient::new(ip_addr);

    let event_loop = winit::event_loop::EventLoop::new();
    let context = renderer::context::Context::new(&event_loop);
    let renderer = renderer::Renderer::new(context);
    let mut application = application::Application::new(renderer, game_client);

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
                MouseButton::Left => application.on_left_mouse(state),
                MouseButton::Right => application.on_right_mouse(state),
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
                application.on_mouse_move(position.x, position.y);
            } // call application.on_mouse_moved()
            _ => {} // If there's an event to detect loss/gain of focus, we will need to clear our pressed keys just in case
        }

        application.update();
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
