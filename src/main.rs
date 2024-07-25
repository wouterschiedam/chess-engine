use std::thread;

use board::defs::SQUARE_NAME;
use engine::Engine;
use extra::parse::algebraic_move_to_number;
use movegen::defs::Move;
use ui::ui::run;
mod board;
mod comm;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod movegen;
mod search;
pub mod ui;

fn main() {
    // Start thread for the chess engine
    let mut engine = Engine::new();
    let _ = engine.run();
}
