use std::collections::HashMap;
use std::time::Instant;

use chariot_core::networking::ws::QuestionBody;
use chariot_core::networking::Uuid;

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
 * 3. GamePhase::PlayingGameState
 *  This is the phase when players will be zooming around and doing stuff. This phase ends when
 *  someone wins, and the server changes state to AllPlayersDone, and sends the AllDone packet.
 *  This phase also features a property called VotingState, which determines if we are waiting for
 *  votes right now or acting on decision to be made
 *
 * 5. GamePhase::AllPlayersDone
 *  Show standings, perhaps a retry button, any other end-of-race stuff
 */

#[derive(Debug)]
pub enum GamePhase {
    WaitingForPlayerReady(WaitingForPlayerReadyState),
    CountingDownToGameStart(CountingDownToGameStartState),
    PlayingGame(PlayingGameState),
    AllPlayersDone(AllPlayersDoneState),
}

#[derive(Debug)]
pub enum VotingState {
    VoteCooldown(Instant), // Instant corresponds to the time we will start waitingforvotes again
    WaitingForVotes(WaitingForVotesState),
    VoteResultActive(i32), // i32 corresponds to the decision that was made (will likely change into a more complex data structure later)
}

#[derive(Debug)]
pub struct WaitingForVotesState {
    pub audience_votes: HashMap<Uuid, i32>,
    pub current_question: QuestionBody,
    pub vote_close_time: Instant,
}

#[derive(Debug)]
pub struct WaitingForPlayerReadyState {
    pub players_ready: [bool; 4],
    pub new_players_joined: Vec<(String, usize)>,
}

#[derive(Debug)]
pub struct CountingDownToGameStartState {
    pub countdown_end_time: Instant,
}

#[derive(Debug)]
pub struct PlayingGameState {
    pub voting_game_state: VotingState,
}

// end deprecated

#[derive(Debug)]
pub struct AllPlayersDoneState {}
