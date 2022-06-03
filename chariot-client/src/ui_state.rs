use std::time::{Duration, Instant};

use chariot_core::GLOBAL_CONFIG;
use glam::Vec2;
use image::ImageFormat;
use lazy_static::lazy_static;

use chariot_core::player::choices::Chair;
use chariot_core::questions::{QuestionData, QuestionOption};

use crate::drawable::AnimatedUIDrawable;
use crate::renderer::Renderer;
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

pub enum InteractionState {
    None,
    Voting {
        tally: Vec<u32>,
        end_time: Instant,
    },
    Active {
        question: QuestionData,
        choice: QuestionOption,
        end_time: Instant,
        bar_filled: bool,
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
        chair_description: UIDrawable,
        player_chair_images: Vec<Option<UIDrawable>>,
    },
    InGameHUD {
        place_position_image: UIDrawable,
        minimap_ui: UIDrawable,
        timer_ui: UIDrawable,
        lap_ui: UIDrawable,
        interaction_ui: AnimatedUIDrawable,
        interaction_text: UIDrawable,
        interaction_state: InteractionState,
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
        UIStringBuilder::new(*assets::fonts::PLACEMENT_TEXT_FONT)
            .alignment(StringAlignment::RIGHT)
            .content("")
            .position(1.0, 0.057);
    static ref TIMER_TEXT: UIStringBuilder = UIStringBuilder::new(assets::fonts::PRIMARY_FONT)
        .alignment(StringAlignment::LEFT)
        .content("00:00:000")
        .position(30.0 / 1280.0, 651.0 / 720.0);
    static ref LAP_TEXT: UIStringBuilder = UIStringBuilder::new(*assets::fonts::LAP_TEXT_FONT)
        .alignment(StringAlignment::LEFT)
        .content(format!("lap 0/{}", GLOBAL_CONFIG.number_laps).as_str())
        .position(30.0 / 1280.0, 0.35);
    static ref INTERACTION_TEXT: UIStringBuilder =
        UIStringBuilder::new(*assets::fonts::LAP_TEXT_FONT)
            .alignment(StringAlignment::CENTERED)
            .content("The audience is deciding your fate...");
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

    pub fn update_dynamic_ui(&mut self) {
        self.update_minimap();

        if let UIState::InGameHUD {
            interaction_ui,
            interaction_text,
            interaction_state,
            ..
        } = &mut self.ui
        {
            interaction_ui.update(&mut self.renderer);

            if let InteractionState::Active {
                end_time,
                bar_filled,
                ..
            } = interaction_state
            {
                let now = Instant::now();
                let last = interaction_ui.layers.len() - 1;
                if !*bar_filled && interaction_ui.layers[last].2.is_none() {
                    interaction_ui.layers.swap(0, last);
                    interaction_ui.layers.truncate(1);
                    let duration = *end_time - now;

                    interaction_ui.pos_to(
                        0,
                        glam::vec2(0.5, Self::INTERACTION_VOTING_Y_POS),
                        duration,
                    );
                    interaction_ui.size_to(
                        0,
                        self.renderer.pixel(0, Self::INTERACTION_VOTING_HEIGHT),
                        duration,
                    );
                }
                if now > *end_time {
                    *interaction_state = InteractionState::None;
                    interaction_ui.layers.clear();
                    interaction_text.layers.clear();
                }
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
                    Vec2::new(0.2 * location.0, 0.3 * location.1),
                    Vec2::new(0.02, 0.02),
                );
                let verts_data: &[u8] = bytemuck::cast_slice(&raw_verts_data);

                self.renderer
                    .write_buffer(&player_layer.vertex_buffer, verts_data);
            }
        }
    }

    const INTERACTION_VOTING_WIDTH: u32 = 600;
    const INTERACTION_VOTING_HEIGHT: u32 = 20;
    const INTERACTION_VOTING_Y_POS: f32 = 0.1;
    const INTERACTION_SECTION_COLORS: [[f32; 4]; 4] = [
        [254.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0, 1.0],
        [98.0 / 255.0, 87.0 / 255.0, 227.0 / 255.0, 1.0],
        [137.0 / 255.0, 202.0 / 255.0, 127.0 / 255.0, 1.0],
        [203.0 / 255.0, 157.0 / 255.0, 67.0 / 255.0, 1.0],
    ];

