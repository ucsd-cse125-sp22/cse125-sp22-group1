use std::collections::HashSet;

use chariot_core::player::choices::Track;
use graphics::GraphicsManager;
use winit::{
    event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::graphics::register_passes;

mod application;
mod drawable;
mod game;
mod graphics;
mod renderer;
mod resources;
mod scenegraph;
mod ui;
mod ui_state;
mod util;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let context = renderer::context::Context::new(
        &event_loop,
        winit::dpi::PhysicalSize::<u32>::new(1280, 720),
    );

    let renderer = renderer::Renderer::new(context);

    let dev_mode = true; //std::env::args().find(|a| a == "d").is_some();
    let mut graphics_manager = GraphicsManager::new(renderer);

    // Example of main loop deferring to elsewhere
    if dev_mode {
        println!("Running in dev mode...");
        let mut pressed_keys = HashSet::new();
        graphics_manager.load_dev_mode(Track::Track);
        event_loop.run(move |event, _, control_flow| {
            // TRIGGER EVENTS
            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared => graphics_manager.renderer.request_redraw(),
                // Window changes
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    graphics_manager.renderer.handle_surface_resize(size);
                }

                // Forced redraw from OS
                Event::RedrawRequested(_) => {
                    //application.update(); // Is this the right location?
                    graphics_manager.render();
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
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit;
                    }

                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            ElementState::Pressed => pressed_keys.insert(key),
                            ElementState::Released => pressed_keys.remove(&key),
                        };

                        if key == winit::event::VirtualKeyCode::R
                            && input.state == ElementState::Pressed
                        {
                            register_passes(&mut graphics_manager.renderer);
                        }
                    }
                }

                // Mouse moved
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    graphics_manager.update_flycam_angle(position.x, position.y);
                }

                // If there's an event to detect loss/gain of focus, we will need to clear our pressed keys just in case
                // Other
                _ => {}
            }

            let forward_sign = if pressed_keys.contains(&VirtualKeyCode::W) {
                1.0
            } else if pressed_keys.contains(&VirtualKeyCode::S) {
                -1.0
            } else {
                0.0
            };

            let right_sign = if pressed_keys.contains(&VirtualKeyCode::D) {
                1.0
            } else if pressed_keys.contains(&VirtualKeyCode::A) {
                -1.0
            } else {
                0.0
            };

            let cam_dir = forward_sign * glam::Vec3::Z + right_sign * glam::Vec3::X;
            graphics_manager.update_flycam_pos(cam_dir);
            graphics_manager.update(0.01);

            graphics_manager.add_fire_to_player(0, 0.01);
        });
    } else {
        let mut application = application::Application::new(graphics_manager);

        let mut gamepad_manager = gilrs::Gilrs::new().unwrap();
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
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit;
                    }

                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            ElementState::Pressed => application.on_key_down(key),
                            ElementState::Released => application.on_key_up(key),
                        }
                    }
                }

                // Mouse input
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { button, state, .. },
                    ..
                } => match button {
                    MouseButton::Left => application.on_left_mouse(state),
                    MouseButton::Right => application.on_right_mouse(state),
                    _ => (),
                },

                // Mouse moved
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    application.on_mouse_move(position.x, position.y);
                }

                // If there's an event to detect loss/gain of focus, we will need to clear our pressed keys just in case
                // Other
                _ => {}
            }

            while let Some(event) = gamepad_manager.next_event() {
                application.handle_gamepad_event(event);
            }

            // Right now update isn't called at even intervals
            // (try moving the mouse around - the helmet spins faster because its getting more updates per frame)
            // This can be fixed by just calling update before render is called (see above) since that event
            // (RedrawRequested) seems to be getting a more even update interval
            application.update();

            // Tell the gamepad manager we have finished an update tick
            gamepad_manager.inc();
        });
    }
}
