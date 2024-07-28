use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::{
    board::{defs::Pieces, Board},
    engine::transposition::{HashFlag, SearchData},
    extra::parse::algebraic_move_to_number,
    movegen::defs::{Move, MoveList, MoveType, ShortMove},
    search::defs::SearchTerminate,
};

use super::{
    defs::{SearchRefs, CHECKMATE, CHECK_TERMINATION, DRAW, INF, STALEMATE},
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
        let is_root = refs.search_info.ply == 0; // At root if no moves were played.
        let mut pvs = false; // Used for PVS (Principal Variation Search)

        // Check if termination condition is met
        if refs.search_info.nodes & CHECK_TERMINATION == 0 {
            Search::check_termination(refs);
        }

        // If time is up, abort. This depth won't be considered in
        // iterative deepening as it is unfinished.
        if refs.search_info.terminated != SearchTerminate::Nothing {
            return 0;
        }

        // Determine if we are in check.
        let is_check = refs.move_generator.square_attacked(
            refs.board,
            refs.board.side_to_not_move(),
            refs.board.king_square(refs.board.side_to_move()),
        );

        // If so, extend search depth by 1
        if is_check {
            depth += 1;
        }

        // Base case: leaf node evaluation
        if depth == 0 {
            return Search::quiescent(alpha, beta, possible_moves, refs);
        }

        // Increment node count
        refs.search_info.nodes += 1;

        // Check for repetition
        if Search::is_repition(&refs.board) {
            return 0; // or DRAW or any other value representing a draw
        }

        // Variables to hold TT value and move if any.
        let mut tt_value: Option<i16> = None;
        let mut tt_move: ShortMove = ShortMove::new(0);

        // Probe the TT for information.
        if refs.tt_enabled {
            if let Some(data) = refs
                .tt
                .lock()
                .expect("Error locking TT")
                .probe(refs.board.gamestate.zobrist_key)
            {
                let tt_result = data.get(depth, refs.search_info.ply, alpha, beta);
                tt_value = tt_result.0;
                tt_move = tt_result.1;
            }
        }

        // If we have a value from the TT, then return immediately.
        if let Some(v) = tt_value {
            if !is_root {
                return v;
            }
        }

        // Generate and score moves
        let mut legal_moves = 0;
        let mut move_list = MoveList::new();
        refs.move_generator
            .generate_moves(&refs.board, &mut move_list, MoveType::All);

        // Check the book for the current position
        let fen = Board::normalize_fen(&refs.board.create_fen()).to_string();
        if let Some(book_moves) = refs.book.get(&fen) {
            if !book_moves.is_empty()
                && Self::handle_book_moves(&book_moves, is_root, &move_list, possible_moves)
            {
                return 0;
            }
        } else {
            println!("No book moves found for FEN: {}", fen);
        }

        Search::score_moves(&mut move_list, tt_move, refs);

        // Set init best eval_score
        let mut best_eval_score = -INF;

        // Set init hash value assuming we do not beat alpha
        let mut hash_flag = HashFlag::Alpha;

        // Holds the best move in the loop
        let mut best_possible_move = ShortMove::new(0);

        for x in 0..move_list.len() {
            Search::swap_move(&mut move_list, x);

            let current_move = move_list.get_move(x);
            if !refs.board.make_move(current_move, refs.move_generator) {
                continue;
            }

            // Avoid moves that would lead to a third repetition if other moves are possible
            if Search::is_repition(&refs.board) && legal_moves > 0 {
                refs.board.unmake();
                continue; // Avoid unfavorable repetition
            }

            legal_moves += 1;
            refs.search_info.ply += 1;

            let mut node_pv = Vec::new();
            let mut eval_score = DRAW;

            if refs.search_info.ply > refs.search_info.seldepth {
                refs.search_info.seldepth = refs.search_info.ply;
            }

            // Perform alpha-beta search
            if !Search::is_draw(refs) {
                // Try pvs if possible
                if pvs {
                    eval_score =
                        -Search::alpha_beta(depth - 1, -alpha - 1, -alpha, &mut node_pv, refs);

                    // Failed pvs?
                    if eval_score > alpha && eval_score < beta {
                        eval_score =
                            -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                    }
                } else {
                    eval_score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                }
            } else {
                return -INF;
            }

            refs.board.unmake();
            refs.search_info.ply -= 1;

            // Update best move and alpha value
            if eval_score > best_eval_score {
                best_eval_score = eval_score;
                best_possible_move = current_move.to_short_move();
            }

            // Beta cutoff: this move is so good for our opponent, that we
            // do not search any further. Insert into TT and return beta.
            if eval_score >= beta {
                refs.tt.lock().expect("Error locking TT").insert(
                    refs.board.gamestate.zobrist_key,
                    SearchData::create(
                        depth,
                        refs.search_info.ply,
                        HashFlag::Beta,
                        beta,
                        best_possible_move,
                    ),
                );

                // If the move is not a capture but still causes a
                // beta-cutoff, then store it as a killer move and update
                // the history heuristics.
                if current_move.captured() == Pieces::NONE {
                    Search::store_killer_move(current_move, refs);
                    //Search::update_history_heuristic(current_move, depth, refs);
                }

                return beta;
            }

            if eval_score > alpha {
                // Save our better eval in alpha
                alpha = eval_score;

                hash_flag = HashFlag::Exact;

                pvs = true;
                possible_moves.clear();
                possible_moves.push(current_move);
                possible_moves.append(&mut node_pv);
            }

            // if alpha >= beta {
            //     break;
            // }
        }

        // Check for checkmate or stalemate
        if legal_moves == 0 {
            return if is_check {
                // The return value is minus CHECKMATE, because if we have
                // no legal moves and are in check, it's game over.
                -CHECKMATE + (refs.search_info.ply as i16)
            } else {
                -STALEMATE
            };
        }

        // We save the best move we found for us; with an ALPHA flag if we
        // didn't improve alpha, or EXACT if we did raise alpha.
        refs.tt.lock().expect("Failed locking tt table").insert(
            refs.board.gamestate.zobrist_key,
            SearchData::create(
                depth,
                refs.search_info.ply,
                hash_flag,
                alpha,
                best_possible_move,
            ),
        );

        alpha
    }

    fn handle_book_moves(
        book_moves: &Vec<(String, u32)>,
        is_root: bool,
        move_list: &MoveList,
        possible_moves: &mut Vec<Move>,
    ) -> bool {
        possible_moves.clear();

        let selected_move = if is_root {
            let mut rng = thread_rng();
            book_moves.choose(&mut rng)
        } else {
            book_moves.first()
        };

        if let Some(book_move) = selected_move {
            if let Ok(parsed_move) = algebraic_move_to_number(&book_move.0) {
                for i in 0..move_list.len() {
                    let current = move_list.get_move(i);
                    if parsed_move.0 == current.from()
                        && parsed_move.1 == current.to()
                        && parsed_move.2 == current.promoted()
                    {
                        possible_moves.push(current);
                        return true;
                    }
                }
            }
        }
        false
    }
}
