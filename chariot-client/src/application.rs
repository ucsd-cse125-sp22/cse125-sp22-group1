use chariot_core::networking::ClientBoundPacket;
use chariot_core::GLOBAL_CONFIG;
use std::collections::HashSet;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, VirtualKeyCode};

use crate::game::{self, GameClient};
use crate::graphics::{register_passes, GraphicsManager};

use chariot_core::player_inputs::{EngineStatus, InputEvent, RotationStatus};

pub struct Application {
    pub graphics: GraphicsManager,
    pub game: GameClient,
    pub pressed_keys: HashSet<VirtualKeyCode>,
    mouse_pos: PhysicalPosition<f64>,
}

impl Application {
    pub fn new(graphics_manager: GraphicsManager) -> Self {
        let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
        let mut game = game::GameClient::new(ip_addr);

        game.send_ready_packet("standard".to_string());

        Self {
            graphics: graphics_manager,
            game,
            pressed_keys: HashSet::new(),
            mouse_pos: PhysicalPosition::<f64> { x: -1.0, y: -1.0 },
        }
    }

    pub fn render(&mut self) {
        self.graphics.render();
    }

    pub fn update(&mut self) {
        let mouse_pos = glam::Vec2::new(self.mouse_pos.x as f32, self.mouse_pos.y as f32);
        self.graphics.update(mouse_pos);

        self.game.fetch_incoming_packets();

        for packet in self.game.current_packets() {
            match packet {
                ClientBoundPacket::PlayerNumber(player_number) => {
                    self.graphics.add_player(player_number, true)
                }
                ClientBoundPacket::LocationUpdate(locations) => {
                    for (i, location) in locations.iter().enumerate() {
                        if location.is_some() {
                            self.graphics
                                .update_player_location(&location.unwrap(), i as u8);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Input configuration
    fn get_input_event(&self, key: VirtualKeyCode) -> Option<InputEvent> {
        match key {
            // Forwards
            VirtualKeyCode::W => Some(InputEvent::Engine(EngineStatus::Accelerating)),
            // Backwards
            VirtualKeyCode::S => Some(InputEvent::Engine(EngineStatus::Braking)),
            // Left
            VirtualKeyCode::A => Some(InputEvent::Rotation(RotationStatus::InSpinCounterclockwise)),
            // Right
            VirtualKeyCode::D => Some(InputEvent::Rotation(RotationStatus::InSpinClockwise)),
            // Right
            _ => None,
        }
    }

    fn invert_event(&self, event: Option<InputEvent>) -> Option<InputEvent> {
        Some(match event {
            Some(InputEvent::Engine(EngineStatus::Accelerating)) => {
                InputEvent::Engine(EngineStatus::Neutral)
            }
            Some(InputEvent::Engine(EngineStatus::Braking)) => {
                InputEvent::Engine(EngineStatus::Neutral)
            }
            Some(InputEvent::Rotation(RotationStatus::InSpinClockwise)) => {
                InputEvent::Rotation(RotationStatus::NotInSpin)
            }
            Some(InputEvent::Rotation(RotationStatus::InSpinCounterclockwise)) => {
                InputEvent::Rotation(RotationStatus::NotInSpin)
            }
            _ => return None,
        })
    }

    // Input Handlers
    pub fn on_key_down(&mut self, key: VirtualKeyCode) {
        // winit sends duplicate keydown events, so we will just make sure we don't already have this processed
        if self.pressed_keys.contains(&key) {
            return;
        };

        println!("Key down [{:?}]!", key);
        self.pressed_keys.insert(key);

        if let Some(event) = self.get_input_event(key) {
            self.game.send_input_event(event);
        };

        if key == VirtualKeyCode::R {
            println!("Reloading shaders");
            register_passes(&mut self.graphics.renderer);
        }
    }

    pub fn on_key_up(&mut self, key: VirtualKeyCode) {
        println!("Key up [{:?}]!", key);
        self.pressed_keys.remove(&key);

        if let Some(event) = self.invert_event(self.get_input_event(key)) {
            self.game.send_input_event(event);
        };
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_pos.x = x;
        self.mouse_pos.y = y;
        //println!("Mouse moved! ({}, {})", x, y);
    }

    pub fn on_left_mouse(&mut self, state: ElementState) {
        let x = self.mouse_pos.x;
        let y = self.mouse_pos.y;

        if let ElementState::Released = state {
            println!("Mouse clicked @ ({}, {})!", x, y);
        }
    }

    pub fn on_right_mouse(&mut self, state: ElementState) {
        let x = self.mouse_pos.x;
        let y = self.mouse_pos.y;

        if let ElementState::Released = state {
            println!("Mouse right clicked @ ({}, {})!", x, y);
        }
    }

    pub fn print_keys(&self) {
        println!("Pressed keys: {:?}", self.pressed_keys)
    }
}
