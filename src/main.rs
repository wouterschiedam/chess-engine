use board::defs::Pieces;
use movegen::{defs::print_bitboard, MoveGenerator};
mod board;
mod defs;
mod movegen;

mod extra;

fn main() {
    let movegen = MoveGenerator::new();

    extra::magics::find_magics(Pieces::BISHOP);
    // print_bitboard(movegen.knight[42]);
}
