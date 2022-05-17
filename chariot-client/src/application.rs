use std::collections::HashSet;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, VirtualKeyCode};

use chariot_core::networking::ClientBoundPacket;
use chariot_core::player_inputs::{EngineStatus, InputEvent, RotationStatus};
use chariot_core::GLOBAL_CONFIG;

use crate::game::{self, GameClient};
use crate::graphics::{register_passes, GraphicsManager};

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
        self.game.fetch_incoming_packets();

        // process current packets
        while let Some(packet) = self.game.connection.pop_incoming() {
            match packet {
                ClientBoundPacket::PlayerNumber(player_number) => {
                    self.graphics.add_player(player_number, true)
                }
                ClientBoundPacket::EntityUpdate(locations) => {
                    locations.iter().enumerate().for_each(|(i, update)| {
                        self.graphics
                            .update_player_location(&update.0, &update.1, i)
                    });
                }
                ClientBoundPacket::PlacementUpdate(position) => {
                    println!("I am now placed {}!", position);
                }
                ClientBoundPacket::LapUpdate(lap_num) => {
                    println!("I am now on lap {}!", lap_num);
                }
                ClientBoundPacket::GameStart(_) => println!("The game has begun!"),
                ClientBoundPacket::PowerupPickup => println!("we got a powerup!"),
                ClientBoundPacket::InteractionActivate(question, decision) => {
                    println!(
                        "The Audience has voted on {}, and voted for option {}!",
                        question.prompt, decision.label
                    );
                }
                ClientBoundPacket::AllDone => println!("This game is over!"),
                ClientBoundPacket::VotingStarted(question) => {
                    println!("The audience is now voting on {}", question.prompt)
                }
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
        self.pressed_keys.remove(&key);

        if let Some(event) = self.invert_event(self.get_input_event(key)) {
            self.game.send_input_event(event);
        };
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_pos.x = x;
        self.mouse_pos.y = y;
    }

    pub fn on_left_mouse(&mut self, state: ElementState) {
        let _x = self.mouse_pos.x;
        let _y = self.mouse_pos.y;

        if let ElementState::Released = state {
            // println!("Mouse clicked @ ({}, {})!", x, y);
        }
    }

    pub fn on_right_mouse(&mut self, state: ElementState) {
        let _x = self.mouse_pos.x;
        let _y = self.mouse_pos.y;

        if let ElementState::Released = state {
            // println!("Mouse right clicked @ ({}, {})!", x, y);
        }
    }

    pub fn _print_keys(&self) {
        println!("Pressed keys: {:?}", self.pressed_keys)
    }
}
