use crate::movegen::defs::{movelist, Move, MoveList, MoveType, ShortMove};
use std::time::Duration;

use super::{
    defs::{SearchRefs, CHECKMATE, CHECK_TERMINATION, DRAW, INF, SEND_STATS, STALEMATE},
    Search,
};

impl Search {
    pub fn alpha_beta(
        mut depth: i8,
        mut alpha: i16,
        beta: i16,
        possible_moves: &mut Vec<Move>,
        refs: &mut SearchRefs,
    ) -> i16 {
        // Define a node limit and time limit for early termination
        const NODE_LIMIT: usize = 1000000; // Example node limit
        const TIME_LIMIT: Duration = Duration::from_secs(10); // Example time limit

        // Check if the node limit or time limit has been reached
        if refs.search_info.nodes > NODE_LIMIT
        // || refs.search_info.start_time.elapsed() > TIME_LIMIT
        {
            return 0; // Early termination value (could be adjusted based on context)
        }

        // Principal variation search flag
        let mut pvs = false;

        // Check if we are in check
        let is_check = refs.move_generator.square_attacked(
            refs.board,
            refs.board.side_to_not_move(),
            refs.board.king_square(refs.board.side_to_move()),
        );

        // Increase depth if in check
        if is_check {
            depth += 1;
        }

        // Base case: leaf node evaluation
        if depth == 0 {
            return Search::quiescent(alpha, beta, possible_moves, refs);
        }

        // Increment node count
        refs.search_info.nodes += 1;

        // Generate and score moves
        let mut move_list = MoveList::new();
        refs.move_generator
            .generate_moves(&refs.board, &mut move_list, MoveType::All);
        Search::score_moves(&mut move_list, ShortMove::new(0), refs);

        let mut best_eval_score = -INF;
        let mut best_possible_move = ShortMove::new(0);
        let mut legal_moves = 0;

        for x in 0..move_list.len() {
            Search::swap_move(&mut move_list, x);

            let current_move = move_list.get_move(x);
            if !refs.board.make_move(current_move, refs.move_generator) {
                continue;
            }

            legal_moves += 1;
            refs.search_info.ply += 1;

            let mut node_pv = Vec::new();
            let mut eval_score;

            // Perform alpha-beta search
            if !Search::is_draw(refs) {
                if pvs {
                    eval_score =
                        -Search::alpha_beta(depth - 1, -alpha - 1, -alpha, &mut node_pv, refs);
                    if eval_score > alpha && eval_score < beta {
                        eval_score =
                            -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                    }
                } else {
                    eval_score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                }
            } else {
                eval_score = 0;
            }

            refs.board.unmake();
            refs.search_info.ply -= 1;

            // Update best move and alpha value
            if eval_score > best_eval_score {
                best_eval_score = eval_score;
                best_possible_move = current_move.to_short_move();
            }

            if eval_score > alpha {
                alpha = eval_score;
                pvs = true;
                possible_moves.clear();
                possible_moves.push(current_move);
                possible_moves.append(&mut node_pv);
            }

            if alpha >= beta {
                break;
            }
        }

        // Check for checkmate or stalemate
        if legal_moves == 0 {
            return if is_check {
                -CHECKMATE + (refs.search_info.ply as i16)
            } else {
                -STALEMATE
            };
        }

        alpha
    }
}
