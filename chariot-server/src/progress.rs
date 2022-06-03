use crate::{
    checkpoints::Checkpoint,
    physics::{player_entity::PlayerEntity, trigger_entity::TriggerEntity},
};
use chariot_core::player::{lap_info::*, PlayerID};
use std::cmp::Ordering;
use std::time::Duration;

pub enum PlayerProgress {
    PreGame,
    Racing { lap_info: LapInformation },
    Finished { finish_time: Duration },
}

impl PlayerProgress {
    pub fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PlayerProgress::PreGame, PlayerProgress::PreGame) => Ordering::Equal,
            (
                PlayerProgress::Racing {
                    lap_info: self_lap_info,
                },
                PlayerProgress::Racing {
                    lap_info: other_lap_info,
                },
            ) => {
                if self_lap_info.lap != other_lap_info.lap {
                    self_lap_info.lap.cmp(&other_lap_info.lap)
                } else if self_lap_info.zone != other_lap_info.zone {
                    self_lap_info.zone.cmp(&other_lap_info.zone)
                } else {
                    Ordering::Equal
                }
            }
            (PlayerProgress::Finished { .. }, PlayerProgress::Racing { .. }) => Ordering::Greater,
            (PlayerProgress::Racing { .. }, PlayerProgress::Finished { .. }) => Ordering::Less,
            (
                PlayerProgress::Finished { finish_time },
                PlayerProgress::Finished {
                    finish_time: other_finish_time,
                },
            ) => finish_time.cmp(other_finish_time).reverse(),
            (_, _) => panic!("This comparison shouldn't be possible"),
        }
    }
}
