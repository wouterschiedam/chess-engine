pub const NOT_A_FILE: u64 = 18374403900871474942;

pub const NOT_H_FILE: u64 = 9187201950435737471;

pub const NOT_AB_FILE: u64 = 18229723555195321596;

pub const NOT_HG_FILE: u64 = 4557430888798830399;

pub use super::{magics::Magic, movelist::MoveList};
use crate::{
    board::defs::{PIECE_CHAR_CAPS, PIECE_CHAR_SMALL, PIECE_NAME, SQUARE_NAME},
    defs::{Castling, Piece, Square},
};

const MOVE_ONLY: usize = 0x00_00_00_00_00_FF_FF_FF;

pub struct Shift;
impl Shift {
    pub const PIECE: usize = 0;
    pub const FROM_SQ: usize = 3;
    pub const TO_SQ: usize = 9;
    pub const CAPTURE: usize = 15;
    pub const PROMOTION: usize = 18;
    pub const EN_PASSANT: usize = 21;
    pub const DOUBLE_STEP: usize = 22;
    pub const CASTLING: usize = 23;
    pub const SORTSCORE: usize = 24;
}

pub struct Mask;
impl Mask {
    pub const PIECE: usize = 0x3F;
    pub const FROM_SQ: usize = 0x3F << Shift::FROM_SQ;
    pub const TO_SQ: usize = 0x3F << Shift::TO_SQ;
    pub const CAPTURE: usize = 0xF << Shift::CAPTURE;
    pub const EN_PASSANT: usize = 0x1 << Shift::EN_PASSANT;
    pub const DOUBLE_STEP: usize = 0x1 << Shift::DOUBLE_STEP;
    pub const CASTLING: usize = 0x1 << Shift::CASTLING;
}

pub struct MoveData {
    pub piece: usize,
    pub from: usize,
    pub to: usize,
    pub capture: usize,
    pub en_passant: bool,
    pub double_step: bool,
    pub castling: bool,
}

impl MoveData {
    pub fn from_bits(move_data: usize) -> Self {
        Self {
            piece: move_data & Mask::PIECE,
            from: (move_data & Mask::FROM_SQ) >> Shift::FROM_SQ,
            to: (move_data & Mask::TO_SQ) >> Shift::TO_SQ,
            capture: (move_data & Mask::CAPTURE) >> Shift::CAPTURE,
            en_passant: ((move_data & Mask::EN_PASSANT) >> Shift::EN_PASSANT) != 0,
            double_step: ((move_data & Mask::DOUBLE_STEP) >> Shift::DOUBLE_STEP) != 0,
            castling: ((move_data & Mask::CASTLING) >> Shift::CASTLING) != 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MoveType {
    Quiet,
    Capture,
    All,
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Side {
    White,
    Black,
}

impl From<usize> for Side {
    fn from(value: usize) -> Self {
        match value {
            0 => Side::White,
            1 => Side::Black,
            _ => panic!("Invalid side index"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub data: usize,
}

impl Move {
    pub fn new(data: usize) -> Self {
        Self { data }
    }
    pub fn from(&self) -> Square {
        // 0x3F is binary number 0b00111111
        ((self.data >> Shift::FROM_SQ as u64) & 0x3F) as Square
    }

    pub fn to(&self) -> Square {
        // 0x3F is binary number 0b00111111
        ((self.data >> Shift::TO_SQ as u64) & 0x3F) as Square
    }
    pub fn promoted(&self) -> Piece {
        // 0x7 is binary number 111
        ((self.data >> Shift::PROMOTION as u64) & 0x7) as Piece
    }
    pub fn piece(&self) -> Piece {
        // 0x7 is binary number 111
        ((self.data >> Shift::PIECE as u64) & 0x7) as Piece
    }
    pub fn captured(&self) -> Piece {
        // 0x7 is binary number 111
        ((self.data >> Shift::CAPTURE as u64) & 0x7) as Piece
    }
    pub fn castling(&self) -> bool {
        ((self.data >> Shift::CASTLING as u64) & 0x1) as u8 == 1
    }
    pub fn double_push(&self) -> bool {
        ((self.data >> Shift::DOUBLE_STEP as u64) & 0x1) as u8 == 1
    }
    pub fn en_passant(&self) -> bool {
        ((self.data >> Shift::EN_PASSANT as u64) & 0x1) as u8 == 1
    }
    pub fn get_sort_score(self) -> u32 {
        ((self.data >> Shift::SORTSCORE as u64) & 0xFFFFFFFF) as u32
    }

    pub fn set_sort_score(&mut self, value: u32) {
        let mask: usize = 0xFFFFFFFF << Shift::SORTSCORE;
        let v: usize = (value as usize) << Shift::SORTSCORE;
        self.data = (self.data & !mask) | v;
    }
    pub fn to_short_move(self) -> ShortMove {
        ShortMove::new((self.data & MOVE_ONLY) as u32)
    }

    pub fn get_move(&self) -> u32 {
        (self.data & MOVE_ONLY) as u32
    }

    pub fn as_string(&self) -> String {
        format!(
            "{}{}{}",
            SQUARE_NAME[self.from()],
            SQUARE_NAME[self.to()],
            PIECE_CHAR_SMALL[self.promoted()]
        )
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct ShortMove {
    data: u32,
}

impl ShortMove {
    pub fn new(m: u32) -> Self {
        Self { data: m }
    }

    pub fn get_move(&self) -> u32 {
        self.data
    }
}

// Prints a given movelist to the screen.
#[allow(dead_code)]
pub fn movelist(ml: &MoveList) {
    for i in 0..ml.len() {
        move_data(ml.get_move(i), i);
    }
}

// Prints decoded move data to the screen.
#[allow(dead_code)]
pub fn move_data(m: Move, nr: u8) {
    println!(
        "{}. Move: {}{}{} capture: {}, promotion: {}, ep: {}, double: {}, castling: {}, score: {}",
        nr + 1,
        PIECE_CHAR_CAPS[m.piece()],
        SQUARE_NAME[m.from()],
        SQUARE_NAME[m.to()],
        PIECE_NAME[m.captured()],
        PIECE_NAME[m.promoted()],
        m.en_passant(),
        m.double_push(),
        m.castling(),
        m.get_sort_score(),
    );
}

pub fn castling_as_string(permissions: u8) -> String {
    let mut castling_as_string: String = String::from("");
    let p = permissions;

    castling_as_string += if p & Castling::WK > 0 { "K" } else { "" };
    castling_as_string += if p & Castling::WQ > 0 { "Q" } else { "" };
    castling_as_string += if p & Castling::BK > 0 { "k" } else { "" };
    castling_as_string += if p & Castling::BQ > 0 { "q" } else { "" };

    if castling_as_string.is_empty() {
        castling_as_string = String::from("-");
    }

    castling_as_string
}

// Convert square names to numbers.
pub fn algebraic_from_str(square: &str) -> Option<usize> {
    SQUARE_NAME.iter().position(|&element| element == square)
}

pub fn get_bit(bitboard: &u64, square: usize) -> u64 {
    bitboard & (1 << square)
}

pub fn set_bit(bitboard: &mut u64, square: usize) {
    *bitboard |= 1u64 << square
}

pub fn pop_bit(bitboard: &mut u64, square: usize) {
    if get_bit(&bitboard, square) != 0 {
        *bitboard &= !(1u64 << square)
    }
}

pub fn get_least_significant_1st_bit(bitboard: &u64) -> usize {
    if *bitboard != 0 {
        bitboard.trailing_zeros() as usize
    } else {
        usize::MAX
    }
}
