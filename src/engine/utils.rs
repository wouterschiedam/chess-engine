use std::sync::{Arc, Mutex};
use std::fs::File;
use std::error::Error;

use crate::search::defs::{SearchControl, SearchParams, SearchType};
use crate::{
    puzzle::Puzzle,
    search::Search,
    board::Board,
    comm::CommControl,
    defs::{EngineRunResult, FEN_KIWIPETE_POSITION},
    extra::parse::{algebraic_move_to_number, PotentialMove},
    movegen::{
        defs::{Move, MoveList, MoveType},
        MoveGenerator, MoveStats,
    },
};

use super::{defs::Information, Engine};

impl Engine {
    pub fn main_loop(&mut self) {
        // Get receiver for incoming messages
        let (info_sender, info_receiver) = crossbeam_channel::unbounded::<Information>();

        // store receiver in the Engine
        self.info_receiver = Some(info_receiver);

        // init communication
        self.comm.init(info_sender.clone(), Arc::clone(&self.board));

        // init search
        self.search.init(
            info_sender.clone(),
            Arc::clone(&self.board),
            Arc::clone(&self.movegen),
            Arc::clone(&self.tt_search),
            self.settings.tt_size > 0,
        );
        // update Comm interface
        self.comm.send(CommControl::Update);

        while !self.quit {
            let info = &self.get_info_receiver();
            match info {
                Information::Comm(cr) => self.comm_reports(cr),
                Information::Search(sr) => self.search_report(sr),
            }
        }

        // main loop ended shut down threads
        self.comm.wait_for_shutdown();
        self.search.wait_for_shutdown();
    }

    // This is the main engine thread Information receiver.
    fn get_info_receiver(&mut self) -> Information {
        match &self.info_receiver {
            Some(i) => i.recv().expect("error receiving message"),
            None => panic!("{}", "Error receiving"),
        }
    }

    pub fn setup_position(&mut self) -> EngineRunResult {
        let fen = &self.cmdline.fen()[..];
        let k = self.cmdline.has_kiwipete();

        let new_fen = if k { FEN_KIWIPETE_POSITION } else { &fen };

        let _ = self
            .board
            .lock()
            .expect("error locking board to setup fen string")
            .read_fen(Some(&new_fen));

        Ok(())
    }

    pub fn execute_move(&mut self, m: String) -> bool {
        let empty = (0, 0, 0);

        let potential_move = algebraic_move_to_number(&m[..]).unwrap_or(empty);
        let is_psuedo_legal = self.psuedo_legal(potential_move, &self.board, &self.movegen);

        let mut is_legal = false;

        if let Ok(ips) = is_psuedo_legal {
            is_legal = self
                .board
                .lock()
                .expect("error locking board")
                .make_move(ips, &self.movegen);
        }
        is_legal
    }

    pub fn psuedo_legal(
        &self,
        pm: PotentialMove,
        board: &Mutex<Board>,
        movegen: &MoveGenerator,
    ) -> Result<Move, ()> {
        let mut result = Err(());
        let mut movelist = MoveList::new();
        let mut move_stats = MoveStats::new();

        let mutex_board = board.lock().expect("error locking board");

        movegen.generate_moves(&mutex_board, &mut movelist, MoveType::All, &mut move_stats);
        // we dont need that sheit anymore
        std::mem::drop(mutex_board);

        // Determine if the potential move is pseudo-legal. make() wil
        // determine final legality when executing the move.
        for i in 0..movelist.len() {
            let current = movelist.get_move(i);
            if pm.0 == current.from() {
                if pm.1 == current.to() {
                    if pm.2 == current.promoted() {
                        result = Ok(current);
                        break;
                    }
                }
            }
        }
        result
    }

    /// Function to solve puzzles and log results to a file
    pub fn solve_puzzle(&mut self, puzzle: Puzzle,  sp: SearchParams) -> Result<(), ()> {
        
        // Set up the board with the initial FEN position
        self.board.lock().expect("error locking board").read_fen(Some(&puzzle.fen));

        self.search.send(SearchControl::Start(sp, SearchType::Search));

        // // Log the result
        // if solved {
        //     writeln!(writer, "Puzzle {} solved correctly: {:?}", i + 1, calculated_moves)?;
        // } else {
        //     writeln!(writer, "Puzzle {} failed: expected {:?}, got {:?}", i + 1, puzzle.solution_moves, calculated_moves)?;
        //     writeln!(writer, "FEN: {}", locked_board.create_fen());
        // }

        Ok(())
    }
}
