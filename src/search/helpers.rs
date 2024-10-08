use crate::{
    board::{defs::Pieces, Board},
    defs::{Sides, MAX_MOVE_RULE},
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use super::{defs::SearchRefs, Search};

pub type MoveBook = HashMap<String, Vec<(String, u32)>>;

impl Search {
    pub fn is_draw(refs: &SearchRefs) -> bool {
        let max_move_rule = refs.board.gamestate.halfclock_move >= MAX_MOVE_RULE;
        // Check for max game_rule | insufficiant material | repetition
        max_move_rule || Search::insufficiant_material(refs) || Search::is_repition(refs.board)
    }

    pub fn is_repition(board: &Board) -> bool {
        let mut count = 0;
        let mut stop = false;
        let len = board.history.len();

        // Ensure there is history to check
        if len < 1 {
            return false;
        }
        let mut x = board.history.len() - 1;

        // Traverse the history in reverse
        while x > 0 && !stop {
            x -= 1; // Decrement first since we are indexing from len() - 1
            let historic = board.history.get_ref(x);

            // If zobrist keys are the same, we found a repetition
            if historic.zobrist_key == board.gamestate.zobrist_key {
                count += 1;
            }

            // Stop if we encounter a capture or pawn move (halfmove clock reset)
            stop = historic.halfclock_move == 0;

            // If the position has appeared three times, return true
            if count >= 2 {
                return true;
            }
        }

        false
    }

    // This function calculates the number of nodes per second.
    pub fn nodes_per_second(nodes: usize, msecs: u128) -> usize {
        let mut nps: usize = 0;
        let seconds = msecs as f64 / 1000f64;
        if seconds > 0f64 {
            nps = (nodes as f64 / seconds).round() as usize;
        }
        // if nps == 0 {
        //     nps = 1;
        // }
        nps
    }

    pub fn load_book(filename: &str) -> MoveBook {
        let mut book = MoveBook::new();
        let file = File::open(filename).expect("Failed to open book file");
        let reader = BufReader::new(file);
        let mut current_pos = String::new();

        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            if line.starts_with("pos ") {
                current_pos = line[4..].to_string();
            } else {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    let mv = parts[0].to_string();
                    let weight: u32 = parts[1].parse().expect("Invalid weight");
                    book.entry(current_pos.clone())
                        .or_insert_with(Vec::new)
                        .push((mv, weight));
                }
            }
        }
        book
    }
}

#[rustfmt::skip]
impl Search {
    pub fn insufficiant_material(refs: &SearchRefs) -> bool {
        // It's not a draw if: ...there are still pawns.
        let w_p = refs.board.get_pieces(Pieces::PAWN, Sides::WHITE).count_ones() > 0;     
        let b_p = refs.board.get_pieces(Pieces::PAWN, Sides::BLACK).count_ones() > 0;        
        // ...there's a major piece on the board.
        let w_q = refs.board.get_pieces(Pieces::QUEEN, Sides::WHITE).count_ones() > 0;
        let b_q = refs.board.get_pieces(Pieces::QUEEN, Sides::BLACK).count_ones() > 0;
        let w_r = refs.board.get_pieces(Pieces::ROOK, Sides::WHITE).count_ones() > 0;
        let b_r = refs.board.get_pieces(Pieces::ROOK, Sides::BLACK).count_ones() > 0;
        // ...or two bishops for one side.
        let w_b = refs.board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 1;
        let b_b = refs.board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 1;
        // ... or a bishop+knight for at least one side.
        let w_bn =
            refs.board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 0 &&
            refs.board.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() > 0;
        let b_bn =
            refs.board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 0 &&
            refs.board.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() > 0;
         
        // If one of the conditions above is true, we still have enough
        // material for checkmate, so insufficient_material returns false.
        !(w_p || b_p || w_q || b_q || w_r || b_r || w_b || b_b ||  w_bn || b_bn)
    }
}
