use crate::{checkpoints::MinorCheckpoint, physics::player_entity::PlayerEntity};

impl PlayerEntity {
    // Values returned aren't intended to be interpreted directly, only compared
    fn get_progress_score(&self, minor_checkpoints: &Vec<MinorCheckpoint>) -> (u8, u8, u8, f64) {
        // There's a hierarchy of four pieces of progress information we have:
        // from least to most granular lap number, zone number (zones are
        // between major checkpoints), minor checkpoint number, and linear
        // interpolation between minor checkpoint
        let lap_number = self.lap_info.lap;
        let zone_number = self.lap_info.zone;
        let checkpoint_number = self.lap_info.last_checkpoint;

        let current_checkpoint = minor_checkpoints
            .get(checkpoint_number as usize)
            .expect("Invalid current checkpoint number");
        let next_checkpoint = minor_checkpoints
            .get(checkpoint_number as usize + 1)
            .expect("Invalid next checkpoint number");

        let trackline = next_checkpoint.pos - current_checkpoint.pos;
        let player_relative_location = self.entity_location.position - current_checkpoint.pos;

        let position_within_checkpoint = player_relative_location.project_onto(trackline).length();

        return (
            lap_number,
            zone_number,
            checkpoint_number,
            position_within_checkpoint,
        );
    }
}

pub fn get_player_placement_array(
    players: &[PlayerEntity; 4],
    minor_checkpoints: &Vec<MinorCheckpoint>,
) -> [u8; 4] {
    let mut player_nums_with_scores: Vec<(u8, (u8, u8, u8, f64))> = [0, 1, 2, 3]
        .into_iter()
        .zip(
            players
                .iter()
                .map(|p| p.get_progress_score(minor_checkpoints)),
        )
        .collect();

    // Sort progress scores, priority given to most significant placement measures
    player_nums_with_scores.sort_by(|a, b| {
        let a_arr = [a.1 .0 as f64, a.1 .1 as f64, a.1 .2 as f64, a.1 .3];
        let b_arr = [b.1 .0 as f64, b.1 .1 as f64, b.1 .2 as f64, b.1 .3];

        for index in 0..=3 {
            if a_arr[index] < b_arr[index] {
                return std::cmp::Ordering::Less;
            } else if a_arr[index] > b_arr[index] {
                return std::cmp::Ordering::Greater;
            }
        }
        std::cmp::Ordering::Equal
    });

    return [0, 1, 2, 3].map(|i| player_nums_with_scores.get(i).unwrap().0);
}