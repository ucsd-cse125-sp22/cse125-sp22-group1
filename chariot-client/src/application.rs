use chariot_core::sound_effect::SoundEffect;
use std::collections::HashSet;
use std::time::{Duration, Instant};

use std::time::SystemTime;

use winit::event::VirtualKeyCode;

use crate::assets::audio::{get_sfx, CYBER_RECLINER, HOLD_ON_TO_YOUR_SEATS};
use crate::audio::AudioManager;
use chariot_core::networking::ClientBoundPacket;
use chariot_core::GLOBAL_CONFIG;

use crate::game::GameClient;
use crate::graphics::GraphicsManager;

use crate::audio::thread::context::AudioCtx;
use crate::audio::thread::options::SourceOptions;
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
    last_update: SystemTime,
    game_start_time: SystemTime,
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
            game_start_time: SystemTime::now(),
            last_update: SystemTime::now(),
        }
    }

    pub fn render(&mut self) {
        self.graphics.render();
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
                ClientBoundPacket::PlacementUpdate(given_position) => {
                    let position = if given_position > 4 {
                        1
                    } else {
                        given_position
                    };

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
        self.graphics.update_minimap();
    }
}
