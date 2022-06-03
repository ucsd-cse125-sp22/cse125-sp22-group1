use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
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

    // Non-stats physics
    ShoppingCart,      // Players drift right when not turning
    SpeedBalanceBoost, // Everyone except the first-place player gets 1.5x speed

    // Physics stats
    DoubleMaxSpeed,            // Players can go up to double starting maximum speed
    SuperAccelerator,          // Players accelerate 3 times as fast
    SuperSpin,                 // Players spin 5 times as fast
    MoonGravity,               // Gravity is 0.25 as much
    IceRink,                   // No more rolling resistance
    ExplosivePlayerCollisions, // Collisions with players have 3x more of an effect
    SuperBouncyObjects,        // Collisions with objects have 3x more of an effect

    // One-time events
    SwapFirstAndLast,       // First and last player switch places
    ShufflePlayerPositions, // All players' positions are switched
    RewindLapCounter,       // Change everyone's lap counter back 1 lap
    Backwards,              // Players instantly rotate 180 degrees and have their speed inverted
}

impl AudienceAction {
    pub fn get_description(&self) -> &str {
        match self {
            AudienceAction::Null => "The audience has decided to do nothing. booooo",
            AudienceAction::NoLeft => {
                "If you're nothing without turning left, then you shouldn't have it"
            }
            AudienceAction::NoRight => {
                "If you're nothing without turning left, then you shouldn't have it"
            }
            AudienceAction::InvertControls => "Switch your hands, controls are inverted!",
            AudienceAction::AutoAccelerate => "chair go brrrrrr!! (now with more brrr)",
            AudienceAction::TurnOnlyWhenNotMoving => "You can only turn when you stop moving!",
            AudienceAction::ShoppingCart => "Don't you just love how shopping carts drift?",
            AudienceAction::SpeedBalanceBoost => "Everyone but first place: FULL SPEED AHEAD",
            AudienceAction::DoubleMaxSpeed => "You can go faster now :O",
            AudienceAction::SuperAccelerator => "You can accelerate faster now B)",
            AudienceAction::SuperSpin => "I'll try spinning, that's a cool trick!",
            AudienceAction::MoonGravity => "Fly me to the moon!",
            AudienceAction::IceRink => "It's a great time to go ice skating!",
            AudienceAction::ExplosivePlayerCollisions => "Make sure to practice social distancing!",
            AudienceAction::SuperBouncyObjects => "What if we made everything bouncy?",
            AudienceAction::SwapFirstAndLast => "first and last have been swapped!",
            AudienceAction::ShufflePlayerPositions => "Oops, we seem to have misplaced you all!",
            AudienceAction::RewindLapCounter => "Everyone has been set back one lap!",
            AudienceAction::Backwards => "Whoops! Drive backwards now!",
        }
    }
}

fn get_shuffled_questions() -> Vec<QuestionData> {
    let mut questions = vec![
        QuestionData {
            prompt: "Turning is overrated. Which direction should we ban?".to_string(),
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
        },
        QuestionData {
            prompt: "Let's mess with their controls.".to_string(),
            options: vec![
                QuestionOption {
                    label: "Invert controls".to_string(),
                    action: AudienceAction::InvertControls,
                },
                QuestionOption {
                    label: "Accelerate no matter what".to_string(),
                    action: AudienceAction::AutoAccelerate,
                },
                QuestionOption {
                    label: "Only allow turns when not moving".to_string(),
                    action: AudienceAction::TurnOnlyWhenNotMoving,
                },
            ],
        },
        QuestionData {
            prompt: "Where will we be borrowing some physics from?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Moon gravity (less gravity)".to_string(),
                    action: AudienceAction::MoonGravity,
                },
                QuestionOption {
                    label: "Ice rink (no friction)".to_string(),
                    action: AudienceAction::IceRink,
                },
            ],
        },
        QuestionData {
            prompt: "We love equality! How should we equalize players?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Swap first and last place".to_string(),
                    action: AudienceAction::SwapFirstAndLast,
                },
                QuestionOption {
                    label: "Give a speed boost to everyone except first place".to_string(),
                    action: AudienceAction::SpeedBalanceBoost,
                },
            ],
        },
        QuestionData {
            prompt: "boing boing boing - what should be super bouncy?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Players".to_string(),
                    action: AudienceAction::ExplosivePlayerCollisions,
                },
                QuestionOption {
                    label: "Terrain".to_string(),
                    action: AudienceAction::SuperBouncyObjects,
                },
            ],
        },
        QuestionData {
            prompt: "Time to speed things up a bit. What do we think?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Higher max speed for everyone".to_string(),
                    action: AudienceAction::DoubleMaxSpeed,
                },
                QuestionOption {
                    label: "Faster acceleration for everyone".to_string(),
                    action: AudienceAction::SuperAccelerator,
                },
            ],
        },
        QuestionData {
            prompt: "Spinning is a cool trick! What modification will we engage?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Super spin: turn super fast".to_string(),
                    action: AudienceAction::SuperSpin,
                },
                QuestionOption {
                    label: "Shopping cart mode: perpetual drift to the right".to_string(),
                    action: AudienceAction::ShoppingCart,
                },
            ],
        },
        QuestionData {
            prompt: "Should we shuffle all player positions?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Yes!".to_string(),
                    action: AudienceAction::ShufflePlayerPositions,
                },
                QuestionOption {
                    label: "No!".to_string(),
                    action: AudienceAction::Null,
                },
            ],
        },
        QuestionData {
            prompt: "Throwback time! What should we do?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Set everybody back one lap!".to_string(),
                    action: AudienceAction::RewindLapCounter,
                },
                QuestionOption {
                    label: "Flip everyone backwards".to_string(),
                    action: AudienceAction::Backwards,
                },
            ],
        },
    ];

    questions.shuffle(&mut thread_rng());

    questions
}

lazy_static! {
    pub static ref QUESTIONS: Vec<QuestionData> = get_shuffled_questions();
}
