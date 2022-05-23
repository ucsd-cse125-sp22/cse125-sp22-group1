use std::collections::HashSet;
use std::time::Instant;

use chariot_core::player::choices::{Chair, Track};

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, VirtualKeyCode};

use chariot_core::networking::ClientBoundPacket;
use chariot_core::player::player_inputs::{EngineStatus, InputEvent, RotationStatus};
use chariot_core::GLOBAL_CONFIG;

use crate::game::GameClient;
use crate::graphics::{register_passes, GraphicsManager};

use crate::ui::ui_region::UIRegion;
use crate::ui_state::AnnouncementState;

pub struct Application {
    pub graphics: GraphicsManager,
    pub game: GameClient,
    pub pressed_keys: HashSet<VirtualKeyCode>,
    mouse_pos: PhysicalPosition<f64>,
    ui_regions: Vec<UIRegion>,
}

impl Application {
    pub fn new(mut graphics_manager: GraphicsManager) -> Self {
        let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
        let game = GameClient::new(ip_addr);
        let ui_regions = graphics_manager.load_menu();

        Self {
            graphics: graphics_manager,
            game,
            pressed_keys: HashSet::new(),
            mouse_pos: PhysicalPosition::<f64> { x: -1.0, y: -1.0 },
            ui_regions,
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
                ClientBoundPacket::PlayerNumber(player_number, others_choices) => {
                    self.graphics.player_num = player_number;
                    println!("I am now player #{}!", player_number);
                    self.graphics.player_choices = others_choices;
                    self.graphics.player_choices[player_number] = Some(Default::default());
                    self.graphics.load_pregame();
                }
                ClientBoundPacket::PlayerJoined(player_number) => {
                    self.graphics.player_choices[player_number] = Some(Default::default());
                }

                ClientBoundPacket::PlayerChairChoice(player_num, chair) => {
                    println!("Player #{} has chosen chair {}!", player_num, chair.clone());
                    self.graphics.player_choices[player_num]
                        .as_mut()
                        .expect("Attempted to set chair on player we don't know about!")
                        .chair = chair;
                }
                ClientBoundPacket::PlayerMapChoice(player_num, map) => {
                    println!("Player #{} has voted for map {}!", player_num, map.clone());
                    self.graphics.player_choices[player_num]
                        .as_mut()
                        .expect("Attempted to set chair on player we don't know about!")
                        .map = map;
                }
                ClientBoundPacket::PlayerReadyStatus(player_num, status) => {
                    println!(
                        "Player #{} is no{} ready!",
                        player_num,
                        if status { "w" } else { "t" }
                    );
                }

                ClientBoundPacket::LoadGame(map) => {
                    println!("Loading map {}!", map);
                    self.graphics.load_map(map);
                    self.game.signal_loaded();
                }

                ClientBoundPacket::EntityUpdate(locations) => {
                    locations.iter().enumerate().for_each(|(i, update)| {
                        self.graphics
                            .update_player_location(&update.0, &update.1, i)
                    });
                }
                ClientBoundPacket::PlacementUpdate(position) => {
                    self.graphics.maybe_update_place(position);
                }
                ClientBoundPacket::LapUpdate(lap_num) => {
                    println!("I am now on lap {}!", lap_num);
                }
                ClientBoundPacket::GameStart(_) => {
                    self.graphics.display_hud();
                    println!("The game has begun!")
                }
                ClientBoundPacket::PowerupPickup => println!("we got a powerup!"),
                ClientBoundPacket::VotingStarted {
                    question,
                    time_until_vote_end,
                } => {
                    let vote_end_time = Instant::now() + time_until_vote_end;
                    self.graphics.make_announcement(
                        "The audience is deciding your fate",
                        format!(
                            "They decide in {} seconds",
                            (vote_end_time - Instant::now()).as_secs()
                        )
                        .as_str(),
                    );

                    self.graphics.maybe_set_announcement_state(
                        AnnouncementState::VotingInProgress {
                            prompt: question.prompt,
                            vote_end_time,
                        },
                    );
                }
                ClientBoundPacket::InteractionActivate {
                    question,
                    decision,
                    time_effect_is_live,
                } => {
                    let effect_end_time = Instant::now() + time_effect_is_live;
                    self.graphics.make_announcement(
                        format!("{} was chosen!", decision.label).as_str(),
                        format!(
                            "Effects will last for another {} seconds",
                            (effect_end_time - Instant::now()).as_secs()
                        )
                        .as_str(),
                    );

                    self.graphics
                        .maybe_set_announcement_state(AnnouncementState::VoteActiveTime {
                            prompt: question.prompt,
                            decision: decision.label,
                            effect_end_time,
                        });
                }
                ClientBoundPacket::AllDone(final_placements) => {
                    println!(
                        "This game is over! Results:\n{}",
                        final_placements
                            .iter()
                            .enumerate()
                            .map(|(player_num, place)| format!(
                                "\t(#{} came {})\n",
                                player_num, place
                            ))
                            .collect::<String>()
                    );
                }
                ClientBoundPacket::StartNextGame => {
                    self.graphics.load_pregame();
                }
            }
        }
        self.graphics.update_minimap();
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
            self.graphics.set_loading_text("Reloading shaders");
            register_passes(&mut self.graphics.renderer);
        } else if key == VirtualKeyCode::Return {
            self.graphics.set_loading_text("Picking chair");
            self.game.pick_chair(Chair::Standard);
        } else if key == VirtualKeyCode::Apostrophe {
            self.graphics.set_loading_text("Picking map");
            self.game.pick_map(Track::Track);
        } else if key == VirtualKeyCode::Semicolon {
            self.graphics.set_loading_text("Setting ready");
            self.game.signal_ready_status(true);
        } else if key == VirtualKeyCode::L {
            self.graphics.set_loading_text("Forcing a start!");
            self.game.force_start();
        } else if key == VirtualKeyCode::P {
            self.graphics.set_loading_text("Starting next game!");
            self.game.next_game();
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

        self.ui_regions
            .iter_mut()
            .for_each(|reg| reg.set_hovering(x, y, &mut self.graphics));
    }

    pub fn on_left_mouse(&mut self, state: ElementState) {
        let x = self.mouse_pos.x;
        let y = self.mouse_pos.y;

        match state {
            ElementState::Pressed => self
                .ui_regions
                .iter_mut()
                .for_each(|reg| reg.set_active(x, y, &mut self.graphics)),
            ElementState::Released => self
                .ui_regions
                .iter_mut()
                .for_each(|reg| reg.set_inactive(&mut self.graphics)),
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
