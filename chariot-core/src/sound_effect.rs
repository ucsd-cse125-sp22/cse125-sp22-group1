use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum SoundEffect {
    EnterChairacterSelect,
    SelectChairacter,
    ReadyUp,

    GameStart,
    NextLap,
    GameEnd,

    PlayerCollision,
    TerrainCollision,

    InteractionVoteStart,
    InteractionChosen,
}
