use crate::board::Board;
use crate::utils::display::print_position;

/// Interactive game loop
pub fn play_game(starting_fen: Option<&str>, _engine_plays_black: bool, _depth: u8) {
    // Initialize your board type here
    let _board = Board::build(starting_fen);
    // TODO: Implement game loop
}

/// Display a board position
pub fn show_position(fen: Option<&str>) {
    let board = Board::build(fen);

    if let Some(f) = fen {
        println!("FEN: {}", f);
    }

    print_position(&board, false, None);
}

