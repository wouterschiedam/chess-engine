mod create;
pub mod defs;
mod init;
mod magics;
mod movelist;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    board::{
        defs::{Pieces, Squares, BB_RANKS, BB_SQUARES},
        Board,
    },
    defs::{Bitboard, Castling, NrOf, Piece, Side, Sides, Square, EMPTY},
    extra::bits::{self},
    search::defs::PerftSummary,
};

use crate::movegen::magics::Magic;

use self::defs::{Move, MoveList, MoveType, Shift};

pub const PROMOTION_PIECES: [usize; 4] =
    [Pieces::QUEEN, Pieces::ROOK, Pieces::KNIGHT, Pieces::BISHOP];

pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.
pub struct MoveGenerator {
    pub king: [Bitboard; NrOf::SQUARES],
    pub knight: [Bitboard; NrOf::SQUARES],
    pub pawns: [[Bitboard; NrOf::SQUARES]; Sides::BOTH],
    pub rook: Vec<Bitboard>,
    pub bishop: Vec<Bitboard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
}

#[derive(Debug, PartialEq, Clone)]
pub struct MoveStats {
    captures: u64,
    en_passants: u64,
    castles: u64,
    promotions: u64,
    checks: u64,
    discovery_checks: u64,
    double_checks: u64,
    checkmates: u64,
}

impl MoveStats {
    pub fn new() -> Self {
        Self {
            captures: 0,
            en_passants: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            discovery_checks: 0,
            double_checks: 0,
            checkmates: 0,
        }
    }
    pub fn log(&self) {
        println!("Captures: {}", self.captures);
        println!("En Passants: {}", self.en_passants);
        println!("Castles: {}", self.castles);
        println!("Promotions: {}", self.promotions);
        println!("Checks: {}", self.checks);
        println!("Discovery Checks: {}", self.discovery_checks);
        println!("Double Checks: {}", self.double_checks);
        println!("Checkmates: {}\n", self.checkmates);
    }
}

impl MoveGenerator {
    pub fn new() -> Self {
        let magics: Magic = Default::default();

        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NrOf::SQUARES],
            bishop_magics: [magics; NrOf::SQUARES],
        };

        mg.init_king_attack();
        mg.init_knight_attack();
        mg.init_pawn_attack();
        mg.init_magics(Pieces::ROOK);
        mg.init_magics(Pieces::BISHOP);
        mg
    }

    // Generates moves for the side that is to move. The MoveType parameter
    // determines if all moves, or only captures need to be generated.
    pub fn generate_moves(
        &self,
        board: &Board,
        move_list: &mut MoveList,
        move_type: MoveType,
        stats: &mut MoveStats,
    ) {
        self.piece(board, Pieces::KING, move_list, move_type, stats);
        self.piece(board, Pieces::KNIGHT, move_list, move_type, stats);
        self.piece(board, Pieces::ROOK, move_list, move_type, stats);
        self.piece(board, Pieces::BISHOP, move_list, move_type, stats);
        self.piece(board, Pieces::QUEEN, move_list, move_type, stats);
        self.pawns(board, move_list, move_type, stats);
        if move_type == MoveType::All || move_type == MoveType::Quiet {
            self.castling(board, move_list, stats);
        }
    }

    pub fn get_non_slider_moves(&self, piece: Piece, square: Square) -> Bitboard {
        match piece {
            Pieces::KING => self.king[square],
            Pieces::KNIGHT => self.knight[square],
            _ => panic!("Not a king or knight {piece}"),
        }
    }

    pub fn get_slider_moves(&self, piece: Piece, square: Square, occupancy: Bitboard) -> Bitboard {
        match piece {
            Pieces::ROOK => {
                let index = self.rook_magics[square].get_index(occupancy);
                self.rook[index]
            }
            Pieces::BISHOP => {
                let index = self.bishop_magics[square].get_index(occupancy);
                self.bishop[index]
            }
            Pieces::QUEEN => {
                let r_index = self.rook_magics[square].get_index(occupancy);
                let b_index = self.bishop_magics[square].get_index(occupancy);
                self.rook[r_index] ^ self.bishop[b_index]
            }
            _ => panic!("Not a ROOK or BISHOP but {piece}"),
        }
    }

    pub fn get_pawn_attacks(&self, side: Side, square: Square) -> Bitboard {
        self.pawns[side][square]
    }
}

