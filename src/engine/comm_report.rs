use crate::{
    comm::{uci::UciReport, CommControl, CommReport}, defs::FEN_START_POSITION, puzzle::Puzzle, search::defs::{SearchControl, SearchMode, SearchParams, SearchRefs, SearchType}
};
use std::env;
use std::fs::File;
use std::error::Error;

use super::Engine;

impl Engine {
    pub fn comm_reports(&mut self, comm: &CommReport) {
        match comm {
            CommReport::Uci(c) => self.comm_report_uci(c),
        }
    }

    fn comm_report_uci(&mut self, ucireport: &UciReport) {
        let mut sp = SearchParams::new();
        sp.quiet = self.settings.quiet;
        match ucireport {
            UciReport::Uci => {
                self.comm.send(CommControl::Identify);
            }

            UciReport::UciNewGame => {
                let _ = self
                    .board
                    .lock()
                    .expect("error locking board")
                    .read_fen(Some(FEN_START_POSITION));
            }
            UciReport::IsReady => {
                self.comm.send(CommControl::Ready);
            }

            // UciReport::SetOption(option) => (),
            UciReport::Position(fen, moves) => {
                let fen_result = self
                    .board
                    .lock()
                    .expect("Error locking board")
                    .read_fen(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            let msg = format!("{}: {}", m, "illigal move");
                            self.comm.send(CommControl::InfoString(msg));
                            break;
                        }
                    }
                }
            }
            UciReport::GoInfinite => {
                sp.search_mode = SearchMode::Infinite;
                self.search
                    .send(SearchControl::Start(sp, SearchType::Search));
            }
            UciReport::GoDepth(depth) => {
                sp.depth = *depth;
                sp.search_mode = SearchMode::Depth;
                self.search
                    .send(SearchControl::Start(sp, SearchType::Search));
            }
            UciReport::GoPerft(depth) => {
                sp.depth = *depth;
                sp.search_mode = SearchMode::Depth;
                self.search
                    .send(SearchControl::Start(sp, SearchType::Perft));
            }
            UciReport::GoMoveTime(t) => {
                sp.move_time = *t;
                sp.search_mode = SearchMode::MoveTime;
                self.search
                    .send(SearchControl::Start(sp, SearchType::Search));
            }

            UciReport::GoNodes(nodes) => {
                sp.nodes = *nodes;
                sp.search_mode = SearchMode::Nodes;
                self.search
                    .send(SearchControl::Start(sp, SearchType::Search));
            }

            UciReport::GoGameTime(gt) => {
                sp.game_time = *gt;
                sp.search_mode = SearchMode::GameTime;
                self.search
                    .send(SearchControl::Start(sp, SearchType::Search));
            }

            UciReport::Puzzle => {
                let path = env::current_dir().unwrap();
                let formatted_path = format!("{}/../sorted_puzzles.csv", path.display());

                match Puzzle::read_puzzles_from_csv(&formatted_path) {
                    Ok(puzzles) => {
                        for puzzle in puzzles {
                            self.solve_puzzle(puzzle, sp);
                        }
                    }
                    Err(e) => println!("Error reading puzzles from file: {}", e),
                };


            },

            UciReport::Quit => self.quit(),
            UciReport::Stop => self.search.send(SearchControl::Stop),
            UciReport::Board => self.comm.send(CommControl::PrintBoard),
            UciReport::Unknown => (),
        }
    }
}
