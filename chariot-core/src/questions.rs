use crate::GLOBAL_CONFIG;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    NoLeft,                    // Player can no longer turn left
    NoRight,                   // Player can no longer turn right
    InvertControls,            // Brake/accelerate swapped, turn left/right swapped
    SwapFirstAndLast,          // First and last player switch places
    ShufflePlayerPositions,    // All players' positions are switched
    DoubleMaxSpeed,            // Players can go up to double starting maximum speed
    SuperAccelerator,          // Players accelerate 3 times as fast
    SuperSpin,                 // Players spin 5 times as fast
    AutoAccelerate,            // Players accelerate no matter what
    ShoppingCart,              // Players drift right when not turning
    MoonGravity,               // Gravity is 0.25 as much
    IceRink,                   // No more rolling resistance or air resistance
    ExplosivePlayerCollisions, // Collisions with players have 3x more of an effect
    SuperBouncyObjects,        // Collisions with objects have 3x more of an effect
    SpeedBalanceBoost,         // Everyone except the first-place player gets 1.5x speed
    ResetLapCounter,           // Change everyone's lap counter back to 1
    TurnOnlyWhenNotMoving,     // Players can't turn when moving at all
    Backwards,                 // Players instantly rotate 180 degrees and have their speed inverted
}

pub fn load_questions() -> Vec<QuestionData> {
    // get questions file
    let questions_yaml_path = PathBuf::from(&GLOBAL_CONFIG.resource_folder).join("questions.yaml");

    let f = std::fs::File::open(questions_yaml_path).expect("Should have a questions.yaml file!");
    let q_data: Vec<QuestionData> =
        serde_yaml::from_reader(f).expect("should be able to read yaml file!");
    let mut questions = q_data.to_vec();
    questions.shuffle(&mut thread_rng());
    return questions;
}

lazy_static! {
    pub static ref QUESTIONS: Vec<QuestionData> = load_questions();
}
