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
            if tr as isize - i as isize >= 1 && tf + i <= 6 {
                attacks |= 1u64 << ((tr - i) * 8 + tf + i);
            }
            if tr + i <= 6 && tf as isize - i as isize >= 1 {
                attacks |= 1u64 << ((tr + i) * 8 + tf - i);
            }
            if tr as isize - i as isize >= 1 && tf as isize - i as isize >= 1 {
                attacks |= 1u64 << ((tr - i) * 8 + tf - i);
            }
        }
        attacks
    }

    pub fn rook_mask(square: usize) -> Bitboard {
        let mut attacks: u64 = 0;

        // init target rank & files
        let (tr, tf) = Board::square_on_file_rank(square);
        // mask relevant rook occupancy bits
        for i in 1..8 {
            if tr + i <= 6 {
                attacks |= 1u64 << ((tr + i) * 8 + tf);
            }
            if tr as isize - i as isize >= 1 {
                attacks |= 1u64 << ((tr - i) * 8 + tf);
            }
            if tf + i <= 6 {
                attacks |= 1u64 << (tr * 8 + tf + i);
            }
            if tf as isize - i as isize >= 1 {
                attacks |= 1u64 << (tr * 8 + tf - i);
            }
        }
        attacks
    }
    pub fn rook_attack(square: usize, blockers: &[Bitboard]) -> AttackBoard {
        // result attacks bitboard
        let mut bb_attack_boards: AttackBoard = Vec::new();
        for b in blockers.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::Up)
                | MoveGenerator::bb_ray(*b, square, Direction::Right)
                | MoveGenerator::bb_ray(*b, square, Direction::Down)
                | MoveGenerator::bb_ray(*b, square, Direction::Left);
            bb_attack_boards.push(bb_attacks);
        }

        bb_attack_boards
    }

    // Same as the function above, but for the bishop.
    pub fn bishop_attack(square: Square, blockers: &[Bitboard]) -> AttackBoard {
        let mut bb_attack_boards: AttackBoard = Vec::new();

        for b in blockers.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::UpLeft)
                | MoveGenerator::bb_ray(*b, square, Direction::UpRight)
                | MoveGenerator::bb_ray(*b, square, Direction::DownRight)
                | MoveGenerator::bb_ray(*b, square, Direction::DownLeft);
            bb_attack_boards.push(bb_attacks);
        }

        bb_attack_boards
    }
    pub fn blocker_boards(mask: Bitboard) -> BlockerBoards {
        // d is a copy of the input mask
        let d: Bitboard = mask;

        // Initialize the result vector
        let mut bb_blocker_boards: BlockerBoards = Vec::new();

        // n is the current subset of the input mask
        let mut n: Bitboard = 0;

        // Carry-Rippler method for generating subsets
        loop {
            // Add the current subset to the result vector
            bb_blocker_boards.push(n);

            // Update n using the Carry-Rippler method
            n = n.wrapping_sub(d) & d;

            // Break the loop if n becomes zero
            if n == 0 {
                break;
            }
        }

        // Return the vector of blocker boards
        bb_blocker_boards
    }

    pub fn bb_ray(bb_in: Bitboard, square: Square, direction: Direction) -> Bitboard {
        let mut file = Board::square_on_file_rank(square).0 as usize;
        let mut rank = Board::square_on_file_rank(square).1 as usize;
        let mut bb_square = BB_SQUARES[square];
        let mut bb_ray = 0;
        let mut done = false;
        while !done {
            done = true;
            match direction {
                Direction::Up => {
                    if rank != Ranks::R8 {
                        bb_square <<= 8;
                        bb_ray |= bb_square;
                        rank += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Right => {
                    if file != Files::H {
                        bb_square <<= 1;
                        bb_ray |= bb_square;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Down => {
                    if rank != Ranks::R1 {
                        bb_square >>= 8;
                        bb_ray |= bb_square;
                        rank -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Left => {
                    if file != Files::A {
                        bb_square >>= 1;
                        bb_ray |= bb_square;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::UpLeft => {
                    if (rank != Ranks::R8) && (file != Files::A) {
                        bb_square <<= 7;
                        bb_ray |= bb_square;
                        rank += 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::UpRight => {
                    if (rank != Ranks::R8) && (file != Files::H) {
                        bb_square <<= 9;
                        bb_ray |= bb_square;
                        rank += 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::DownRight => {
                    if (rank != Ranks::R1) && (file != Files::H) {
                        bb_square >>= 7;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::DownLeft => {
                    if (rank != Ranks::R1) && (file != Files::A) {
                        bb_square >>= 9;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
            };
        }
        bb_ray
    }
}
