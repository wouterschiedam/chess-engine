use super::MoveGenerator;
use crate::movegen::defs::*;
use crate::movegen::magics::Magic;
use crate::{
    board::defs::{Files, Pieces, RangeOf, Ranks, BB_FILES, BB_RANKS, BB_SQUARES},
    defs::{Piece, Sides, EMPTY},
};

impl MoveGenerator {
    // King attacks table [square]
    pub fn init_king_attack(&mut self) {
        for square in RangeOf::SQUARES {
            let mut bitboard: u64 = 0;
            let mut attacks: u64 = 0;
            set_bit(&mut bitboard, square);

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

            self.king[square] = attacks;
        }
    }

    // Knight attacks table [square]
    pub fn init_knight_attack(&mut self) {
        for square in RangeOf::SQUARES {
            let mut bitboard: u64 = 0;
            let mut attacks: u64 = 0;
            set_bit(&mut bitboard, square);

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

            self.knight[square] = attacks;
        }
    }

    // Pawn attacks table [side][square]
    pub fn init_pawn_attack(&mut self) {
        for square in RangeOf::SQUARES {
            let mut bitboard: u64 = 0;
            let mut w: u64 = 0;
            let mut b: u64 = 0;

            set_bit(&mut bitboard, square);

            // White pawns
            if (bitboard >> 7) & NOT_A_FILE != 0 {
                w |= bitboard >> 7;
            }
            if (bitboard >> 9) & NOT_H_FILE != 0 {
                w |= bitboard >> 9;
            }
            // Black pawns
            if (bitboard << 7) & NOT_H_FILE != 0 {
                b |= bitboard << 7;
            }
            if (bitboard << 9) & NOT_A_FILE != 0 {
                b |= bitboard << 9;
            }

            self.pawns[Sides::WHITE][square] = w;
            self.pawns[Sides::BLACK][square] = b;
        }
    }

    pub fn init_magics(&mut self, piece: Piece) {
        // Check for valid piece
        let ok = piece == Pieces::ROOK || piece == Pieces::BISHOP;
        assert!(ok, "Illegal piece: {piece}");

        let is_rook = piece == Pieces::ROOK;

        let mut offset = 0;

        for square in RangeOf::SQUARES {
            let r_mask = MoveGenerator::rook_mask(square);
            let b_mask = MoveGenerator::bishop_mask(square);

            let mask = if is_rook { r_mask } else { b_mask };

            let bits = mask.count_ones();
            let permutations = 2u64.pow(bits); // amount of permutations
            let end = offset + permutations - 1;
            let blocker_boards = MoveGenerator::blocker_boards(mask);

            let r_ab = MoveGenerator::rook_attack(square, &blocker_boards);
            let b_ab = MoveGenerator::bishop_attack(square, &blocker_boards);
            let attack_boards = if is_rook { r_ab } else { b_ab };

            // generate magic numbers
            let mut try_this: Magic = Default::default(); // New magic
            let mut found = false; // While loop breaker if magic works;
            let mut attempts = 0; // Track needed attempts to find the magic.

            // Set up the new magic with the values we already know.
            try_this.mask = mask;
            try_this.shift = (64 - bits) as u8;
            try_this.offset = offset;

            // Start looking for magic numbers that work for this square
            while !found {
                attempts += 1;
                found = true;

                // generate random number
                // try_this = random.gen::<u64>() & random.gen::<u64> & random.gen::<u64>();
            }
        }
    }
}