    pub fn begin_audience_voting(&mut self, num_options: usize, until_end: Duration) {
        if let UIState::InGameHUD {
            interaction_ui,
            interaction_state,
            interaction_text,
            ..
        } = &mut self.ui
        {
            interaction_ui.layers.clear();

            let local_origin = glam::vec2(
                0.5 - self.renderer.pixel_x(Self::INTERACTION_VOTING_WIDTH / 2),
                Self::INTERACTION_VOTING_Y_POS,
            );

            let EVEN_WIDTH = self
                .renderer
                .pixel_x(Self::INTERACTION_VOTING_WIDTH / num_options as u32);
            for i in 0..num_options {
                interaction_ui.push(UILayerTechnique::new(
                    &self.renderer,
                    local_origin + glam::vec2(EVEN_WIDTH * i as f32, 0.0),
                    glam::vec2(
                        EVEN_WIDTH,
                        self.renderer.pixel_y(Self::INTERACTION_VOTING_HEIGHT),
                    ),
                    glam::vec2(0.0, 0.0),
                    glam::vec2(1.0, 1.0),
                    self.resources.textures.get(&self.white_box_tex).unwrap(),
                ));
                let color = {
                    if i < 4 {
                        Self::INTERACTION_SECTION_COLORS[i]
                    } else {
                        [1.0, 1.0, 1.0, 1.0]
                    }
                };
                interaction_ui
                    .layers
                    .get_mut(i)
                    .unwrap()
                    .0
                    .update_color(&self.renderer, color);
            }

            *interaction_state = InteractionState::Voting {
                tally: vec![0; num_options],
                end_time: Instant::now() + until_end,
            };
            *interaction_text = INTERACTION_TEXT
                .clone()
                .position(0.5, Self::INTERACTION_VOTING_Y_POS / 1.5)
                .build_drawable(&self.renderer, &mut self.resources);
        }
    }

    pub fn update_audience_votes(&mut self, new_tally: Vec<u32>) {
        if let UIState::InGameHUD {
            interaction_ui,
            interaction_state,
            ..
        } = &mut self.ui
        {
            if let InteractionState::Voting { tally, .. } = interaction_state {
                let local_origin = glam::vec2(
                    0.5 - self.renderer.pixel_x(Self::INTERACTION_VOTING_WIDTH / 2),
                    Self::INTERACTION_VOTING_Y_POS,
                );

                let sum: u32 = new_tally.iter().sum();
                let mut sibling_push = 0;
                for idx in 0..new_tally.len() {
                    let width = Self::INTERACTION_VOTING_WIDTH * new_tally[idx] / sum;
                    interaction_ui.layers[idx].0.update_pos(
                        &self.renderer,
                        local_origin + self.renderer.pixel(sibling_push, 0),
                    );
                    interaction_ui.layers[idx].0.update_size(
                        &self.renderer,
                        self.renderer.pixel(width, Self::INTERACTION_VOTING_HEIGHT),
                    );
                    sibling_push += width;
                }

                *tally = new_tally;
            }
        }
    }

    pub fn start_audience_interaction(
        &mut self,
        question: QuestionData,
        choice: QuestionOption,
        duration: Duration,
    ) {
        if let UIState::InGameHUD {
            interaction_ui,
            interaction_text,
            interaction_state,
            ..
        } = &mut self.ui
        {
            // TODO: fix hardcode
            let VICTOR_IDX = 1;
            let local_origin = glam::vec2(
                0.5 - self.renderer.pixel_x(Self::INTERACTION_VOTING_WIDTH / 2),
                Self::INTERACTION_VOTING_Y_POS,
            );

            interaction_ui.pos_to(VICTOR_IDX, local_origin, Duration::from_secs(1));
            interaction_ui.size_to(
                VICTOR_IDX,
                self.renderer.pixel(
                    Self::INTERACTION_VOTING_WIDTH,
                    Self::INTERACTION_VOTING_HEIGHT,
                ),
                Duration::from_secs(1),
            );

            let last = interaction_ui.layers.len() - 1;
            interaction_ui.layers.swap(VICTOR_IDX, last);

            *interaction_text = INTERACTION_TEXT
                .clone()
                .content(choice.action.get_description())
                .position(0.5, Self::INTERACTION_VOTING_Y_POS / 1.5)
                .build_drawable(&self.renderer, &mut self.resources);
            *interaction_state = InteractionState::Active {
                question,
                choice,
                end_time: Instant::now() + duration,
                bar_filled: false,
            };
        }
    }

