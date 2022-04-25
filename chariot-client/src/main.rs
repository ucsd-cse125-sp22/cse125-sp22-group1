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
            // Window changes
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                application.renderer.handle_surface_resize(size);
            }

            // Forced redraw from OS
            Event::RedrawRequested(_) => {
                application.render();
            }

            // X button on window clicked
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            // Keyboard input
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
            }

            // Mouse input
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
                _ => (),
            },

            // Mouse moved
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
            }

            // If there's an event to detect loss/gain of focus, we will need to clear our pressed keys just in case
            // Other
            _ => {}
        }

        application.update();
    });
}
