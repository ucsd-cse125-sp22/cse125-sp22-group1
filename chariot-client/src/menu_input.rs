use chariot_core::player::{
    choices::Chair,
    player_inputs::{EngineStatus, InputEvent, RotationStatus},
};
use gilrs::{Axis, Button, Event, EventType};
use winit::event::VirtualKeyCode;

use crate::{
    application::Application, assets::audio::get_sfx, audio::thread::options::SourceOptions,
    graphics::register_passes, ui_state::UIState,
};

impl Application {
    fn input_gamepad_main_menu(&mut self, event: Result<(Button, f32), (Axis, f32)>) {
        if let Ok(_) = event {
            self.graphics.display_chairacter_select();
            self.sfx_manager.play(
                get_sfx(chariot_core::sound_effect::SoundEffect::EnterChairacterSelect),
                &self.audio_context,
                SourceOptions::new(),
            );
        }
    }

    fn input_keyboard_main_menu(&mut self, _: VirtualKeyCode) {
        self.graphics.display_chairacter_select();
        self.sfx_manager.play(
            get_sfx(chariot_core::sound_effect::SoundEffect::EnterChairacterSelect),
            &self.audio_context,
            SourceOptions::new(),
        );
    }

    fn input_gamepad_chairacter_select(&mut self, event: Result<(Button, f32), (Axis, f32)>) {
        if let Ok((button, value)) = event {
            match button {
                // Force-start
                Button::Select if value == 1.0 => {
                    self.game.force_start();
                }
                // Ready up
                Button::Start if value == 1.0 => {
                    self.game.signal_ready_status(true);
                    self.sfx_manager.play(
                        get_sfx(chariot_core::sound_effect::SoundEffect::ReadyUp),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
                }
                Button::DPadRight if value == 1.0 => {
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
                    self.graphics.maybe_select_chair(new_chair);
                    self.game.pick_chair(new_chair);
                    self.sfx_manager.play(
                        get_sfx(chariot_core::sound_effect::SoundEffect::SelectChairacter),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
                }
                Button::DPadLeft if value == 1.0 => {
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
                    self.sfx_manager.play(
                        get_sfx(chariot_core::sound_effect::SoundEffect::SelectChairacter),
                        &self.audio_context,
                        SourceOptions::new(),
                    );
                }
                _ => {}
            }
        }
    }

    fn input_keyboard_chairacter_select(&mut self, key: VirtualKeyCode) {
        if key == VirtualKeyCode::Right {
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
            self.sfx_manager.play(
                get_sfx(chariot_core::sound_effect::SoundEffect::SelectChairacter),
                &self.audio_context,
                SourceOptions::new(),
            );
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
            self.sfx_manager.play(
                get_sfx(chariot_core::sound_effect::SoundEffect::SelectChairacter),
                &self.audio_context,
                SourceOptions::new(),
            );
        } else if key == VirtualKeyCode::Up {
            self.game.signal_ready_status(true);
            self.sfx_manager.play(
                get_sfx(chariot_core::sound_effect::SoundEffect::ReadyUp),
                &self.audio_context,
                SourceOptions::new(),
            );
        }
    }

    fn get_gamepad_input_event_in_game(
        &self,
        event: Result<(Button, f32), (Axis, f32)>,
    ) -> Option<InputEvent> {
        match event {
            // Button (value: [0, 1])
            Ok((button, value)) => match button {
                Button::RightTrigger2 if value > 0.0 => {
                    Some(InputEvent::Engine(EngineStatus::Accelerating(value)))
                }
                Button::RightTrigger2 => Some(InputEvent::Engine(EngineStatus::Neutral)),

                Button::LeftTrigger2 if value > 0.0 => {
                    Some(InputEvent::Engine(EngineStatus::Braking))
                }
                Button::LeftTrigger2 => Some(InputEvent::Engine(EngineStatus::Neutral)),
                _ => None,
            },
            // Axis (value: [-1, 1])
            Err((axis, value)) => match axis {
                Axis::LeftStickX if value > 0.0 => {
                    Some(InputEvent::Rotation(RotationStatus::InSpinClockwise(value)))
                }
                Axis::LeftStickX if value < 0.0 => Some(InputEvent::Rotation(
                    RotationStatus::InSpinCounterclockwise(-value),
                )),
                Axis::LeftStickX => Some(InputEvent::Rotation(RotationStatus::NotInSpin)),
                _ => None,
            },
        }
    }

    fn input_gamepad_in_game(&mut self, event: Result<(Button, f32), (Axis, f32)>) {
        if let Some(valid_input_event) = self.get_gamepad_input_event_in_game(event) {
            self.game.send_input_event(valid_input_event);
        }
    }

    // Input configuration
    fn get_keyboard_input_event_in_game(&self, key: VirtualKeyCode) -> Option<InputEvent> {
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
            _ => None,
        }
    }

    fn input_keyboard_in_game(&mut self, key: VirtualKeyCode) {
        if let Some(event) = self.get_keyboard_input_event_in_game(key) {
            self.game.send_input_event(event);
        };
    }

    fn input_gamepad_final_standings(&mut self, event: Result<(Button, f32), (Axis, f32)>) {
        if let Ok((button, _)) = event {
            if matches!(button, Button::Select) || matches!(button, Button::Start) {
                self.game.next_game();
                self.sfx_manager.play(
                    get_sfx(chariot_core::sound_effect::SoundEffect::ReadyUp),
                    &self.audio_context,
                    SourceOptions::new(),
                );
            }
        }
    }

    fn input_keyboard_final_standings(&mut self, key: VirtualKeyCode) {
        if matches!(key, VirtualKeyCode::Escape) {
            self.game.next_game();
            self.sfx_manager.play(
                get_sfx(chariot_core::sound_effect::SoundEffect::ReadyUp),
                &self.audio_context,
                SourceOptions::new(),
            );
        }
    }

    pub fn handle_gamepad_event(&mut self, event: Event) {
        let input_event = match event.event {
            EventType::ButtonChanged(button, value, _) => Some(Ok((button, value))),
            EventType::AxisChanged(axis, value, _) => Some(Err((axis, value))),
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

        if let Some(input_event) = input_event {
            match self.graphics.ui {
                UIState::None => {}
                UIState::MainMenu { .. } => self.input_gamepad_main_menu(input_event),
                UIState::ChairacterSelect { .. } => {
                    self.input_gamepad_chairacter_select(input_event)
                }
                UIState::InGameHUD { .. } => self.input_gamepad_in_game(input_event),
                UIState::FinalStandings { .. } => self.input_gamepad_final_standings(input_event),
            }
        }
    }

    pub fn on_key_down(&mut self, key: VirtualKeyCode) {
        // winit sends duplicate keydown events, so we will just make sure we don't already have this processed
        if self.pressed_keys.contains(&key) {
            return;
        };

        self.pressed_keys.insert(key);

        // special cases :D
        if key == VirtualKeyCode::R {
            println!("Reloading shaders");
            register_passes(&mut self.graphics.renderer);
        } else if key == VirtualKeyCode::L {
            println!("Forcing a start!");
            self.game.force_start();
        } else if key == VirtualKeyCode::P {
            println!("Starting next game!");
            self.game.next_game();
        } else if key == VirtualKeyCode::F11 {
            println!("toggling fullscreen");
            self.graphics.renderer.context.toggle_fullscreen();
        }

        match self.graphics.ui {
            UIState::None => {}
            UIState::MainMenu { .. } => self.input_keyboard_main_menu(key),
            UIState::ChairacterSelect { .. } => self.input_keyboard_chairacter_select(key),
            UIState::InGameHUD { .. } => self.input_keyboard_in_game(key),
            UIState::FinalStandings { .. } => self.input_keyboard_final_standings(key),
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

    pub fn on_key_up(&mut self, key: VirtualKeyCode) {
        self.pressed_keys.remove(&key);

        if let UIState::InGameHUD { .. } = self.graphics.ui {
            if let Some(event) = self.invert_event(self.get_keyboard_input_event_in_game(key)) {
                self.game.send_input_event(event);
            };
        }
    }
}
