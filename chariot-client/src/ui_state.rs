use std::time::{Duration, Instant};

use glam::Vec2;
use image::ImageFormat;
use lazy_static::lazy_static;
use ordinal::Ordinal;

use chariot_core::player::choices::Chair;

use crate::ui::string::{StringAlignment, UIStringBuilder};
use crate::{
    assets,
    drawable::{
        technique::{self, UILayerTechnique},
        UIDrawable,
    },
    graphics::GraphicsManager,
    resources::TextureHandle,
    scenegraph::components::Transform,
    ui::ui_region::UIRegion,
};

pub enum AnnouncementState {
    None,
    VotingInProgress {
        prompt: String,
        vote_end_time: Instant,
    },
    VoteActiveTime {
        prompt: String,
        decision: String,
        effect_end_time: Instant,
    },
}

pub enum UIState {
    None,
    MainMenu {
        background: UIDrawable,
    },
    ChairacterSelect {
        background: UIDrawable,
        chair_select_box: UIDrawable,
        player_chair_images: Vec<Option<UIDrawable>>,
    },
    InGameHUD {
        place_position_text: UIDrawable,
        game_announcement_title: UIDrawable,
        game_announcement_subtitle: UIDrawable,
        announcement_state: AnnouncementState,
        minimap_ui: UIDrawable,
        timer_ui: UIDrawable,
    },
}

// by initializing the builders statically,
// we can quickly clone then and change their content to regenerate drawables
lazy_static! {
    static ref ANNOUNCEMENT_TITLE: UIStringBuilder =
        UIStringBuilder::new(assets::fonts::PRIMARY_FONT)
            .alignment(StringAlignment::CENTERED)
            .content("")
            .position(0.50, 0.04);
    static ref ANNOUNCEMENT_SUBTITLE: UIStringBuilder =
        UIStringBuilder::new(assets::fonts::PRIMARY_FONT)
            .alignment(StringAlignment::CENTERED)
            .content("")
            .position(0.50, 0.14);
    static ref PLACEMENT_TEXT: UIStringBuilder =
        UIStringBuilder::new(*assets::fonts::PLACEMENT_FONT_SELECTION)
            .alignment(StringAlignment::RIGHT)
            .content("")
            .position(1.0, 0.057);
    static ref TIMER_TEXT: UIStringBuilder = UIStringBuilder::new(assets::fonts::PRIMARY_FONT)
        .alignment(StringAlignment::RIGHT)
        .content("00:00:000")
        .position(0.95, 0.9);
}

impl GraphicsManager {
    pub fn update_timer(&mut self, time_elapsed: Duration) {
        if let UIState::InGameHUD {
            ref mut timer_ui, ..
        } = self.ui
        {
            let minutes = time_elapsed.as_secs() / 60;
            let seconds = time_elapsed.as_secs() % 60;
            let millis = time_elapsed.subsec_millis();
            let time = format!("{:02}:{:02}:{:03}", minutes, seconds, millis);
            *timer_ui = TIMER_TEXT
                .clone()
                .content(&time)
                .build_drawable(&self.renderer, &mut self.resources);
        }
    }

    pub fn make_announcement(&mut self, title: &str, subtitle: &str) {
        if let UIState::InGameHUD {
            ref mut game_announcement_subtitle,
            ref mut game_announcement_title,
            ..
        } = self.ui
        {
            *game_announcement_title = ANNOUNCEMENT_TITLE
                .clone()
                .content(title)
                .build_drawable(&self.renderer, &mut self.resources);
            *game_announcement_subtitle = ANNOUNCEMENT_SUBTITLE
                .clone()
                .content(subtitle)
                .build_drawable(&self.renderer, &mut self.resources);
        }
    }

