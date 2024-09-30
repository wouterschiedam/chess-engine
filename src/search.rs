use std::{
    env,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender};
use defs::SearchType;

use crate::{
    board::Board,
    engine::{
        defs::Information,
        transposition::{SearchData, TT},
    },
    movegen::MoveGenerator,
};

use self::defs::{
    SearchControl, SearchInfo, SearchParams, SearchRefs, SearchReport, SearchTerminate,
};

mod alpha_beta;
mod bestmove;
pub mod defs;
mod helpers;
pub mod search_routine;
mod sorting;
pub mod time;
pub mod utils;

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle: None,
            control_tx: None,
        }
    }

    pub fn init(
        &mut self,
        report_tx: Sender<Information>, // Used to send information to engine.
        board: Arc<Mutex<Board>>,       // Arc pointer to engine's board.
        mg: Arc<MoveGenerator>,         // Arc pointer to engine's move generator.
        tt: Arc<Mutex<TT<SearchData>>>,
        tt_enabled: bool,
    ) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables.
        let t_report_tx = report_tx;

        // Create the search thread.
        let h = thread::spawn(move || {
            // Create thread-local variables.
            let arc_board = Arc::clone(&board);
            let arc_mg = Arc::clone(&mg);
            let arc_tt = Arc::clone(&tt);
            // let arc_tt = Arc::clone(&tt);
            let mut search_params = SearchParams::new();
            let mut search_type = SearchType::Nothing;

            let mut quit = false;
            let mut halt = true;

            while !quit {
                // Wait for the next incoming command from the engine.
                let cmd = control_rx.recv().expect("Channel error");

                // And react accordingly.
                match cmd {
                    SearchControl::Start(sp, st) => {
                        search_params = sp;
                        search_type = st;
                        halt = false; // This will start the search.
                    }
                    SearchControl::Stop => halt = true,
                    SearchControl::Quit => quit = true,
                    SearchControl::Nothing => (),
                }

                // Search isn't halted and not going to quit.
                if !halt && !quit {
                    match search_type {
                        SearchType::Search => {
                            Search::search_best_move(
                                halt,
                                quit,
                                &arc_board,
                                &arc_mg,
                                &arc_tt,
                                tt_enabled,
                                &control_rx,
                                &t_report_tx,
                                search_params,
                            );
                        }
                        SearchType::Perft => {
                            Search::perft_score(
                                &arc_board,
                                &arc_mg,
                                &t_report_tx,
                                search_params.depth,
                            );
                        }
                        _ => (),
                    }
                }
            }
        });

        // Store the thread's handle and command sender.
        self.handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    // This function is used to send commands into the search thread.
    pub fn send(&self, cmd: SearchControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(cmd).expect("Thread failed");
        }
    }

    // After sending the quit command, the engine calls this function to
    // wait for the search to shut down.
    pub fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect("Thread failed");
        }
    }

    pub fn perft_score(
        board: &Arc<Mutex<Board>>,
        arc_mg: &Arc<MoveGenerator>,
        t_report_tx: &Sender<Information>,
        depth: i8,
    ) {
        let mtx_board = board.lock().expect("lock failed");
        let board = mtx_board.clone();
        std::mem::drop(mtx_board);

        let perft_summary = MoveGenerator::go_perft_results(board, depth, arc_mg);

        // Inform the engine that the search has finished.
        let information = Information::Search(SearchReport::PerftScore(perft_summary));
        t_report_tx.send(information).expect("channel failed");
    }

    pub fn search_best_move(
        mut halt: bool,
        mut quit: bool,
        board: &Arc<Mutex<Board>>,
        arc_mg: &Arc<MoveGenerator>,
        arc_tt: &Arc<Mutex<TT<SearchData>>>,
        tt_enabled: bool,
        control_rx: &Receiver<SearchControl>,
        t_report_tx: &Sender<Information>,
        mut search_params: SearchParams,
    ) {
        // Copy the current board to be used in this thread.
        let mtx_board = board.lock().expect("lock failed");
        let mut board = mtx_board.clone();
        std::mem::drop(mtx_board);

        // Create a place to put search information
        let mut search_info = SearchInfo::new();

        // Create references to all needed information and structures.
        let path = env::current_dir().unwrap();
        let formatted_path = format!("{}/../book.txt", path.display());
        let mut search_refs = SearchRefs {
            board: &mut board,
            move_generator: &arc_mg,
            tt: &arc_tt,
            tt_enabled,
            search_info: &mut search_info,
            search_params: &mut search_params,
            control_rx: &control_rx,
            report_tx: &t_report_tx,
            book: &Search::load_book(&formatted_path),
        };

        // Start the search using Iterative Deepening.
        let (best_move, terminate) = Search::search_routine(&mut search_refs);

        // Inform the engine that the search has finished.
        let information = Information::Search(SearchReport::Finished(best_move));
        t_report_tx.send(information).expect("channel failed");

        // If the search was finished due to a Stop or Quit
        // command then either halt or quit the search.
        match terminate {
            SearchTerminate::Stop => {
                halt = true;
            }
            SearchTerminate::Quit => {
                halt = true;
                quit = true;
            }
            SearchTerminate::Nothing => (),
        }
    }
}
