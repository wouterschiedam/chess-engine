mod create;
pub mod defs;
mod init;
mod magics;

use crate::{
    board::defs::{Pieces, Squares, BB_RANKS, BB_SQUARES},
    defs::{Bitboard, Castling, NrOf, Piece, Side, Sides, Square, EMPTY},
};

use crate::movegen::magics::Magic;

const PROMOTION_PIECES: [usize; 4] = [Pieces::QUEEN, Pieces::ROOK, Pieces::KNIGHT, Pieces::BISHOP];

pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.

pub struct MoveGenerator {
    pub king: [Bitboard; NrOf::SQUARES],
    pub knight: [Bitboard; NrOf::SQUARES],
    pub pawns: [[Bitboard; NrOf::SQUARES]; Sides::BOTH],
    pub rook: Vec<Bitboard>,
    pub bishop: Vec<Bitboard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
}

impl MoveGenerator {
    pub fn new() -> Self {
        let magics: Magic = Default::default();

        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NrOf::SQUARES],
            bishop_magics: [magics; NrOf::SQUARES],
        };

        mg.init_king_attack();
        mg.init_knight_attack();
        mg.init_pawn_attack();
        mg.init_magics(Pieces::ROOK);
        mg.init_magics(Pieces::BISHOP);
        mg
    }
}
