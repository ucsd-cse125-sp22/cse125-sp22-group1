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
    NoLeft,
    NoRight,
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
