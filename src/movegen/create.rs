use super::MoveGenerator;
use crate::{
    board::{
        defs::{Direction, Files, Location, Ranks, BB_FILES, BB_RANKS, BB_SQUARES},
        Board,
    },
    defs::{Bitboard, Square},
};

pub type BlockerBoards = Vec<Bitboard>;
pub type AttackBoard = Vec<Bitboard>;

impl MoveGenerator {
    pub fn bishop_mask(square: usize) -> Bitboard {
        let mut attacks: Bitboard = 0;

        // init target rank & files
        let (tr, tf) = Board::square_on_file_rank(square);

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

    pub fn mask_rook_attack(square: usize) -> u64 {
        let mut attacks: u64 = 0;

        // init target rank & files
        let (tr, tf) = Board::square_on_file_rank(square);
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
}
