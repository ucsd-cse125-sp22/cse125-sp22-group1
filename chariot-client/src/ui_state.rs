use std::{fmt, time::Instant};

use glam::Vec2;
use ordinal::Ordinal;

use crate::{
    drawable::{
        string::StringDrawable,
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

pub enum UIState {
    LoadingScreen {
        loading_text: StringDrawable,
    },
    InGameHUD {
        place_position_text: StringDrawable,
        game_announcement_title: StringDrawable,
        game_announcement_subtitle: StringDrawable,
        announcement_state: AnnouncementState,
        minimap_ui: UIDrawable,
    },
}
impl fmt::Display for UIState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printable = match self {
            UIState::LoadingScreen { .. } => "loading text",
            UIState::InGameHUD {
                announcement_state, ..
            } => match announcement_state {
                AnnouncementState::None => "none",
                AnnouncementState::VotingInProgress { .. } => "voting in progress",
                AnnouncementState::VoteActiveTime { .. } => "vote active time",
            },
        };
        write!(f, "{}", printable)
    }
}

impl GraphicsManager {
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
            place_position_text,
            ..
        } = &mut self.ui
        {
            place_position_text.set(
                Ordinal(position).to_string().as_str(),
                &self.renderer,
                &mut self.resources,
            );
        }
    }

    pub fn display_hud(&mut self) {
        let mut place_position_text =
            StringDrawable::new("PressStart2P-Regular", 38.0, Vec2::new(0.905, 0.057));
        place_position_text.set("tbd", &self.renderer, &mut self.resources);

        let mut game_announcement_title =
            StringDrawable::new("ArialMT", 32.0, Vec2::new(0.50, 0.04));
        game_announcement_title.set("NO MORE LEFT TURNS", &self.renderer, &mut self.resources);
        let mut game_announcement_subtitle =
            StringDrawable::new("ArialMT", 32.0, Vec2::new(0.50, 0.14));
        game_announcement_subtitle.set(
            "activating in 20 seconds",
            &self.renderer,
            &mut self.resources,
        );

        // minimap
        let minimap_map_handle = self
            .resources
            .import_texture(&self.renderer, "UI/minimap/track_transparent.png");
        let player_location_handles: Vec<TextureHandle> = [
            "UI/Map Select/P1Btn.png",
            "UI/Map Select/P2Btn.png",
            "UI/Map Select/P3Btn.png",
            "UI/Map Select/P4Btn.png",
        ]
        .iter()
        .map(|filename| self.resources.import_texture(&self.renderer, filename))
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
        }
    }
}
