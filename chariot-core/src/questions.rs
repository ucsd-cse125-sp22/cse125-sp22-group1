use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestionData {
    pub prompt: String,
    pub options: Vec<QuestionOption>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestionOption {
    pub label: String,
    pub action: AudienceAction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AudienceAction {
    Null, // if we want to allow the boring choice ya know

    // For now, we'll only have interactions that are server-side
    // controls/physics manipulations only (no client-side visual effects)

    // Controls
    NoLeft,                // Player can no longer turn left
    NoRight,               // Player can no longer turn right
    InvertControls,        // Brake/accelerate swapped, turn left/right swapped
    AutoAccelerate,        // Players accelerate no matter what
    TurnOnlyWhenNotMoving, // Players can't turn when moving at all

    SwapFirstAndLast,          // First and last player switch places
    ShufflePlayerPositions,    // All players' positions are switched
    DoubleMaxSpeed,            // Players can go up to double starting maximum speed
    SuperAccelerator,          // Players accelerate 3 times as fast
    SuperSpin,                 // Players spin 5 times as fast
    ShoppingCart,              // Players drift right when not turning
    MoonGravity,               // Gravity is 0.25 as much
    IceRink,                   // No more rolling resistance
    ExplosivePlayerCollisions, // Collisions with players have 3x more of an effect
    SuperBouncyObjects,        // Collisions with objects have 3x more of an effect
    SpeedBalanceBoost,         // Everyone except the first-place player gets 1.5x speed
    ResetLapCounter,           // Change everyone's lap counter back to 1
    Backwards,                 // Players instantly rotate 180 degrees and have their speed inverted
}

lazy_static! {
    pub static ref QUESTIONS: Vec<QuestionData> = vec![QuestionData {
        prompt: "Q1 Turning is overrated. Which direction should we ban?".to_string(),
        options: vec![
            QuestionOption {
                label: "Left".to_string(),
                action: AudienceAction::NoLeft,
            },
            QuestionOption {
                label: "Right".to_string(),
                action: AudienceAction::NoRight,
            },
        ],
    }];
}
