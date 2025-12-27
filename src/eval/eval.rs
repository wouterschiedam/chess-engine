// Main evaluation function
// Placeholder for evaluation implementation

use crate::{
    board::{BitboardIter, Board},
    defs::{NrOf, Sides},
    eval::pst::{EG_PST, MG_PST},
    utils::Pieces,
};

pub const PIECE_VALUES: [i32; NrOf::PIECE_TYPES] = [
    20000, // KING (Value doesn't strictly matter if not capturable, but helps with search)
    1000,  // QUEEN
    525,   // ROOK
    350,   // BISHOP
    350,   // KNIGHT
    100,   // PAWN
];

const PHASE_WEIGHTS: [i32; 6] = [0, 4, 2, 1, 1, 0]; // K, Q, R, B, N, P
const MAX_PHASE: i32 = 24;

pub fn evaluate(board: &Board) -> i32 {
    let mut mg_score = 0;
    let mut eg_score = 0;
    let mut game_phase = 0;

    for side in [Sides::WHITE, Sides::BLACK] {
        let is_white = side == Sides::WHITE;

        for piece_type in 0..NrOf::PIECE_TYPES {
            let bb = board.bb_pieces[side][piece_type];
            for square in BitboardIter::new(bb) {
                let eval_square = if is_white { square } else { square ^ 56 };

                let mg_eval = PIECE_VALUES[piece_type] + MG_PST[piece_type][eval_square] as i32;
                let eg_eval = PIECE_VALUES[piece_type] + EG_PST[piece_type][eval_square] as i32;

                if is_white {
                    mg_score += mg_eval;
                    eg_score += eg_eval;
                } else {
                    mg_score -= mg_eval;
                    eg_score -= eg_eval;
                }

                game_phase += PHASE_WEIGHTS[piece_type];
            }
        }
    }

    let phase = (game_phase).min(MAX_PHASE);
    let score = ((mg_score * phase) + (eg_score * (MAX_PHASE - phase))) / MAX_PHASE;

    if board.gamestate.active_side == Sides::WHITE {
        score
    } else {
        -score
    }
}
