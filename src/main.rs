use board::{defs::Pieces, Board};
use defs::Sides;
use movegen::{
    defs::{algebraic_from_str, print_bitboard, Move, MoveList, MoveType},
    MoveGenerator,
};
mod board;
mod defs;
mod evaluation;
mod movegen;

mod extra;

fn main() {
    // extra::magics::find_magics(Pieces::BISHOP);

    let mut board = Board::new();
    let _ = board.read_fen(Some(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ));
    let movegen = MoveGenerator::new();
    let mut movelist = MoveList::new();
    movegen.generate_moves(&board, &mut movelist, MoveType::All);
    let x: Vec<&Move> = movelist.moves.iter().filter(|y| y.data > 0).collect();
    println!("{:?}", x);
    //
    // print_bitboard(board.bb_side[Sides::WHITE]);
    // //
    // let x = algebraic_from_str("a2").unwrap();
    //
    // let y = algebraic_from_str("a3").unwrap();
    //
    // board.move_piece(Sides::WHITE, Pieces::PAWN, x, y);
    // print_bitboard(board.bb_side[Sides::WHITE]);
}