    pub fn update_voting_announcements(&mut self) {
        if let UIState::InGameHUD {
            ref announcement_state,
            ..
        } = &self.ui
        {
            match announcement_state {
                AnnouncementState::VotingInProgress { vote_end_time, .. } => {
                    self.make_announcement(
                        "The audience is deciding your fate",
                        format!(
                            "They decide in {} seconds",
                            (*vote_end_time - Instant::now()).as_secs()
                        )
                        .as_str(),
                    );
                }
                AnnouncementState::VoteActiveTime {
                    prompt: _,
                    decision,
                    effect_end_time,
                } => {
                    let effect_end_time = effect_end_time;
                    self.make_announcement(
                        format!("{} was chosen!", decision).as_str(),
                        format!(
                            "Effects will last for another {} seconds",
                            (*effect_end_time - Instant::now()).as_secs()
                        )
                        .as_str(),
                    );
                }
                AnnouncementState::None => {}
            }
        }
    }

    pub fn update_minimap(&mut self) {
        if let UIState::InGameHUD { minimap_ui, .. } = &mut self.ui {
            // Only update if we actually have entities to map
            if !self.player_entities.iter().all(Option::is_some) {
                return;
            }

            // Convert "map units" locations into proportions of minimap size
            fn get_minimap_player_location(location: (f32, f32)) -> (f32, f32) {
                // these values are guesses btw
                const MIN_TRACK_X: f32 = -119.0; // top
                const MAX_TRACK_X: f32 = 44.0; // bottom
                const MIN_TRACK_Z: f32 = -48.0; // right
                const MAX_TRACK_Z: f32 = 119.0; // left

                (
                    (MAX_TRACK_Z - location.1) / (MAX_TRACK_Z - MIN_TRACK_Z),
                    (location.0 - MIN_TRACK_X) / (MAX_TRACK_X - MIN_TRACK_X),
                )
            }

            let player_locations = self
                .player_entities
                .iter()
                .map(|player_num| {
                    let location = self
                        .world
                        .get::<Transform>(player_num.unwrap())
                        .unwrap()
                        .translation;
                    (location.x, location.z)
                })
                .map(get_minimap_player_location);

            for (player_index, location) in player_locations.enumerate() {
                let player_layer = minimap_ui.layers.get_mut(player_index + 1).unwrap();

                let raw_verts_data = UILayerTechnique::create_verts_data(
                    Vec2::new(0.2 * location.0, 0.2 * location.1),
                    Vec2::new(0.02, 0.02),
                );
                let verts_data: &[u8] = bytemuck::cast_slice(&raw_verts_data);

                self.renderer
                    .write_buffer(&player_layer.vertex_buffer, verts_data);
            }
        }
    }

    pub fn maybe_set_announcement_state(&mut self, new_announcement_state: AnnouncementState) {
        if let UIState::InGameHUD {
            announcement_state, ..
        } = &mut self.ui
        {
            *announcement_state = new_announcement_state;
        }
    }

    pub fn maybe_update_place(&mut self, position: u8) {
        if let UIState::InGameHUD {
            ref mut place_position_text,
            ..
        } = self.ui
        {
            *place_position_text = PLACEMENT_TEXT
                .clone()
                .content(Ordinal(position).to_string().as_str())
                .build_drawable(&self.renderer, &mut self.resources);
        }
    }

    pub fn display_main_menu(&mut self) {
        let background_handle = self.resources.import_texture_embedded(
            &self.renderer,
            "homebackground",
            assets::ui::HOME_BACKGROUND,
            ImageFormat::Png,
        );

        let background_texture = self
            .resources
            .textures
            .get(&background_handle)
            .expect("main menu background doesn't exist!");

        let layer_vec = vec![technique::UILayerTechnique::new(
            &self.renderer,
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &background_texture,
        )];

        let background = UIDrawable { layers: layer_vec };

        self.ui = UIState::MainMenu { background };

        // join lobby button
        let mut join_lobby_button = UIRegion::new(472.0, 539.0, 336.0, 87.0);
        join_lobby_button.on_click(|graphics, game| {
            graphics.display_chairacter_select();
            game.pick_chair(Chair::Swivel);
        });

        self.ui_regions = vec![join_lobby_button];
    }

