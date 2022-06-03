use crate::{
    checkpoints::Checkpoint,
    physics::{player_entity::PlayerEntity, trigger_entity::TriggerEntity},
};
use chariot_core::player::{lap_info::*, PlayerID};
use std::time::Duration;

enum PlayerProgress {
    Racing { lap_info: LapInformation },
    Finished { finish_time: Duration },
}

impl PlayerProgress {}
