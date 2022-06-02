use chariot_core::sound_effect::SoundEffect;
use gilrs::{Axis, Button, EventType};
use std::collections::HashSet;
use std::time::{Duration, Instant};

use std::time::SystemTime;

use chariot_core::player::choices::{Chair, Track};

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, VirtualKeyCode};

use crate::assets::audio::{get_sfx, CYBER_RECLINER, HOLD_ON_TO_YOUR_SEATS};
use crate::audio::AudioManager;
use chariot_core::networking::ClientBoundPacket;
use chariot_core::player::player_inputs::{EngineStatus, InputEvent, RotationStatus};
use chariot_core::GLOBAL_CONFIG;

use crate::game::GameClient;
use crate::graphics::{register_passes, GraphicsManager};

use crate::audio::thread::context::AudioCtx;
use crate::audio::thread::options::SourceOptions;
use crate::ui::ui_region::UIRegion;
use crate::ui_state::AnnouncementState;

pub struct Application {
    // audio
    pub audio_context: AudioCtx,
    pub music_manager: AudioManager,
    pub sfx_manager: AudioManager,

    // everything else haha
    pub graphics: GraphicsManager,
    pub game: GameClient,
    pub pressed_keys: HashSet<VirtualKeyCode>,
    mouse_pos: PhysicalPosition<f64>,
    last_update: SystemTime,
    game_start_time: SystemTime,
    ui_regions: Vec<UIRegion>,
}

impl Application {
    pub fn new(mut graphics_manager: GraphicsManager) -> Self {
        let ip_addr = format!("{}:{}", GLOBAL_CONFIG.server_address, GLOBAL_CONFIG.port);
        let game = GameClient::new(ip_addr);
        graphics_manager.display_main_menu();

        // create audio resources and play title track
        let audio_context = AudioCtx::new();
        let mut music_manager = AudioManager::new();
        music_manager.play(
            CYBER_RECLINER,
            &audio_context,
            SourceOptions::new().set_repeat(true),
        );
        let sfx_manager = AudioManager::new();

        Self {
            audio_context,
            music_manager,
            sfx_manager,
            graphics: graphics_manager,
            game,
            pressed_keys: HashSet::new(),
            mouse_pos: PhysicalPosition::<f64> { x: -1.0, y: -1.0 },
            game_start_time: SystemTime::now(),
            ui_regions: vec![],
            last_update: SystemTime::now(),
        }
    }

    pub fn render(&mut self) {
        self.graphics.render();
        let ui_regions = self.graphics.get_ui_regions();
        if ui_regions.len() > 0 {
            self.ui_regions = ui_regions;
        }
    }