// GET ACTUAL PSEUDO LEGAL MOVES!!!!
impl MoveGenerator {
    pub fn piece(
        &self,
        board: &Board,
        piece: Piece,
        move_list: &mut MoveList,
        move_type: MoveType,
        stats: &mut MoveStats,
    ) {
        let player = board.side_to_move();
        let bb_occupancy = board.occupancy();
        // find all square that are empty occupied by our pieces and opponent pieces
        let bb_empty = !bb_occupancy;
        let bb_own_pieces = board.bb_side[player];
        let bb_opponent_pieces = board.bb_side[board.side_to_not_move()];
        let mut bb_pieces = board.get_pieces(piece, player);
        // Generate all moves for each piece
        while bb_pieces > 0 {
            let from = bits::next(&mut bb_pieces);
            let bb_target = match piece {
                Pieces::KING | Pieces::KNIGHT => self.get_non_slider_moves(piece, from),
                Pieces::ROOK | Pieces::BISHOP | Pieces::QUEEN => {
                    self.get_slider_moves(piece, from, bb_occupancy)
                }
                _ => panic!("Not a valid piece {piece}"),
            };

            // generate move according to movetype
            let bb_moves = match move_type {
                MoveType::All => bb_target & !bb_own_pieces,
                MoveType::Quiet => bb_target & bb_empty,
                MoveType::Capture => bb_target & bb_opponent_pieces,
            };

            self.add_move(board, piece, from, bb_moves, move_list, stats);
        }
    }

    pub fn pawns(&self, board: &Board, list: &mut MoveList, mt: MoveType, stats: &mut MoveStats) {
        const UP: i8 = 8;
        const DOWN: i8 = -8;

        // Create shorthand variables.
        let us = board.side_to_move();
        let bb_opponent_pieces = board.bb_side[board.side_to_not_move()];
        let bb_empty = !board.occupancy();
        let bb_fourth = BB_RANKS[Board::fourth_rank(us)];
        let direction = if us == Sides::WHITE { UP } else { DOWN };
        let rotation_count = (NrOf::SQUARES as i8 + direction) as u32;
        let mut bb_pawns = board.get_pieces(Pieces::PAWN, us);

        // As long as there are pawns, generate moves for each of them.
        while bb_pawns > 0 {
            let from = bits::next(&mut bb_pawns);
            let to = (from as i8 + direction) as usize;
            let mut bb_moves = 0;

            // Generate pawn pushes
            if mt == MoveType::All || mt == MoveType::Quiet {
                let bb_push = BB_SQUARES[to];
                let bb_one_step = bb_push & bb_empty;
                let bb_two_step = bb_one_step.rotate_left(rotation_count) & bb_empty & bb_fourth;
                bb_moves |= bb_one_step | bb_two_step;
            }

            // Generate pawn captures
            if mt == MoveType::All || mt == MoveType::Capture {
                let bb_targets = self.get_pawn_attacks(us, from);
                let bb_captures = bb_targets & bb_opponent_pieces;
                let bb_ep_capture = match board.gamestate.en_passant {
                    Some(ep) => bb_targets & BB_SQUARES[ep as usize],
                    None => 0,
                };
                bb_moves |= bb_captures | bb_ep_capture;
            }

            self.add_move(board, Pieces::PAWN, from, bb_moves, list, stats);
        }
    }