    pub fn maybe_update_lap(&mut self, lap: u8) {
        if let UIState::InGameHUD { ref mut lap_ui, .. } = self.ui {
            *lap_ui = LAP_TEXT
                .clone()
                .content(format!("lap {}/{}", lap, GLOBAL_CONFIG.number_laps).as_str())
                .build_drawable(&self.renderer, &mut self.resources);
        }
    }

    pub fn maybe_update_place(&mut self, position: u8) {
        if let UIState::InGameHUD {
            ref mut place_position_image,
            ..
        } = self.ui
        {
            let texture_name = match position {
                1 => "1st",
                2 => "2nd",
                3 => "3rd",
                4 => "4th",
                _ => "1st",
            };
            let placement_handle = self.resources.import_texture_embedded(
                &self.renderer,
                texture_name,
                assets::ui::PLACE_IMAGES[position as usize - 1],
                ImageFormat::Png,
            );

            let place_position_texture = self
                .resources
                .textures
                .get(&placement_handle)
                .expect("Expected placement text image!");

            *place_position_image = UIDrawable {
                layers: vec![technique::UILayerTechnique::new(
                    &self.renderer,
                    glam::vec2(1117.0 / 1280.0, 590.0 / 720.0),
                    glam::vec2(0.1, 0.15),
                    glam::vec2(0.0, 0.0),
                    glam::vec2(1.0, 1.0),
                    &place_position_texture,
                )],
            };
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

        let layer_vec = vec![UILayerTechnique::new(
            &self.renderer,
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &background_texture,
        )];

        let background = UIDrawable { layers: layer_vec };

        self.ui = UIState::MainMenu { background };
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

        let layer_vec = vec![UILayerTechnique::new(
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

        let chair_description_handle = self.resources.import_texture_embedded(
            &self.renderer,
            "swivel_description",
            assets::ui::get_chair_description(Chair::Swivel),
            ImageFormat::Png,
        );

        let chair_select_box_texture = self
            .resources
            .textures
            .get(&chair_select_box_handle)
            .expect("background doesn't exist!");

        let chair_description_texture = self
            .resources
            .textures
            .get(&chair_description_handle)
            .expect("description doesn't exist!");

        let layer_vec = vec![technique::UILayerTechnique::new(
            &self.renderer,
            glam::vec2(343.0 / 1280.0, 565.0 / 720.0),
            glam::vec2(128.0 / 1280.0, 122.0 / 720.0),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &chair_select_box_texture,
        )];

        let chair_select_box = UIDrawable { layers: layer_vec };

        let layer_vec = vec![technique::UILayerTechnique::new(
            &self.renderer,
            glam::vec2(317.0 / 1280.0, 433.0 / 720.0),
            glam::vec2(640.0 / 1280.0, 117.0 / 720.0),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &chair_description_texture,
        )];

        let chair_description = UIDrawable { layers: layer_vec };

        self.ui = UIState::ChairacterSelect {
            background,
            chair_select_box,
            chair_description,
            player_chair_images: vec![None, None, None, None],
        };

        (0..4).for_each(|i| self.maybe_display_chair(None, i));
    }

    pub fn maybe_select_chair(&mut self, chair: Chair) {
        if let UIState::ChairacterSelect {
            chair_select_box,
            chair_description,
            ..
        } = &mut self.ui
        {
            let position = match chair {
                Chair::Swivel => glam::vec2(343.0 / 1280.0, 565.0 / 720.0),
                Chair::Recliner => glam::vec2(460.0 / 1280.0, 565.0 / 720.0),
                Chair::Beanbag => glam::vec2(576.0 / 1280.0, 565.0 / 720.0),
                Chair::Ergonomic => glam::vec2(693.0 / 1280.0, 565.0 / 720.0),
                Chair::Folding => glam::vec2(809.0 / 1280.0, 565.0 / 720.0),
            };
            // not sure the best way to change the position; for now, I'm just rerendering completely
            let chair_select_box_handle = self.resources.import_texture_embedded(
                &self.renderer,
                format!("p{}rectangle", self.player_num).as_str(),
                assets::ui::CHAIR_SELECT_RECT[self.player_num],
                ImageFormat::Png,
            );

            let chair_description_handle = self.resources.import_texture_embedded(
                &self.renderer,
                format!("{}_description", chair.file()).as_str(),
                assets::ui::get_chair_description(chair),
                ImageFormat::Png,
            );

            let chair_select_box_texture = self
                .resources
                .textures
                .get(&chair_select_box_handle)
                .expect("chair select box doesn't exist!");

            let chair_description_texture = self
                .resources
                .textures
                .get(&chair_description_handle)
                .expect("description doesn't exist!");

            let layer_vec = vec![technique::UILayerTechnique::new(
                &self.renderer,
                position,
                glam::vec2(127.0 / 1280.0, 121.0 / 720.0),
                glam::vec2(0.0, 0.0),
                glam::vec2(1.0, 1.0),
                &chair_select_box_texture,
            )];

            *chair_select_box = UIDrawable { layers: layer_vec };

            let layer_vec = vec![technique::UILayerTechnique::new(
                &self.renderer,
                glam::vec2(317.0 / 1280.0, 433.0 / 720.0),
                glam::vec2(640.0 / 1280.0, 117.0 / 720.0),
                glam::vec2(0.0, 0.0),
                glam::vec2(1.0, 1.0),
                &chair_description_texture,
            )];

            *chair_description = UIDrawable { layers: layer_vec };

            for (player_id, choice) in self.player_choices.clone().iter().flatten().enumerate() {
                self.maybe_display_chair(Some(choice.chair), player_id);
            }
        }
    }

    pub fn maybe_display_chair(&mut self, chair: Option<Chair>, player: usize) {
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
                .expect(format!("chair doesn't exist!").as_str());

            let position = match player {
                0 => glam::vec2(165.0 / 1280.0, 187.0 / 720.0),
                1 => glam::vec2(422.0 / 1280.0, 146.0 / 720.0),
                2 => glam::vec2(697.0 / 1280.0, 196.0 / 720.0),
                3 => glam::vec2(956.0 / 1280.0, 146.0 / 720.0),
                _ => glam::vec2(165.0 / 1280.0, 187.0 / 720.0),
            };

            let layers = vec![UILayerTechnique::new(
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
        let place_1st_handle = self.resources.import_texture_embedded(
            &self.renderer,
            "1st",
            assets::ui::PLACE_IMAGES[0],
            ImageFormat::Png,
        );

        let place_position_texture = self
            .resources
            .textures
            .get(&place_1st_handle)
            .expect("Expected placement text image!");

        let place_position_image = UIDrawable {
            layers: vec![technique::UILayerTechnique::new(
                &self.renderer,
                glam::vec2(1117.0 / 1280.0, 590.0 / 720.0),
                glam::vec2(0.1, 0.1),
                glam::vec2(0.0, 0.0),
                glam::vec2(1.0, 1.0),
                &place_position_texture,
            )],
        };

        let timer_ui = TIMER_TEXT
            .clone()
            .build_drawable(&self.renderer, &mut self.resources);

        let lap_ui = LAP_TEXT
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

        let mut layer_vec = vec![UILayerTechnique::new(
            &self.renderer,
            glam::vec2(0.0, 0.0),
            glam::vec2(0.2, 0.3),
            glam::vec2(0.0, 0.0),
            glam::vec2(1.0, 1.0),
            &minimap_map_texture,
        )];
        layer_vec.append(&mut player_location_markers);

        let minimap_ui = UIDrawable { layers: layer_vec };

        self.ui = UIState::InGameHUD {
            place_position_image,
            minimap_ui,
            timer_ui,
            lap_ui,
            interaction_ui: AnimatedUIDrawable::new(),
            interaction_text: UIDrawable { layers: vec![] },
            interaction_state: InteractionState::None,
        }
    }
}
