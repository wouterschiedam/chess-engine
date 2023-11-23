use crate::defs::{Bitboard, Square};

pub fn next(bitboard: &mut Bitboard) -> Square {
    let square = bitboard.trailing_zeros() as Square;
    *bitboard ^= 1u64 << square;
    square
}