    pub fn update(&mut self) {
        let delta_time = self.last_update.elapsed().unwrap().as_secs_f32();
        self.graphics.update(delta_time);

        if let Ok(since_game_started) = self.game_start_time.elapsed() {
            self.graphics.update_timer(since_game_started);
        }

        // TODO: do this for other players
        if self.pressed_keys.contains(&VirtualKeyCode::W) {
            self.graphics.add_fire_to_player(0, delta_time);
        }

        self.last_update = SystemTime::now();

        self.game.fetch_incoming_packets();

        // process current packets
        while let Some(packet) = self.game.connection.pop_incoming() {
            match packet {
                ClientBoundPacket::PlayerNumber(player_number, others_choices) => {
                    self.graphics.player_num = player_number;
                    self.graphics.player_choices = others_choices;
                    self.graphics.player_choices[player_number] = Some(Default::default());
                    self.graphics.load_pregame();
                }
                ClientBoundPacket::PlayerJoined(player_number) => {
                    if player_number != self.graphics.player_num {
                        self.graphics.player_choices[player_number] = Some(Default::default());
                    }
                }

                ClientBoundPacket::PlayerChairChoice(player_num, chair) => {
                    self.graphics.player_choices[player_num]
                        .as_mut()
                        .expect("Attempted to set chair on player we don't know about!")
                        .chair = chair;
                    self.graphics.maybe_display_chair(chair, player_num);
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
                    self.music_manager.play_cf(
                        HOLD_ON_TO_YOUR_SEATS,
                        &self.audio_context,
                        SourceOptions::new().set_repeat(true),
                        Duration::new(2, 0),
                    );
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
                    self.sfx_manager.play(
                        get_sfx(SoundEffect::NextLap),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
                }
                ClientBoundPacket::GameStart(duration) => {
                    self.graphics.display_hud();
                    self.sfx_manager.play(
                        get_sfx(SoundEffect::GameStart),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
                    self.game_start_time = SystemTime::now() + duration;
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

                    self.sfx_manager.play(
                        get_sfx(SoundEffect::InteractionVoteStart),
                        &self.audio_context,
                        SourceOptions::new(),
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

                    self.sfx_manager.play(
                        get_sfx(SoundEffect::InteractionChosen),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
                }
                ClientBoundPacket::SoundEffectEvent(effect) => {
                    self.sfx_manager.play(
                        get_sfx(effect),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
                }
                ClientBoundPacket::AllDone(final_placements) => {
                    self.sfx_manager.play(
                        get_sfx(SoundEffect::GameEnd),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
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
    }

    // Input configuration
    fn get_input_event(&self, key: VirtualKeyCode) -> Option<InputEvent> {
        match key {
            // Forwards
            VirtualKeyCode::W => Some(InputEvent::Engine(EngineStatus::Accelerating(1.0))),
            // Backwards
            VirtualKeyCode::S => Some(InputEvent::Engine(EngineStatus::Braking)),
            // Left
            VirtualKeyCode::A => Some(InputEvent::Rotation(
                RotationStatus::InSpinCounterclockwise(1.0),
            )),
            // Right
            VirtualKeyCode::D => Some(InputEvent::Rotation(RotationStatus::InSpinClockwise(1.0))),
            // Right
            _ => None,
        }
    }

    fn invert_event(&self, event: Option<InputEvent>) -> Option<InputEvent> {
        Some(match event {
            Some(InputEvent::Engine(EngineStatus::Accelerating(_))) => {
                InputEvent::Engine(EngineStatus::Neutral)
            }
            Some(InputEvent::Engine(EngineStatus::Braking)) => {
                InputEvent::Engine(EngineStatus::Neutral)
            }
            Some(InputEvent::Rotation(RotationStatus::InSpinClockwise(_))) => {
                InputEvent::Rotation(RotationStatus::NotInSpin)
            }
            Some(InputEvent::Rotation(RotationStatus::InSpinCounterclockwise(_))) => {
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
        } else if key == VirtualKeyCode::Apostrophe {
            println!("Picking map");
            self.game.pick_map(Track::Track);
        } else if key == VirtualKeyCode::Semicolon {
            println!("Setting ready");
            self.game.signal_ready_status(true);
        } else if key == VirtualKeyCode::L {
            println!("Forcing a start!");
            self.game.force_start();
        } else if key == VirtualKeyCode::P {
            println!("Starting next game!");
            self.game.next_game();
        } else if key == VirtualKeyCode::Right {
            let new_chair = match self.graphics.player_choices[self.graphics.player_num]
                .as_ref()
                .unwrap()
                .chair
            {
                Chair::Swivel => Chair::Recliner,
                Chair::Recliner => Chair::Beanbag,
                Chair::Beanbag => Chair::Ergonomic,
                Chair::Ergonomic => Chair::Folding,
                Chair::Folding => Chair::Swivel,
            };
            self.game.pick_chair(new_chair);
            self.graphics.maybe_select_chair(new_chair);
        } else if key == VirtualKeyCode::Left {
            let new_chair = match self.graphics.player_choices[self.graphics.player_num]
                .as_ref()
                .unwrap()
                .chair
            {
                Chair::Swivel => Chair::Folding,
                Chair::Recliner => Chair::Swivel,
                Chair::Beanbag => Chair::Recliner,
                Chair::Ergonomic => Chair::Beanbag,
                Chair::Folding => Chair::Ergonomic,
            };
            self.graphics.maybe_select_chair(new_chair);
            self.game.pick_chair(new_chair);
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

        let regions = &mut self.ui_regions;
        regions
            .iter_mut()
            .for_each(|reg| reg.set_hovering(x, y, &mut self.graphics, &mut self.game));
    }

    pub fn on_left_mouse(&mut self, state: ElementState) {
        let x = self.mouse_pos.x;
        let y = self.mouse_pos.y;

        match state {
            ElementState::Pressed => self
                .ui_regions
                .iter_mut()
                .for_each(|reg| reg.set_active(x, y, &mut self.graphics, &mut self.game)),
            ElementState::Released => self
                .ui_regions
                .iter_mut()
                .for_each(|reg| reg.set_inactive(&mut self.graphics, &mut self.game)),
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

    fn get_input_event_gamepad(
        &mut self,
        event: Result<(Button, f32), (Axis, f32)>,
    ) -> Option<InputEvent> {
        match event {
            // Button (value: [0, 1])
            Ok((button, value)) => {
                match button {
                    /***** GAMEPLAY *****/
                    // Item?
                    Button::South => None,

                    /***** CONTROLS *****/
                    // Accelerator
                    Button::RightTrigger2 if value > 0.0 => {
                        Some(InputEvent::Engine(EngineStatus::Accelerating(value)))
                    }
                    Button::RightTrigger2 => Some(InputEvent::Engine(EngineStatus::Neutral)),
                    // Brake
                    Button::LeftTrigger2 if value > 0.0 => {
                        Some(InputEvent::Engine(EngineStatus::Braking))
                    }
                    Button::LeftTrigger2 => Some(InputEvent::Engine(EngineStatus::Neutral)),
                    /***** MENU *****/ // TODO: this is temporary. we need a real way to handle menu input w controllers
                    // Force-start
                    Button::Start if value == 1.0 => {
                        self.game.force_start();
                        None
                    }
                    // Ready up
                    Button::Select if value == 1.0 => {
                        self.game.signal_ready_status(true);
                        None
                    }
                    Button::DPadLeft if value == 1.0 => {
                        let new_chair = match self.graphics.player_choices[self.graphics.player_num]
                            .as_ref()
                            .unwrap()
                            .chair
                        {
                            Chair::Swivel => Chair::Folding,
                            Chair::Recliner => Chair::Swivel,
                            Chair::Ergonomic => Chair::Recliner,
                            Chair::Beanbag => Chair::Ergonomic,
                            Chair::Folding => Chair::Beanbag,
                        };
                        self.game.pick_chair(new_chair);
                        None
                    }
                    Button::DPadRight if value == 1.0 => {
                        let new_chair = match self.graphics.player_choices[self.graphics.player_num]
                            .as_ref()
                            .unwrap()
                            .chair
                        {
                            Chair::Swivel => Chair::Recliner,
                            Chair::Recliner => Chair::Ergonomic,
                            Chair::Ergonomic => Chair::Beanbag,
                            Chair::Beanbag => Chair::Folding,
                            Chair::Folding => Chair::Swivel,
                        };
                        self.game.pick_chair(new_chair);
                        None
                    }
                    _ => None,
                }
            }
            // Axis (value: [-1, 1])
            Err((axis, value)) => match axis {
                /***** MOVEMENT *****/
                // Turn right
                Axis::LeftStickX if value > 0.0 => {
                    Some(InputEvent::Rotation(RotationStatus::InSpinClockwise(value)))
                }
                // Turn left
                Axis::LeftStickX if value < 0.0 => Some(InputEvent::Rotation(
                    RotationStatus::InSpinCounterclockwise(-value),
                )),
                // No turn
                Axis::LeftStickX => Some(InputEvent::Rotation(RotationStatus::NotInSpin)),

                /***** CAMERA *****/
                // TODO?
                Axis::RightStickX => None,
                Axis::RightStickY => None,
                _ => None,
            },
        }
    }

    pub fn handle_gamepad_event(&mut self, event: gilrs::Event) {
        let input_event = match event.event {
            EventType::ButtonChanged(button, value, _) => {
                self.get_input_event_gamepad(Ok((button, value)))
            }
            EventType::AxisChanged(axis, value, _) => {
                self.get_input_event_gamepad(Err((axis, value)))
            }
            EventType::Connected => {
                println!("Connected new gamepad #{}!", event.id);
                None
            }
            EventType::Disconnected => {
                println!("Gamepad #{} disconnected!", event.id);
                None
            }
            _ => None,
        };

        if let Some(valid_input_event) = input_event {
            self.game.send_input_event(valid_input_event);
        }
    }
}
