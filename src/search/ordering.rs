use crate::board::Board;
use crate::movegen::{generate_legal_moves, Move};
use crate::board::types::Pieces;

// MVV-LVA: Most Valuable Victim - Least Valuable Attacker
// Order: [KING, QUEEN, ROOK, BISHOP, KNIGHT, PAWN]
const VICTIM_VALUES: [i32; 6] = [600, 500, 400, 300, 200, 100];

pub fn score_move(board: &Board, mv: &Move) -> i32 {
    if mv.flags.is_capture {
        let victim = mv.capture.unwrap_or(Pieces::PAWN);
        let attacker = mv.piece;
        // Higher score for capturing valuable pieces with cheap ones
        return (VICTIM_VALUES[victim] * 10) - VICTIM_VALUES[attacker];
    }
    
    if mv.flags.is_promotion {
        return 900; // Prioritize promotions
    }

    0 // Quiet moves
}
