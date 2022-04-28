use graphics::GraphicsManager;
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::ControlFlow,
};

mod application;
mod drawable;
mod game;
mod graphics;
mod renderer;
mod resources;
mod scenegraph;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let context = renderer::context::Context::new(&event_loop);
    let renderer = renderer::Renderer::new(context);

    let graphics_manager = GraphicsManager::new(renderer);
    let mut application = application::Application::new(graphics_manager);

    // Example of main loop deferring to elsewhere
    event_loop.run(move |event, _, control_flow| {
        // TRIGGER EVENTS
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => application.graphics.renderer.request_redraw(),
            // Window changes
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                application.graphics.renderer.handle_surface_resize(size);
            }

            // Forced redraw from OS
            Event::RedrawRequested(_) => {
                //application.update(); // Is this the right location?
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

        // Right now update isn't called at even intervals
        // (try moving the mouse around - the helmet spins faster because its getting more updates per frame)
        // This can be fixed by just calling update before render is called (see above) since that event
        // (RedrawRequested) seems to be getting a more even update interval
        application.update();
    });
}
