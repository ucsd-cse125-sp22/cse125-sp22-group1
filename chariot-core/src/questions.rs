use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Questions {
    pub questions: Vec<QuestionData>,
}

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

impl Questions {
    fn load_questions() -> Questions {
        // get questions file
        let questions_yaml_path = format!(
            "{}/chariot-core/questions.yaml",
            env::current_dir() // gets the path to the root directory for chariot
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
        );

        let f =
            std::fs::File::open(questions_yaml_path).expect("Should have a questions.yaml file!");
        let q_data: Questions =
            serde_yaml::from_reader(f).expect("should be able to read yaml file!");
        let mut questions = q_data.questions.to_vec();
        questions.shuffle(&mut thread_rng());
        return Questions { questions };
    }
}

lazy_static! {
    pub static ref QUESTIONS: Questions = Questions::load_questions();
}
