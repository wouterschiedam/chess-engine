use super::{defs::SearchRefs, Search};
use crate::{defs::Sides, movegen::defs::Move, search::defs::MAX_KILLER_MOVES};

pub const OVERHEAD: i128 = 50; // msecs
const GAME_LENGTH: usize = 25; // moves
const MOVES_BUFFER: usize = 5; //moves
const CRITICAL_TIME: u128 = 1_000; // msecs
const OK_TIME: u128 = CRITICAL_TIME * 5; // msecs

impl Search {
    // Determine if allocated search time has been used up.
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        let elapsed = refs.search_info.time_elapsed();
        let allocated = refs.search_info.allocated_time;

        // Calculate a factor with which it is allowed to overshoot the
        // allocated search time. The more time the engine has, the larger
        // the overshoot-factor can be.
        let overshoot_factor = match allocated {
            x if x > OK_TIME => 2.0,                       // Allow large overshoot.
            x if x > CRITICAL_TIME && x <= OK_TIME => 1.5, // Low on time. Reduce overshoot.
            x if x <= CRITICAL_TIME => 1.0,                // Critical time. Don't overshoot.
            _ => 1.0,                                      // This case shouldn't happen.
        };

        elapsed >= (overshoot_factor * allocated as f64).round() as u128
    }

    // Calculates the time the engine allocates for searching a single
    // move. This depends on the number of moves still to go in the game.
    pub fn calculate_time_slice(refs: &SearchRefs) -> u128 {
        // Calculate the time slice step by step.
        let gt = &refs.search_params.game_time;
        let mtg = Search::moves_to_go(refs);
        let white = refs.board.side_to_move() == Sides::WHITE;
        let clock = if white { gt.white_time } else { gt.black_time };
        let increment = if white {
            gt.white_time_incr
        } else {
            gt.black_time_incr
        } as i128;
        let base_time = ((clock as f64) / (mtg as f64)).round() as i128;
        let time_slice = base_time + increment - OVERHEAD;

        // Make sure we're never sending less than 0 msecs of available time.
        if time_slice > 0 {
            // Just send the calculated slice.
            time_slice as u128
        } else if (base_time + increment) > (OVERHEAD / 5) {
            // Don't substract GUI lag protection (overhead) if this leads
            // to a negative time allocation.
            (base_time + increment) as u128
        } else {
            // We actually don't have any time.
            0
        }
    }

    // Here we try to come up with some sort of sensible value for "moves
    // to go", if this value is not supplied.
    fn moves_to_go(refs: &SearchRefs) -> usize {
        // If moves to go was supplied, then use this.
        if let Some(x) = refs.search_params.game_time.moves_to_go {
            x
        } else {
            // Guess moves to go if not supplied.
            let white = refs.board.side_to_move() == Sides::WHITE;
            let ply = refs.board.history.len();
            let moves_made = if white { ply / 2 } else { (ply - 1) / 2 };

            GAME_LENGTH - (moves_made % GAME_LENGTH) + MOVES_BUFFER
        }
    }
}

// Killer moves and history heuristics.
impl Search {
    // This function stores a move in the list of killer moves. Normally we
    // store two killer moves per ply. By checking that the move we want to
    // store is not the same as the first killer move in the list, we make sure
    // that both moves are always different. It is possible to store three or
    // more killer moves, but experience shows that checking for ALL of them to
    // be unique costs more time than the extra killer moves could save.
    pub fn store_killer_move(current_move: Move, refs: &mut SearchRefs) {
        const FIRST: usize = 0;
        let ply = refs.search_info.ply as usize;
        let first_killer = refs.search_info.killer_moves[ply][FIRST];

        // First killer must not be the same as the move being stored.
        if first_killer.get_move() != current_move.get_move() {
            // Shift all the moves one index upward...
            for i in (1..MAX_KILLER_MOVES).rev() {
                let n = i;
                let previous = refs.search_info.killer_moves[ply][n - 1];
                refs.search_info.killer_moves[ply][n] = previous;
            }

            // and add the new killer move in the first spot.
            refs.search_info.killer_moves[ply][0] = current_move.to_short_move();
        }
    }
}