    pub fn castling(&self, board: &Board, move_list: &mut MoveList, stats: &mut MoveStats) -> bool {
        let player = board.side_to_move();
        let opponent = board.side_to_not_move();

        let castle_perm_w = (board.gamestate.castling & (Castling::WK | Castling::WQ)) > 0;
        let castle_perm_b = (board.gamestate.castling & (Castling::BK | Castling::BQ)) > 0;

        let bb_occupancy = board.occupancy();

        let mut bb_king = board.get_pieces(Pieces::KING, player);
        let from = bits::next(&mut bb_king);
        // White castling
        if castle_perm_w && player == Sides::WHITE {
            // kingside
            if board.gamestate.castling & Castling::WK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F1] | BB_SQUARES[Squares::G1];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::F1)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::KING, from, to, move_list, stats);
                }
            }

            if board.gamestate.castling & Castling::WQ > 0 {
                // Queenside
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B1] | BB_SQUARES[Squares::C1] | BB_SQUARES[Squares::D1];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::D1)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::KING, from, to, move_list, stats);
                }
            }
        }

        // Black castling
        if player == Sides::BLACK && castle_perm_b {
            // Kingside
            if board.gamestate.castling & Castling::BK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F8] | BB_SQUARES[Squares::G8];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::F8)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::KING, from, to, move_list, stats);
                }
            }

            // Queenside
            if board.gamestate.castling & Castling::BQ > 0 {
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B8] | BB_SQUARES[Squares::C8] | BB_SQUARES[Squares::D8];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::D8)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::KING, from, to, move_list, stats);
                }
            }
        }

        move_list.len() > 0
    }

    pub fn add_move(
        &self,
        board: &Board,
        piece: Piece,
        from: Square,
        to: Bitboard,
        move_list: &mut MoveList,
        stats: &mut MoveStats,
    ) {
        let mut bb_to = to;
        let player = board.side_to_move();
        let promotion_rank = Board::promotion_rank(player);
        let is_pawn = piece == Pieces::PAWN;

        while bb_to > 0 {
            let to_square = bits::next(&mut bb_to);
            let capture = board.piece_list[to_square];
            let en_passant = match board.gamestate.en_passant {
                Some(square) => is_pawn && (square as usize == to_square),
                None => false,
            };

            let promotion = is_pawn && Board::square_on_rank(to_square, promotion_rank);
            let double_push = is_pawn && ((to_square as i8 - from as i8).abs() == 16);
            let castling = (piece == Pieces::KING) && ((to_square as i8 - from as i8).abs() == 2);

            // add all data into a 64 bit variable
            let mut move_data = (piece)
                | from << Shift::FROM_SQ
                | to_square << Shift::TO_SQ
                | capture << Shift::CAPTURE
                | (en_passant as usize) << Shift::EN_PASSANT
                | (double_push as usize) << Shift::DOUBLE_STEP
                | (castling as usize) << Shift::CASTLING;

            if !promotion {
                move_data |= Pieces::NONE << Shift::PROMOTION;

                let mut cloned_board = board.clone();
                cloned_board.make_move(Move::new(move_data), &self);

                if capture != Pieces::NONE {
                    stats.captures += 1;
                }
                if en_passant {
                    stats.en_passants += 1;
                }
                if castling {
                    stats.castles += 1;
                }

                // Check if the move results in a check
                let opponent = board.side_to_not_move();
                let king_square = cloned_board.king_square(opponent);
                if self.square_attacked(&cloned_board, player, king_square) {
                    stats.checks += 1;
                    println!("Check: from {} to {}", from, to_square);
                    println!("fen: {}", cloned_board.create_fen());

                    // Check for double checks
                    let attackers =
                        self.square_attacked_multiple(&cloned_board, player, king_square);
                    if attackers > 1 {
                        stats.double_checks += 1;
                    }
                }

                // // Check if the move results in checkmate
                // if cloned_board.is_checkmate() {
                //     stats.checkmates += 1;
                // }

                move_list.push(Move::new(move_data));
            } else {
                PROMOTION_PIECES.iter().for_each(|&promoted_piece| {
                    let promotion_piece = promoted_piece << Shift::PROMOTION;
                    let move_with_promotion = move_data | promotion_piece;

                    let mut cloned_board = board.clone();
                    cloned_board.make_move(Move::new(move_with_promotion), &self);

                    if capture != Pieces::NONE {
                        stats.captures += 1;
                    }
                    if en_passant {
                        stats.en_passants += 1;
                    }

                    // Check if the promotion move results in a check
                    let opponent = board.side_to_not_move();
                    let king_square = cloned_board.king_square(opponent);
                    if self.square_attacked(&cloned_board, player, king_square) {
                        stats.checks += 1;

                        // Check for double checks
                        let attackers =
                            self.square_attacked_multiple(&cloned_board, player, king_square);
                        if attackers > 1 {
                            stats.double_checks += 1;
                        }
                    }

                    // Check if the promotion move results in checkmate
                    // if self.is_checkmate() {
                    //     stats.checkmates += 1;
                    // }

                    move_list.push(Move::new(move_with_promotion));
                    stats.promotions += 1;
                });
            }
        }
    }
}

