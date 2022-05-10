use std::collections::HashMap;
use std::time::Instant;

use chariot_core::networking::ws::QuestionBody;
use chariot_core::networking::Uuid;

use crate::chairs::get_player_start_physics_properties;

use super::ServerGameState;

pub enum GamePhase {
    // During this phase, players are in selection screens choosing their chair
    // and marking ready; they'll be waiting in UI for all of this phase
    WaitingForPlayerReady,
    // During this phase, players can see the track, the other players, and have
    // a countdown until they'll be able to use controls
    CountingDownToGameStart,
    // The idea we had was to have idk 30 seconds at the start of the race
    // before voting starts; players will be driving around normally
    PlayingBeforeVoting,
    // >:)))
    PlayingWithVoting,
    // Show standings, perhaps a retry button, any other end-of-race stuff
    AllPlayersDone,
}

pub struct WaitingForPlayerReadyState {
    pub players_ready: [bool; 4],
    pub new_players_joined: Vec<usize>,
}

pub struct CountingDownToGameStartState {
    pub countdown_end_time: Instant,
}

pub struct PlayingBeforeVotingState {
    pub voting_start_time: Instant,
}

pub struct PlayingWithVotingState {
    pub audience_votes: HashMap<Uuid, i32>,
    pub current_question: QuestionBody,
    pub is_voting_ongoing: bool,
    pub vote_close_time: Instant,
}

pub struct AllPlayersDoneState {}

pub fn get_starting_server_state() -> ServerGameState {
    ServerGameState {
        phase: GamePhase::WaitingForPlayerReady,
        players: [0, 1, 2, 3]
            .map(|num| get_player_start_physics_properties(&String::from("standard"), num)),
        waiting_for_player_ready_state: WaitingForPlayerReadyState {
            players_ready: [false, false, false, false],
            new_players_joined: Vec::new(),
        },
        counting_down_to_game_start_state: CountingDownToGameStartState {
            countdown_end_time: Instant::now(),
        },
        playing_before_voting_state: PlayingBeforeVotingState {
            voting_start_time: Instant::now(),
        },
        playing_with_voting_state: PlayingWithVotingState {
            audience_votes: HashMap::new(),
            current_question: (
                "q".to_string(),
                (
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string(),
                ),
            ),
            is_voting_ongoing: false,
            vote_close_time: Instant::now(),
        },
        all_players_done_state: AllPlayersDoneState {},
    }
}
