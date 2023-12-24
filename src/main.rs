use engine::Engine;
mod board;
mod comm;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod movegen;
mod search;

fn main() {
    // extra::magics::find_magics(Pieces::BISHOP);
    let mut engine = Engine::new();
    //
    // let mut t_incoming_data = String::from("");
    // io::stdin().read_line(&mut t_incoming_data);
    let x = engine.run();
    // let mut board = Board::new();
    // let _ = board.read_fen(Some(
    //     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    // ));
    // print_bitboard(board.bb_side[Sides::BLACK]);
    // let mut search_info = SearchInfo::new();
    // search_info.depth = 3;
    // let movegen = MoveGenerator::new();
    // let mut search_refs = SearchRefs {
    //     board: &mut board,
    //     move_generator: &movegen,
    //     search_info: &mut search_info,
    // };
    //
    // let (best_move, _interupted) = Search::search_routine(&mut search_refs);
    // move_data(best_move, 0);
    // print_bitboard(search_refs.board.bb_side[Sides::WHITE]);
    //
    // let (best_move, _interupted) = Search::search_routine(&mut search_refs);
    // move_data(best_move, 0);
    // print_bitboard(search_refs.board.bb_side[Sides::BLACK]);
}
