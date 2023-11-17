pub const NOT_A_FILE: u64 = 18374403900871474942;

pub const NOT_H_FILE: u64 = 9187201950435737471;

pub const NOT_AB_FILE: u64 = 18229723555195321596;

pub const NOT_HG_FILE: u64 = 4557430888798830399;

pub const MAX_COLUMNS: usize = 8;
pub const BOARD_SIZE: usize = 64;

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

pub fn algebraic_from_str(square: &str) -> usize {
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

    square_index
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
pub fn count_bits(mut bitboard: u64) -> usize {
    // bit counter
    let mut count = 0;

    // consecutively reset least significant 1st bit
    while bitboard != 0 {
        // increment count
        count += 1;

        // reset least significant 1st bit
        bitboard &= bitboard - 1;
    }

    // return bit count
    count
}

pub fn get_least_significant_1st_bit(bitboard: &u64) -> usize {
    if *bitboard != 0 {
        bitboard.trailing_zeros() as usize
    } else {
        usize::MAX
    }
}