    pub fn display_chairacter_select(&mut self) {
        let background_handle = self.resources.import_texture_embedded(
            &self.renderer,
            "chair-select background",
            assets::ui::CHAIR_SELECT_BACKGROUND,
            ImageFormat::Png,
        );

        let background_texture = self
            .resources
            .textures
            .get(&background_handle)
            .expect("background doesn't exist!");

        let layer_vec = vec![technique::UILayerTechnique::new(
            &self.renderer,
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &background_texture,
        )];

        let background = UIDrawable { layers: layer_vec };

        let chair_select_box_handle = self.resources.import_texture_embedded(
            &self.renderer,
            format!("p{}rectangle", self.player_num).as_str(),
            assets::ui::CHAIR_SELECT_RECT[self.player_num],
            ImageFormat::Png,
        );

        let chair_select_box_texture = self
            .resources
            .textures
            .get(&chair_select_box_handle)
            .expect("background doesn't exist!");

        let layer_vec = vec![technique::UILayerTechnique::new(
            &self.renderer,
            glam::vec2(304.0 / 1280.0, 548.0 / 720.0),
            glam::vec2(141.0 / 1280.0, 134.0 / 720.0),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &chair_select_box_texture,
        )];

        let chair_select_box = UIDrawable { layers: layer_vec };
        self.ui = UIState::ChairacterSelect {
            background,
            chair_select_box,
            player_chair_images: vec![None, None, None, None],
        };

        // buttons
        let mut main_menu_button = UIRegion::new(40.0, 587.0, 224.0, 64.0);
        main_menu_button.on_click(|graphics, _game_client| {
            graphics.display_main_menu();
        });

        let mut ready_button = UIRegion::new(999.0, 587.0, 225.0, 64.0);
        ready_button.on_click(|_graphics, game_client| {
            game_client.signal_ready_status(true);
        });

        let mut force_start_button = UIRegion::new(999.0, 637.0, 225.0, 31.0);
        force_start_button.on_click(|_graphics, game_client| {
            game_client.force_start();
        });

        self.ui_regions = vec![main_menu_button, ready_button, force_start_button];
    }

    pub fn maybe_select_chair(&mut self, chair: Chair) {
        if let UIState::ChairacterSelect {
            chair_select_box, ..
        } = &mut self.ui
        {
            let position = match chair {
                Chair::Swivel => glam::vec2(304.0 / 1280.0, 548.0 / 720.0),
                Chair::Recliner => glam::vec2(437.0 / 1280.0, 548.0 / 720.0),
                Chair::Beanbag => glam::vec2(570.0 / 1280.0, 548.0 / 720.0),
                Chair::Ergonomic => glam::vec2(703.0 / 1280.0, 548.0 / 720.0),
                Chair::Folding => glam::vec2(835.0 / 1280.0, 548.0 / 720.0),
            };
            // not sure the best way to change the position; for now, I'm just rerendering completely
            let chair_select_box_handle = self.resources.import_texture_embedded(
                &self.renderer,
                format!("p{}rectangle", self.player_num).as_str(),
                assets::ui::CHAIR_SELECT_RECT[self.player_num],
                ImageFormat::Png,
            );

            let chair_select_box_texture = self
                .resources
                .textures
                .get(&chair_select_box_handle)
                .expect("chair select box doesn't exist!");

            let layer_vec = vec![technique::UILayerTechnique::new(
                &self.renderer,
                position,
                glam::vec2(141.0 / 1280.0, 134.0 / 720.0),
                glam::vec2(0.0, 0.0),
                glam::vec2(1.0, 1.0),
                &chair_select_box_texture,
            )];

            *chair_select_box = UIDrawable { layers: layer_vec };

            for (player_id, choice) in self.player_choices.clone().iter().flatten().enumerate() {
                self.maybe_display_chair(choice.chair, player_id);
            }
        }
    }