impl MoveGenerator {
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    // Determine if a square is attacked by 'attacker', on the given board.
    pub fn square_attacked(&self, board: &Board, attacker: Side, square: Square) -> bool {
        let attackers = board.bb_pieces[attacker];

        // Use the super-piece method: get the moves for each piece,
        // starting from the given square. This provides the squares where
        // a piece has to be, to be able to reach the given square.
        let occupancy = board.occupancy();
        let bb_king = self.get_non_slider_moves(Pieces::KING, square);
        let bb_rook = self.get_slider_moves(Pieces::ROOK, square, occupancy);
        let bb_bishop = self.get_slider_moves(Pieces::BISHOP, square, occupancy);
        let bb_knight = self.get_non_slider_moves(Pieces::KNIGHT, square);
        let bb_pawns = self.get_pawn_attacks(attacker ^ 1, square);
        let bb_queen = bb_rook | bb_bishop;

        // Then determine if such a piece is actually there: see if a rook
        // is on one of the squares a rook has to be to reach the given
        // square. Same for the queen, knight, etc... As soon as one is
        // found, the square is attacked.
        (bb_king & attackers[Pieces::KING] > 0)
            || (bb_rook & attackers[Pieces::ROOK] > 0)
            || (bb_queen & attackers[Pieces::QUEEN] > 0)
            || (bb_bishop & attackers[Pieces::BISHOP] > 0)
            || (bb_knight & attackers[Pieces::KNIGHT] > 0)
            || (bb_pawns & attackers[Pieces::PAWN] > 0)
    }

    // Determine double attckers mostly used for double checks
    pub fn square_attacked_multiple(&self, board: &Board, attacker: Side, square: Square) -> u8 {
        let attackers = board.bb_pieces[attacker];

        let occupancy = board.occupancy();
        let bb_king = self.get_non_slider_moves(Pieces::KING, square);
        let bb_rook = self.get_slider_moves(Pieces::ROOK, square, occupancy);
        let bb_bishop = self.get_slider_moves(Pieces::BISHOP, square, occupancy);
        let bb_knight = self.get_non_slider_moves(Pieces::KNIGHT, square);
        let bb_pawns = self.get_pawn_attacks(attacker ^ 1, square);
        let bb_queen = bb_rook | bb_bishop;

        let mut count: u8 = 0;
        if bb_king & attackers[Pieces::KING] > 0 {
            count += 1;
        }
        if bb_rook & attackers[Pieces::ROOK] > 0 {
            count += 1;
        }
        if bb_queen & attackers[Pieces::QUEEN] > 0 {
            count += 1;
        }
        if bb_bishop & attackers[Pieces::BISHOP] > 0 {
            count += 1;
        }
        if bb_knight & attackers[Pieces::KNIGHT] > 0 {
            count += 1;
        }
        if bb_pawns & attackers[Pieces::PAWN] > 0 {
            count += 1;
        }
        count
    }
}

impl MoveGenerator {
    pub fn go_perft_results(
        mut board: Board,
        depth: i8,
        movegen: &Arc<MoveGenerator>,
    ) -> PerftSummary {
        let mut move_list = MoveList::new();
        let mut move_stats = MoveStats::new();
        let mut perft_result: HashMap<String, i32> = Default::default();

        let mut total_nodes: i32 = 0;

        let elapsed_time = Self::measure_time(|| {
            let _ = movegen.generate_moves(&board, &mut move_list, MoveType::All, &mut move_stats);

            for mov in move_list.moves.iter() {
                if mov.data > 0 {
                    board.make_move(*mov, &movegen);
                    let the_move = format!("{}", mov.as_string());
                    let nodes = Self::perft_results(
                        depth - 1,
                        &mut board,
                        &movegen,
                        &the_move,
                        &mut move_stats,
                    );
                    total_nodes += nodes;
                    perft_result.insert(the_move, nodes);
                    board.unmake();
                }
            }
        });

        PerftSummary {
            depth,
            nodes: total_nodes,
            moves: perft_result,
            time: elapsed_time,
            move_stats,
        }
    }

    fn perft_results(
        depth: i8,
        board: &mut Board,
        move_generator: &MoveGenerator,
        current_move: &str,
        stats: &mut MoveStats,
    ) -> i32 {
        if depth == 0 {
            return 1;
        }

        let mut move_list = MoveList::new();
        let _ = move_generator.generate_moves(&board, &mut move_list, MoveType::All, stats);

        let mut total_nodes = 0;

        for mov in move_list.moves.iter() {
            if mov.data > 0 {
                if board.make_move(*mov, move_generator) {
                    let nodes =
                        Self::perft_results(depth - 1, board, &move_generator, current_move, stats);
                    total_nodes += nodes;
                    board.unmake();
                }
            }
        }

        total_nodes
    }

    fn measure_time<F: FnOnce()>(function: F) -> Duration {
        let start_time = Instant::now();
        function();
        let end_time = Instant::now();
        end_time - start_time
    }
}
