use crate::helper::{self, *};
pub const NOT_A_FILE: u64 = 18374403900871474942;

pub const NOT_H_FILE: u64 = 9187201950435737471;

pub const NOT_AB_FILE: u64 = 18229723555195321596;

pub const NOT_HG_FILE: u64 = 4557430888798830399;

pub trait PieceType {
    fn attacks(&self, side: Side, square: usize) -> u64;
}

pub struct Pawn;
impl PieceType for Pawn {
    fn attacks(&self, side: Side, square: usize) -> u64 {
        generate_pawn_attack(side, square)
    }
}

pub struct Rook;
impl PieceType for Rook {
    fn attacks(&self, _side: Side, square: usize) -> u64 {
        mask_rook_attack(square)
    }
}

pub struct Knight;
impl PieceType for Knight {
    fn attacks(&self, _side: Side, square: usize) -> u64 {
        generate_knight_attack(square)
    }
}

pub struct Bishop;
impl PieceType for Bishop {
    fn attacks(&self, _side: Side, square: usize) -> u64 {
        mask_bishop_attack(square)
    }
}

pub struct King;
impl PieceType for King {
    fn attacks(&self, _side: Side, square: usize) -> u64 {
        generate_king_attack(square)
    }
}

pub struct AttackTable<P: PieceType> {
    pub pawn_attacks: [[u64; BOARD_SIZE]; 2],
    pub knight_attacks: [u64; BOARD_SIZE],
    pub bishop_attacks: [u64; BOARD_SIZE],
    pub rook_attacks: [u64; BOARD_SIZE],
    pub king_attacks: [u64; BOARD_SIZE],
    _marker: std::marker::PhantomData<P>,
}

impl<P: PieceType> AttackTable<P> {
    pub fn new() -> Self {
        let mut mg = Self {
            pawn_attacks: [[0; BOARD_SIZE]; 2],
            knight_attacks: [0; BOARD_SIZE],
            bishop_attacks: [0; BOARD_SIZE],
            rook_attacks: [0; BOARD_SIZE],
            king_attacks: [0; BOARD_SIZE],
            _marker: std::marker::PhantomData,
        };

        // Here is side essential cause pawns can only move up for white an down for black
        for side in 0..2 {
            for square in 0..BOARD_SIZE {
                mg.pawn_attacks[side][square] = generate_pawn_attack(Side::from(side), square);
            }
        }

        for square in 0..BOARD_SIZE {
            mg.knight_attacks[square] = generate_knight_attack(square);
        }

        for square in 0..BOARD_SIZE {
            mg.bishop_attacks[square] = mask_bishop_attack(square);
        }

        for square in 0..BOARD_SIZE {
            mg.rook_attacks[square] = mask_rook_attack(square);
        }

        for square in 0..BOARD_SIZE {
            mg.king_attacks[square] = generate_king_attack(square);
        }

        mg
    }

    pub fn print_pawn_attacks(&self) {
        for (side_idx, attacks) in self.pawn_attacks.iter().enumerate() {
            for (square_idx, &attack) in attacks.iter().enumerate() {
                println!("Side: {:?}, Square: {}", Side::from(side_idx), square_idx);
                print_bitboard(attack);
                println!();
            }
        }
    }

    pub fn print_knight_attacks(&self) {
        for (square_idx, attacks) in self.knight_attacks.iter().enumerate() {
            println!("Square: {}", square_idx);
            print_bitboard(*attacks);
            println!();
        }
    }

    pub fn print_king_attacks(&self) {
        for (square_idx, attacks) in self.king_attacks.iter().enumerate() {
            println!("Square: {}", square_idx);
            print_bitboard(*attacks);
            println!();
        }
    }

    pub fn print_bishop_attacks(&self) {
        for (square_idx, attacks) in self.bishop_attacks.iter().enumerate() {
            println!("Square: {}", square_idx);
            print_bitboard(*attacks);
            println!();
        }
    }

    pub fn print_rook_attacks(&self) {
        for (square_idx, attacks) in self.rook_attacks.iter().enumerate() {
            println!("Square: {}", square_idx);
            print_bitboard(*attacks);
            println!();
        }
    }
}
// Pawn attacks table [side][square]
pub fn generate_pawn_attack(side: Side, square: usize) -> u64 {
    let mut attacks: u64 = 0;

    let mut bitboard: u64 = 0;

    helper::set_bit(&mut bitboard, square);

    // White pawns
    if side == Side::White {
        if (bitboard >> 7) & NOT_A_FILE != 0 {
            attacks |= bitboard >> 7;
        }
        if (bitboard >> 9) & NOT_H_FILE != 0 {
            attacks |= bitboard >> 9;
        }
    }
    // Black pawns
    else {
        if (bitboard << 7) & NOT_H_FILE != 0 {
            attacks |= bitboard << 7;
        }
        if (bitboard << 9) & NOT_A_FILE != 0 {
            attacks |= bitboard << 9;
        }
    }

    attacks
}

// Rook attacks table [square]
pub fn mask_rook_attack(square: usize) -> u64 {
    let mut attacks: u64 = 0;

    // init target rank & files
    let tr = square as isize / 8;
    let tf = square as isize % 8;

    // mask relevant rook occupancy bits
    for i in 1..8 {
        if tr + i <= 6 {
            attacks |= 1u64 << ((tr + i) * 8 + tf);
        }
        if tr as isize - i >= 1 {
            attacks |= 1u64 << ((tr - i) * 8 + tf);
        }
        if tf + i <= 6 {
            attacks |= 1u64 << (tr * 8 + tf + i);
        }
        if tf as isize - i >= 1 {
            attacks |= 1u64 << (tr * 8 + tf - i);
        }
    }

    attacks
}

