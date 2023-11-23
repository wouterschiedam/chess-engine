pub const NOT_A_FILE: u64 = 18374403900871474942;

pub const NOT_H_FILE: u64 = 9187201950435737471;

pub const NOT_AB_FILE: u64 = 18229723555195321596;

pub const NOT_HG_FILE: u64 = 4557430888798830399;

pub const MAX_COLUMNS: usize = 8;
pub const MAX_ROWS: usize = 8;
pub const BOARD_SIZE: usize = 64;

pub use super::{magics::Magic, movelist::MoveList};
use crate::{
    board::defs::SQUARE_NAME,
    defs::{Castling, Piece, Square},
};
// bishop relevant occupancy bit count for every square on board
const BISHOP_RELEVANT_BITS: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

// rook relevant occupancy bit count for every square on board
const ROOK_RELEVANT_BITS: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

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
#[derive(Clone, Copy, Debug)]
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
        // 0x1 is least_significant bit
        ((self.data >> Shift::CASTLING as u64) & 0x1) as u8 == 1
    }
    pub fn double_push(&self) -> bool {
        ((self.data >> Shift::DOUBLE_STEP as u64) & 0x1) as u8 == 1
    }
    pub fn en_passant(&self) -> bool {
        ((self.data >> Shift::EN_PASSANT as u64) & 0x1) as u8 == 1
    }
}

pub fn print_bitboard(bitboard: u64) -> () {
    println!("\n");

    // loop over board ranks
    for row in 0..MAX_ROWS {
        // loop over board files
        for column in 0..MAX_COLUMNS {
            // convert file & rank into square index
            let square = row * 8 + column;

            // print ranks
            if column == 0 {
                print!(" {}   ", 8 - row);
            }
            // print bit state (either 1 or 0)
            print!(
                " {:?}",
                if get_bit(&bitboard, square) != 0 {
                    1
                } else {
                    0
                }
            );
        }

        // print new line every rank
        println!("\n");
    }

    // print board files
    println!("\n      a b c d e f g h\n");

    // print bitboard as unsigned decimal number
    println!("     Bitboard: {:?}\n\n", bitboard);
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
    if square.len() != 2 {
        // Invalid algebraic notation
        panic!()
    }

    let file = square.chars().nth(0).unwrap();
    let rank = square.chars().nth(1).unwrap();

    // Convert algebraic notation to 0-based indices
    let row = 8 - rank.to_digit(10).unwrap() as usize;
    let column = (file as usize) - ('a' as usize);

    // Calculate the index in the 1D bitboard representation
    let square_index = row * 8 + column;

    Some(square_index)
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
