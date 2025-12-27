pub mod engine;
pub mod perft;
pub mod tournament;

use crate::board::Board;
use crate::utils::display::print_position;

pub use perft::run_perft;
pub use tournament::run_tournament;

/// Interactive game loop (stub)
pub fn play_game(starting_fen: Option<&str>, _engine_plays_black: bool, _depth: u8) {
    let _board = Board::build(starting_fen);
}

/// Display a board position
pub fn show_position(fen: Option<&str>) {
    let board = Board::build(fen);

    if let Some(f) = fen {
        println!("FEN: {}", f);
    }

    print_position(&board, false, None);
}
