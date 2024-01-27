use super::MoveGenerator;
use crate::board::defs::{Files, Ranks, BB_FILES, BB_RANKS, BB_SQUARES};
use crate::movegen::magics::{Magic, BISHOP_MAGIC_NRS, ROOK_MAGIC_NRS};
use crate::movegen::{defs::*, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE};
use crate::{
    board::defs::{Pieces, RangeOf},
    defs::{Piece, Sides, EMPTY},
};

impl MoveGenerator {
    // King attacks table [square]
    pub fn init_king_attack(&mut self) {
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let bb_moves = (bb_square & !BB_FILES[Files::A] & !BB_RANKS[Ranks::R8]) << 7
                | (bb_square & !BB_RANKS[Ranks::R8]) << 8
                | (bb_square & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R8]) << 9
                | (bb_square & !BB_FILES[Files::H]) << 1
                | (bb_square & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R1]) >> 7
                | (bb_square & !BB_RANKS[Ranks::R1]) >> 8
                | (bb_square & !BB_FILES[Files::A] & !BB_RANKS[Ranks::R1]) >> 9
                | (bb_square & !BB_FILES[Files::A]) >> 1;
            self.king[sq] = bb_moves;
        }
    }

    // Knight attacks table [square]
    pub fn init_knight_attack(&mut self) {
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let bb_moves = (bb_square
                & !BB_RANKS[Ranks::R8]
                & !BB_RANKS[Ranks::R7]
                & !BB_FILES[Files::A])
                << 15
                | (bb_square & !BB_RANKS[Ranks::R8] & !BB_RANKS[Ranks::R7] & !BB_FILES[Files::H])
                    << 17
                | (bb_square & !BB_FILES[Files::A] & !BB_FILES[Files::B] & !BB_RANKS[Ranks::R8])
                    << 6
                | (bb_square & !BB_FILES[Files::G] & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R8])
                    << 10
                | (bb_square & !BB_RANKS[Ranks::R1] & !BB_RANKS[Ranks::R2] & !BB_FILES[Files::A])
                    >> 17
                | (bb_square & !BB_RANKS[Ranks::R1] & !BB_RANKS[Ranks::R2] & !BB_FILES[Files::H])
                    >> 15
                | (bb_square & !BB_FILES[Files::A] & !BB_FILES[Files::B] & !BB_RANKS[Ranks::R1])
                    >> 10
                | (bb_square & !BB_FILES[Files::G] & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R1])
                    >> 6;
            self.knight[sq] = bb_moves;
        }
    }

    // Pawn attacks table [side][square]
    pub fn init_pawn_attack(&mut self) {
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let w = (bb_square & !BB_FILES[Files::A]) << 7 | (bb_square & !BB_FILES[Files::H]) << 9;
            let b = (bb_square & !BB_FILES[Files::A]) >> 9 | (bb_square & !BB_FILES[Files::H]) >> 7;
            self.pawns[Sides::WHITE][sq] = w;
            self.pawns[Sides::BLACK][sq] = b;
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
            let mut magic: Magic = Default::default(); // New magic
            let r_magic_nr = ROOK_MAGIC_NRS[square];
            let b_magic_nr = BISHOP_MAGIC_NRS[square];

            // Set up the new magic with the values we already know.
            magic.mask = mask;
            magic.shift = (64 - bits) as u8;
            magic.offset = offset;
            magic.nr = if is_rook { r_magic_nr } else { b_magic_nr };

            for i in 0..permutations {
                let next = i as usize;
                let index = magic.get_index(blocker_boards[next]);
                let rook_table = &mut self.rook[..];
                let bishop_table = &mut self.bishop[..];
                let table = if is_rook { rook_table } else { bishop_table };

                if table[index] == EMPTY {
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error. Error in Magics.");
                    table[index] = attack_boards[next];
                } else {
                    panic!("Attack table index not empty. Error in Magics.");
                }
            }

            // Store magics
            if is_rook {
                self.rook_magics[square] = magic;
            } else {
                self.bishop_magics[square] = magic;
            }

            // Do the next magic.
            offset += permutations;
        }

        // All permutations (blocker boards) should have been indexed.
        let r_ts = ROOK_TABLE_SIZE as u64;
        let b_ts = BISHOP_TABLE_SIZE as u64;
        let expectation = if is_rook { r_ts } else { b_ts };
        const ERROR: &str = "Initializing magics failed. Check magic numbers.";

        assert!(offset == expectation, "{}", ERROR);
    }
}
