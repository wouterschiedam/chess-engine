use crate::defs::{Piece, Square};

#[derive(Debug, Default)]
pub struct MoveFlags {
    pub is_capture: bool,
    pub is_promotion: bool,
    pub is_en_passant: bool,
    pub is_castling: bool,
    pub is_double_push: bool,
}

#[derive(Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub capture: Option<Piece>,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags,
}

#[derive(Clone, Copy)]
pub struct MoveInfo {
    pub captured_piece: Option<Piece>,
    pub captured_square: Option<Square>,
    pub previous_castling: u8,
    pub previous_enpassant: Option<u8>,
    pub previous_halfmove_clock: u8,
    pub promotion_piece: Option<Piece>,
    pub rook_from: Option<Square>,
    pub rook_to: Option<Square>,
    pub en_passant_captured_square: Option<Square>,
}
