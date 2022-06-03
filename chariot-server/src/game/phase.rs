use std::collections::HashMap;
use std::time::Instant;

use chariot_core::networking::Uuid;
use chariot_core::player::choices::PlayerChoices;
use chariot_core::player::lap_info::Placement;
use chariot_core::questions::{QuestionData, QuestionOption};

use super::voting::{AnswerID, QuestionID};

/*
 * Phases of the game are as follows:
*
 * 1. GamePhase::ChoosingSettingsAndConnecting
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

pub enum GamePhase {
    // Choosing the chair/map
    ConnectingAndChoosingSettings {
        force_start: bool,
        player_choices: [Option<PlayerChoices>; 4],
    },
    // Players notified about map/chairs, loading in
    WaitingForPlayerLoad {
        players_loaded: [bool; 4],
    },
    // All players loaded, race counting down
    CountingDownToGameStart(Instant),
    // Game is playing
    PlayingGame {
        voting_game_state: VotingState,
        question_idx: QuestionID, // to keep track of which question we have asked
    },
    // Everyone has finished racing
    AllPlayersDone([(Placement, (u64, u32)); 4]),
}

pub enum VotingState {
    VoteCooldown(Instant), // Instant corresponds to the time we will start waitingforvotes again
    WaitingForVotes {
        audience_votes: HashMap<Uuid, AnswerID>,
        current_question: QuestionData,
        vote_close_time: Instant,
    },
    VoteResultActive {
        decision: QuestionOption,
        decision_end_time: Instant,
    },
}
