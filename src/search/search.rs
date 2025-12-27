use crate::{
    board::Board,
    eval::evaluate,
    movegen::{generate_legal_moves, Move},
    search::ordering::score_move,
};

pub struct Search;

pub struct SearchData {
    pub nodes: u64,
}

pub const INFINITY: i32 = 50000;
pub const MATE_VALUE: i32 = 49000;

impl Search {
    pub fn search(board: &mut Board, depth: u8) -> Option<Move> {
        let mut data = SearchData { nodes: 0 };
        let mut best_move = None;
        let mut alpha = -INFINITY;
        let beta = INFINITY;

        let mut moves = generate_legal_moves(board);
        if moves.is_empty() {
            return None;
        }

        // Sort moves for efficiency
        moves.sort_by_key(|m| -score_move(board, m));

        for mv in moves {
            let info = board.make_move(&mv);
            let score = -Self::negamax(board, -beta, -alpha, depth - 1, 1, &mut data);
            board.unmake_move(&mv, &info);

            if score > alpha {
                alpha = score;
                best_move = Some(mv);
            }
        }

        best_move
    }

    fn negamax(
        board: &mut Board,
        mut alpha: i32,
        beta: i32,
        depth: u8,
        ply: u8,
        data: &mut SearchData,
    ) -> i32 {
        data.nodes += 1;

        if depth == 0 {
            return Self::quiescence(board, alpha, beta, data);
        }

        let mut moves = generate_legal_moves(board);

        if moves.is_empty() {
            if board.is_in_check(board.gamestate.active_side) {
                return -MATE_VALUE + ply as i32; // Prefer shorter mates
            }
            return 0; // Stalemate
        }

        moves.sort_by_key(|m| -score_move(board, m));

        for mv in moves {
            let info = board.make_move(&mv);
            let score = -Self::negamax(board, -beta, -alpha, depth - 1, ply + 1, data);
            board.unmake_move(&mv, &info);

            if score >= beta {
                return beta; // Fail-high cutoff
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn quiescence(board: &mut Board, mut alpha: i32, beta: i32, data: &mut SearchData) -> i32 {
        data.nodes += 1;
        let stand_pat = evaluate(board);

        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut moves = generate_legal_moves(board);
        // Only consider captures in quiescence search
        moves.retain(|m| m.flags.is_capture);
        moves.sort_by_key(|m| -score_move(board, m));

        for mv in moves {
            let info = board.make_move(&mv);
            let score = -Self::quiescence(board, -beta, -alpha, data);
            board.unmake_move(&mv, &info);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }
}
