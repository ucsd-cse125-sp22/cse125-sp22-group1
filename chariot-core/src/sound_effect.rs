use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum SoundEffect {
    GameStart,
    NextLap,
    GameEnd,

    PlayerCollision,
    TerrainCollision,

    InteractionVoteStart,
    InteractionChosen,
}
