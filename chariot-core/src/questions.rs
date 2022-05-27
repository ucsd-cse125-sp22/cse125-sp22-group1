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
    ResetLapCounter,        // Change everyone's lap counter back to 1
    Backwards,              // Players instantly rotate 180 degrees and have their speed inverted
}

lazy_static! {
    pub static ref QUESTIONS: Vec<QuestionData> = vec![
        QuestionData {
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
        },
        QuestionData {
            prompt: "Q2 Let's mess with their controls.".to_string(),
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
                }
            ]
        },
        QuestionData {
            prompt: "Q3 Where will we be borrowing some physics from?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Moon gravity (less gravity)".to_string(),
                    action: AudienceAction::MoonGravity,
                },
                QuestionOption {
                    label: "Ice rink (no friction)".to_string(),
                    action: AudienceAction::IceRink,
                }
            ]
        },
        QuestionData {
            prompt: "Q4 We love equality! How should we equalize players?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Swap first and last place".to_string(),
                    action: AudienceAction::SwapFirstAndLast,
                },
                QuestionOption {
                    label: "Give a speed boost to everyone except first place".to_string(),
                    action: AudienceAction::SpeedBalanceBoost,
                }
            ]
        },
        QuestionData {
            prompt: "Q5 boing boing boing - what should be super bouncy?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Players".to_string(),
                    action: AudienceAction::ExplosivePlayerCollisions,
                },
                QuestionOption {
                    label: "Terrain".to_string(),
                    action: AudienceAction::SuperBouncyObjects,
                }
            ]
        },
        QuestionData {
            prompt: "Q6 Time to speed things up a bit. What do we think?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Higher max speed for everyone".to_string(),
                    action: AudienceAction::DoubleMaxSpeed,
                },
                QuestionOption {
                    label: "Faster acceleration for everyone".to_string(),
                    action: AudienceAction::SuperAccelerator,
                }
            ]
        },
        QuestionData {
            prompt: "Q7 Spinning is a cool trick! What modification will we engage?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Super spin: turn super fast".to_string(),
                    action: AudienceAction::SuperSpin,
                },
                QuestionOption {
                    label: "Shopping cart mode: perpetual drift to the right".to_string(),
                    action: AudienceAction::ShoppingCart,
                }
            ]
        },
        QuestionData {
            prompt: "Q8 Should we shuffle all player positions?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Yes!".to_string(),
                    action: AudienceAction::ShufflePlayerPositions,
                },
                QuestionOption {
                    label: "No!".to_string(),
                    action: AudienceAction::Null,
                }
            ]
        },
        QuestionData {
            prompt: "Q9 Throwback time! What should we do?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Reset everyone to lap 1".to_string(),
                    action: AudienceAction::ResetLapCounter,
                },
                QuestionOption {
                    label: "Flip everyone backwards".to_string(),
                    action: AudienceAction::Backwards,
                }
            ]
        },
    ];
}