    pub fn maybe_display_chair(&mut self, chair: Chair, player: usize) {
        if let UIState::ChairacterSelect {
            player_chair_images,
            ..
        } = &mut self.ui
        {
            let chair_image = self.resources.import_texture_embedded(
                &self.renderer,
                "chair headshot",
                assets::ui::get_chair_image(chair),
                ImageFormat::Png,
            );

            let chair_texture = self
                .resources
                .textures
                .get(&chair_image)
                .expect(format!("{} doesn't exist!", chair.to_string()).as_str());

            let position = match player {
                0 => glam::vec2(165.0 / 1280.0, 187.0 / 720.0),
                1 => glam::vec2(422.0 / 1280.0, 146.0 / 720.0),
                2 => glam::vec2(697.0 / 1280.0, 196.0 / 720.0),
                3 => glam::vec2(956.0 / 1280.0, 146.0 / 720.0),
                _ => glam::vec2(165.0 / 1280.0, 187.0 / 720.0),
            };

            let layers = vec![technique::UILayerTechnique::new(
                &self.renderer,
                position,
                glam::vec2(166.0 / 1280.0, 247.0 / 720.0),
                glam::vec2(0.0, 0.0),
                glam::vec2(1.0, 1.0),
                &chair_texture,
            )];

            player_chair_images[player] = Some(UIDrawable { layers });
        }
    }

    pub fn display_hud(&mut self) {
        let place_position_text = PLACEMENT_TEXT
            .clone()
            .build_drawable(&self.renderer, &mut self.resources);

        let game_announcement_title = ANNOUNCEMENT_TITLE
            .clone()
            .build_drawable(&self.renderer, &mut self.resources);

        let game_announcement_subtitle = ANNOUNCEMENT_SUBTITLE
            .clone()
            .build_drawable(&self.renderer, &mut self.resources);

        let timer_ui = TIMER_TEXT
            .clone()
            .build_drawable(&self.renderer, &mut self.resources);

        // minimap
        let minimap_map_handle = self.resources.import_texture_embedded(
            &self.renderer,
            "track_transparent",
            assets::ui::TRACK_TRANSPARENT,
            ImageFormat::Png,
        );

        let player_location_handles: Vec<TextureHandle> = assets::ui::PLAYER_BUTTONS
            .iter()
            .map(|button| {
                self.resources.import_texture_embedded(
                    &self.renderer,
                    "player button",
                    *button,
                    ImageFormat::Png,
                )
            })
            .collect();

        let minimap_map_texture = self
            .resources
            .textures
            .get(&minimap_map_handle)
            .expect("minimap doesn't exist!");

        let mut player_location_markers: Vec<UILayerTechnique> = player_location_handles
            .iter()
            .map(|handle| self.resources.textures.get(&handle).unwrap())
            .map(|texture| {
                UILayerTechnique::new(
                    &self.renderer,
                    glam::vec2(0.0, 0.0),
                    glam::vec2(0.02, 0.02),
                    glam::vec2(0.0, 0.0),
                    glam::vec2(1.0, 1.0),
                    &texture,
                )
            })
            .collect();

        let mut layer_vec = vec![technique::UILayerTechnique::new(
            &self.renderer,
            glam::vec2(0.0, 0.0),
            glam::vec2(0.2, 0.2),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &minimap_map_texture,
        )];
        layer_vec.append(&mut player_location_markers);

        let minimap_ui = UIDrawable { layers: layer_vec };

        self.ui = UIState::InGameHUD {
            place_position_text,
            game_announcement_title,
            game_announcement_subtitle,
            announcement_state: AnnouncementState::None,
            minimap_ui,
            timer_ui,
        }
    }

    pub fn get_ui_regions(&mut self) -> Vec<UIRegion> {
        std::mem::replace(&mut self.ui_regions, vec![])
    }
}
