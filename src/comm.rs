use std::sync::{Arc, Mutex};

use crossbeam_channel::Sender;

use crate::{
    board::Board,
    engine::defs::Information,
    movegen::defs::Move,
    search::defs::{PerftSummary, SearchCurrentMove, SearchStats, SearchSummary},
};

use self::uci::UciReport;

pub mod uci;

// Defines the public functions a Comm module must implement.
pub trait IComm {
    fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        // options: Arc<Vec<EngineOption>>,
    );
    fn send(&self, msg: CommControl);
    fn wait_for_shutdown(&mut self);
    fn get_protocol_name(&self) -> &'static str;
}

#[derive(PartialEq, Debug)]
pub enum CommControl {
    // Reactions of engine to incoming commands.
    Update,                            // Request Comm module to update its state.
    Quit,                              // Quit the Comm module.
    Identify,                          // Transmit identification of the engine.
    Ready,                             // Transmit that the engine is ready.
    SearchSummary(SearchSummary),      // Transmit search information.
    SearchCurrMove(SearchCurrentMove), // Transmit currently considered move.
    SearchStats(SearchStats),          // Transmit search Statistics.
    InfoString(String),                // Transmit general information.
    BestMove(Move),                    // Transmit the engine's best move.
    PerftScore(PerftSummary),          // Transmit perft score
    SolvePuzzles,
    // Output to screen when running in a terminal window.
    PrintBoard,   // PrintBoard,
    PrintHistory, // PrintHistory,
    PrintHelp,    // PrintHelp,
}

// These are the commands a Comm module can create and send back to the
// engine in the main thread.
#[derive(PartialEq, Clone, Debug)]
pub enum CommReport {
    Uci(UciReport),
}

impl CommReport {
    pub fn is_valid(&self) -> bool {
        true
    }
}
