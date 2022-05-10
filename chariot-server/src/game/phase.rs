use std::collections::HashMap;
use std::time::Instant;

use chariot_core::networking::ws::QuestionBody;
use chariot_core::networking::Uuid;

use crate::chairs::get_player_start_physics_properties;

use super::ServerGameState;

/*
 * Phases of the game are as follows:
*
 * 1. GamePhase::WaitingForPlayerReady
 *  During this phase, players are in selection screens choosing their chair
 *  and marking ready; they'll be waiting in UI for all of this phase. The
 *  transition from Phase 1 to 2 occurs when the server sends all clients an
 *  GameStart packet, which will include a time to transition from phase 2
 *  to 3.
 *
 * 2. GamePhase::CountingDownToGameStart
 *  During this phase, players can see the track, the other players, and have
 *  a countdown until they'll be able to use controls. The transition from Phase
 *  2 to 3 is timed by the server, synchronized by an earlier GameStart packet.
 *
 * 3. GamePhase::PlayingBeforeVoting
 *  The idea we had was to have idk 30 seconds at the start of the race
 *  before voting starts; players will be driving around normally and the
 *  clients will be oblivious to the difference between Phase 3 and 4. (client
 *  will time itself internally)
 *
 * 4. GamePhase::PlayingWithVoting
 *  All the good stuff happening at once! >:))) The transition from Phase 4 to
 *  Phase 5 is marked by the server's AllDone packet.
 *
 * 5. GamePhase::AllPlayersDone
 *  Show standings, perhaps a retry button, any other end-of-race stuff
 */

pub enum GamePhase {
    WaitingForPlayerReady,
    CountingDownToGameStart,
    PlayingBeforeVoting,
    PlayingWithVoting,
    AllPlayersDone,
}

pub struct WaitingForPlayerReadyState {
    pub players_ready: [bool; 4],
    pub new_players_joined: Vec<(String, usize)>,
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
