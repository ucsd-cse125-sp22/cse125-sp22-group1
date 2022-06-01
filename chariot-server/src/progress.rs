use crate::{
    checkpoints::Checkpoint,
    physics::{player_entity::PlayerEntity, trigger_entity::TriggerEntity},
};
use chariot_core::player::{lap_info::*, PlayerID};

impl PlayerEntity {
    // Values returned aren't intended to be interpreted directly, only compared
    fn get_progress_score(
        &self,
        checkpoints: &Vec<Checkpoint>,
    ) -> (LapNumber, ZoneID, CheckpointID, f64) {
        // There's a hierarchy of four pieces of progress information we have:
        // from least to most granular lap number, zone number (zones are
        // between major checkpoints), minor checkpoint number, and linear
        // interpolation between minor checkpoint
        let lap_number = self.lap_info.lap;
        let zone_number = self.lap_info.zone;
        let checkpoint_number = self.lap_info.last_checkpoint;
        let mut position_within_checkpoint = 0.0;

        if checkpoints.len() > 0 {
            let current_checkpoint = checkpoints
                .get(checkpoint_number as usize)
                .expect("Invalid current checkpoint number");
            let next_checkpoint = checkpoints
                .get((checkpoint_number as usize + 1) % checkpoints.len())
                .expect("Invalid next checkpoint number");

            let trackline = next_checkpoint.pos() - current_checkpoint.pos();
            let mut player_relative_location =
                self.entity_location.position - current_checkpoint.pos();
            player_relative_location[1] = trackline[1]; // Ignore the y component for now, if we add vertical maps later we should remove this maybe?
            position_within_checkpoint = player_relative_location.project_onto(trackline).length();
        }

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
    checkpoints: &Vec<Checkpoint>,
) -> [(usize, LapInformation); 4] {
    let mut player_nums_with_scores: Vec<(PlayerID, (LapNumber, ZoneID, CheckpointID, f64))> =
        [0, 1, 2, 3]
            .into_iter()
            .zip(players.iter().map(|p| p.get_progress_score(checkpoints)))
            .collect();

    // Sort progress scores, priority given to most significant placement measures
    player_nums_with_scores.sort_by(|a, b| {
        let a_arr = [a.1 .0 as f64, a.1 .1 as f64, a.1 .2 as f64, a.1 .3];
        let a_finished = players[a.0].lap_info.finished;
        let b_arr = [b.1 .0 as f64, b.1 .1 as f64, b.1 .2 as f64, b.1 .3];
        let b_finished = players[b.0].lap_info.finished;

        if a_finished || b_finished {
            if a_finished && b_finished {
                if players[a.0].lap_info.placement < players[b.0].lap_info.placement {
                    return std::cmp::Ordering::Less;
                } else {
                    return std::cmp::Ordering::Greater;
                }
            } else {
                if a_finished {
                    return std::cmp::Ordering::Greater;
                } else if b_finished {
                    return std::cmp::Ordering::Less;
                }
            }
        } else {
            for index in 0..=3 {
                if a_arr[index] < b_arr[index] {
                    return std::cmp::Ordering::Less;
                } else if a_arr[index] > b_arr[index] {
                    return std::cmp::Ordering::Greater;
                }
            }
        }
        std::cmp::Ordering::Equal
    });

    return [0, 1, 2, 3].map(|i| {
        let data = player_nums_with_scores.get(i).unwrap();
        if players[data.0].lap_info.finished {
            (data.0, players[data.0].lap_info)
        } else {
            (
                data.0,
                LapInformation {
                    lap: data.1 .0,
                    zone: data.1 .1,
                    last_checkpoint: data.1 .2,
                    placement: 4 - i as Placement,
                    finished: false,
                },
            )
        }
    });
}
