mod create;
pub mod defs;
mod init;

pub struct MoveGenerator {
    king: [Bitboard; NrOf::SQUARES],
    knight: [Bitboard; NrOf::SQUARES],
    pawns: [[Bitboard; NrOf::SQUARES]; Sides::BOTH],
    rook: Vec<Bitboard>,
    bishop: Vec<Bitboard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
}
