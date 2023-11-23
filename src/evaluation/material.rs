use super::defs::PIECE_VALUES;
use crate::{board::Board, defs::Sides, extra::bits};

pub fn count(board: &Board) -> (u16, u16) {
    let mut white_material = 0;
    let mut black_material = 0;

    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        let mut white_pieces = *w;
        let mut black_pieces = *b;
        while white_pieces > 0 {
            white_material += PIECE_VALUES[piece];
            bits::next(&mut white_pieces);
        }

        while black_pieces > 0 {
            black_material += PIECE_VALUES[piece];
            bits::next(&mut black_pieces);
        }
    }

    (white_material, black_material)
}