fn rook_attack_on_go(square: usize) -> u64 {
    // result attacks bitboard
    let mut attacks: u64 = 0;

    // init target rank & files
    let tr = square as isize / 8;
    let tf = square as isize % 8;

    // mask relevant rook occupancy bits
    for i in 1..8 {
        if tr + i < 8 {
            attacks |= 1u64 << ((tr + i) * 8 + tf);
        }
        if tr as isize - i >= 0 {
            attacks |= 1u64 << ((tr - i) * 8 + tf);
        }
        if tf + i < 8 {
            attacks |= 1u64 << (tr * 8 + tf + i);
        }
        if tf as isize - i >= 0 {
            attacks |= 1u64 << (tr * 8 + tf - i);
        }
    }

    // return attack map
    attacks
}

// mask bishop attacks table [square]
pub fn mask_bishop_attack(square: usize) -> u64 {
    let mut attacks: u64 = 0;

    // init target rank & files
    let tr = square as isize / 8;
    let tf = square as isize % 8;

    // mask relevant bishop occupancy bits
    for i in 1..8 {
        if tr + i <= 6 && tf + i <= 6 {
            attacks |= 1u64 << ((tr + i) * 8 + tf + i);
        }
        if tr as isize - i >= 1 && tf + i <= 6 {
            attacks |= 1u64 << ((tr - i) * 8 + tf + i);
        }
        if tr + i <= 6 && tf as isize - i >= 1 {
            attacks |= 1u64 << ((tr + i) * 8 + tf as isize - i);
        }
        if tr as isize - i >= 1 && tf as isize - i >= 1 {
            attacks |= 1u64 << ((tr - i) * 8 + tf as isize - i);
        }
    }
    attacks
}

pub fn generate_bishop_attack_on_go(square: usize) -> u64 {
    // result attacks bitboard
    let mut attacks: u64 = 0;

    // init target rank & files
    let tr = square as isize / 8;
    let tf = square as isize % 8;

    // mask relevant bishop occupancy bits
    for i in 1..8 {
        if tr + i < 8 && tf + i < 8 {
            attacks |= 1u64 << ((tr + i) * 8 + tf + i);
        }
        if tr as isize - i >= 0 && tf + i < 8 {
            attacks |= 1u64 << ((tr - i) * 8 + tf + i);
        }
        if tr + i < 8 && tf as isize - i >= 0 {
            attacks |= 1u64 << ((tr + i) * 8 + tf as isize - i);
        }
        if tr as isize - i >= 0 && tf as isize - i >= 0 {
            attacks |= 1u64 << ((tr - i) * 8 + tf as isize - i);
        }
    }

    // return attack map
    attacks
}

// Knight attacks table [square]
pub fn generate_knight_attack(square: usize) -> u64 {
    let mut attacks: u64 = 0;

    let mut bitboard: u64 = 0;

    helper::set_bit(&mut bitboard, square);

    if ((bitboard >> 17) & NOT_H_FILE) != 0 {
        attacks |= bitboard >> 17;
    }
    if ((bitboard >> 15) & NOT_A_FILE) != 0 {
        attacks |= bitboard >> 15;
    }
    if ((bitboard >> 10) & NOT_HG_FILE) != 0 {
        attacks |= bitboard >> 10;
    }
    if ((bitboard >> 6) & NOT_AB_FILE) != 0 {
        attacks |= bitboard >> 6;
    }
    if ((bitboard << 17) & NOT_A_FILE) != 0 {
        attacks |= bitboard << 17;
    }
    if ((bitboard << 15) & NOT_H_FILE) != 0 {
        attacks |= bitboard << 15;
    }
    if ((bitboard << 10) & NOT_AB_FILE) != 0 {
        attacks |= bitboard << 10;
    }
    if ((bitboard << 6) & NOT_HG_FILE) != 0 {
        attacks |= bitboard << 6;
    }

    attacks
}

// King attacks table [square]
pub fn generate_king_attack(square: usize) -> u64 {
    let mut attacks: u64 = 0;

    let mut bitboard: u64 = 0;

    helper::set_bit(&mut bitboard, square);

    if (bitboard >> 8) != 0 {
        attacks |= bitboard >> 8;
    }
    if (bitboard >> 9) & NOT_H_FILE != 0 {
        attacks |= bitboard >> 9;
    }
    if (bitboard >> 7) & NOT_A_FILE != 0 {
        attacks |= bitboard >> 7;
    }
    if (bitboard >> 1) & NOT_H_FILE != 0 {
        attacks |= bitboard >> 1;
    }
    if (bitboard << 8) != 0 {
        attacks |= bitboard << 8;
    }
    if (bitboard << 9) & NOT_A_FILE != 0 {
        attacks |= bitboard << 9;
    }
    if (bitboard << 7) & NOT_H_FILE != 0 {
        attacks |= bitboard << 7;
    }
    if (bitboard << 1) & NOT_A_FILE != 0 {
        attacks |= bitboard << 1;
    }

    attacks
}

pub fn set_occupancy(index: u32, bits_in_mask: usize, attack_mask: &u64) -> u64 {
    // occupancy map
    let mut occupancy: u64 = 0;

    let mut cloned_attack_mask = attack_mask.clone();

    // loop over the range of bits within attack mask
    for count in 0..bits_in_mask {
        // get LS1B
        let square = get_least_significant_1st_bit(&cloned_attack_mask);

        // pop LS1B
        pop_bit(&mut cloned_attack_mask, square);

        if (index & (1 << count)) != 0 {
            occupancy |= 1 << square;
        }
    }

    occupancy
}
