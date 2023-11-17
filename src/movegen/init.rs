use super::MoveGenerator;
use crate::board::defs::{Files, Pieces, RangeOf, Ranks, BB_FILES, BB_RANKS, BB_SQUARES};

impl MoveGenerator {
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
}
