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
    NoLeft,
    NoRight,
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
