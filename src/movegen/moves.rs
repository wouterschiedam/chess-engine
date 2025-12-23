use crate::defs::{Piece, Square};

pub struct MoveFlags {
    pub is_capture: bool,
    pub is_promotion: bool,
    pub is_en_passant: bool,
    pub is_castling: bool,
    pub is_double_push: bool,
}

pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub capture: Option<Piece>,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags
}
