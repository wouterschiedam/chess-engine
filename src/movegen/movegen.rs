// Move generation module
//
// This module contains functions to generate pseudo-legal moves for all piece types.
// Pseudo-legal moves follow piece movement rules but may leave the king in check.
// Legal move filtering (removing moves that leave king in check) should be done separately.

use crate::{
    board::{
        BitboardIter, Board,
        types::{Pieces, Ranks, Squares},
    },
    defs::{Castling, Side, Sides, Square},
    movegen::{
        attacks::{
            get_bishop_attacks, get_king_attacks, get_knight_attacks, get_pawn_attacks,
            get_queen_attacks, get_rook_attacks,
        },
        moves::{Move, MoveFlags},
    },
};

// ============================================================================
// KNIGHT MOVE GENERATION
// ============================================================================

/// Generate all pseudo-legal knight moves for the given side
///
/// Knights move in L-shapes (2 squares in one direction, 1 square perpendicular).
/// This function:
/// 1. Gets all knights for the active side from the board
/// 2. For each knight, uses the pre-computed attack table to find target squares
/// 3. Filters out squares occupied by friendly pieces
/// 4. Creates Move structs for each valid target (empty squares or enemy captures)
pub fn generate_knight_moves(board: &Board, side: Side, moves: &mut Vec<Move>) {
    // Get all knights for this side from the piece bitboards
    let knights = board.bb_pieces[side][Pieces::KNIGHT];

    // Get enemy pieces bitboard (for detecting captures)
    let enemy_pieces = board.get_enemy_pieces(side);

    // Get friendly pieces bitboard (to filter out squares we can't move to)
    let friendly_pieces = board.get_friendly_pieces(side);

    // Iterate over each knight square using the bitboard iterator
    for from_square in BitboardIter::new(knights) {
        // Get all squares this knight can attack using pre-computed attack table
        let attacks = get_knight_attacks(from_square);

        // Filter out squares occupied by friendly pieces (can't capture our own pieces)
        // This gives us valid target squares: empty squares OR enemy pieces (captures)
        let valid_targets = attacks & !friendly_pieces;

        // Iterate over each valid target square
        for to_square in BitboardIter::new(valid_targets) {
            // Check if this move is a capture (target square has enemy piece)
            let is_capture = (enemy_pieces & (1u64 << to_square)) != 0;

            // If it's a capture, get the captured piece type for the Move struct
            let captured_piece = if is_capture {
                Some(board.get_piece_on_square(to_square))
            } else {
                None
            };

            // Create and add the move to the moves vector
            moves.push(Move {
                from: from_square,
                to: to_square,
                piece: Pieces::KNIGHT,
                capture: captured_piece,
                promotion: None, // Knights never promote
                flags: MoveFlags {
                    is_capture,
                    is_promotion: false,
                    is_en_passant: false,
                    is_castling: false,
                    is_double_push: false,
                },
            });
        }
    }
}

// ============================================================================
// KING MOVE GENERATION
// ============================================================================

/// Generate all pseudo-legal king moves for the given side
///
/// Kings move one square in any direction (8 adjacent squares).
/// This function also handles castling moves.
///
/// Steps:
/// 1. Gets all kings for the active side from the board
/// 2. For each king, uses the pre-computed attack table to find target squares
/// 3. Filters out squares occupied by friendly pieces
/// 4. Creates Move structs for regular king moves
/// 5. Handles castling moves (kingside and queenside)
pub fn generate_king_moves(board: &Board, side: Side, moves: &mut Vec<Move>) {
    // Get all kings for this side from the piece bitboards
    let king = board.bb_pieces[side][Pieces::KING];

    // Get enemy pieces bitboard (for detecting captures)
    let enemy_pieces = board.get_enemy_pieces(side);

    // Get friendly pieces bitboard (to filter out squares we can't move to)
    let friendly_pieces = board.get_friendly_pieces(side);

    // Iterate over each king square using the bitboard iterator
    for from_square in BitboardIter::new(king) {
        // Get all squares this king can attack using pre-computed attack table
        let attacks = get_king_attacks(from_square);

        // Filter out squares occupied by friendly pieces (can't capture our own pieces)
        // This gives us valid target squares: empty squares OR enemy pieces (captures)
        let valid_targets = attacks & !friendly_pieces;

        // Iterate over each valid target square
        for to_square in BitboardIter::new(valid_targets) {
            // Check if this move is a capture (target square has enemy piece)
            let is_capture = (enemy_pieces & (1u64 << to_square)) != 0;

            // If it's a capture, get the captured piece type for the Move struct
            let captured_piece = if is_capture {
                Some(board.get_piece_on_square(to_square))
            } else {
                None
            };

            // Create and add the move to the moves vector
            moves.push(Move {
                from: from_square,
                to: to_square,
                piece: Pieces::KING,
                capture: captured_piece,
                promotion: None, // Kings never promote
                flags: MoveFlags {
                    is_capture,
                    is_promotion: false,
                    is_en_passant: false,
                    is_castling: false,
                    is_double_push: false,
                },
            });
        }
    }

    // ========== CASTLING MOVES ==========

    // Determine castling squares based on side
    let (
        king_square,
        kingside_rook_square,
        queenside_rook_square,
        kingside_castle_right,
        queenside_castle_right,
    ) = if side == Sides::WHITE {
        (
            Squares::E1,
            Squares::H1,
            Squares::A1,
            Castling::WK,
            Castling::WQ,
        )
    } else {
        (
            Squares::E8,
            Squares::H8,
            Squares::A8,
            Castling::BK,
            Castling::BQ,
        )
    };

    // Check if king is on starting square (castling only possible if king hasn't moved)
    let king_on_start = (board.bb_pieces[side][Pieces::KING] & (1u64 << king_square)) != 0;
    if !king_on_start {
        return; // King has moved, no castling possible
    }

    // Get occupancy bitboard to check for empty squares between king and rook
    let occupancy = board.get_occupancy();

    // KINGSIDE CASTLING
    if (board.gamestate.castling & kingside_castle_right) != 0 {
        // Check if rook is on starting square
        let rook_on_start =
            (board.bb_pieces[side][Pieces::ROOK] & (1u64 << kingside_rook_square)) != 0;

        if rook_on_start {
            // Squares between king and rook must be empty
            // For kingside: F and G squares (for white: F1=5, G1=6)
            let between_squares = if side == Sides::WHITE {
                (1u64 << Squares::F1) | (1u64 << Squares::G1)
            } else {
                (1u64 << Squares::F8) | (1u64 << Squares::G8)
            };

            let squares_empty = (occupancy & between_squares) == 0;

            if squares_empty {
                // King moves to G square (G1 for white, G8 for black)
                let king_to_square = if side == Sides::WHITE {
                    Squares::G1
                } else {
                    Squares::G8
                };

                // F square is the square the king moves through
                let f_square = if side == Sides::WHITE {
                    Squares::F1
                } else {
                    Squares::F8
                };

                let enemy_side = 1 - side;
                // King must not be in check
                let king_in_check = board.is_in_check(side);
                // F square (king moves through) must not be attacked
                let f_attacked = board.is_square_attacked(f_square, enemy_side);
                // G square (king ends up on) must not be attacked
                let g_attacked = board.is_square_attacked(king_to_square, enemy_side);

                if !king_in_check && !f_attacked && !g_attacked {
                    // Create and add the castling move
                    moves.push(Move {
                        from: king_square,
                        to: king_to_square,
                        piece: Pieces::KING,
                        capture: None,
                        promotion: None,
                        flags: MoveFlags {
                            is_capture: false,
                            is_promotion: false,
                            is_en_passant: false,
                            is_castling: true,
                            is_double_push: false,
                        },
                    });
                }
            }
        }
    }

    // QUEENSIDE CASTLING
    if (board.gamestate.castling & queenside_castle_right) != 0 {
        // Check if rook is on starting square
        let rook_on_start =
            (board.bb_pieces[side][Pieces::ROOK] & (1u64 << queenside_rook_square)) != 0;

        if rook_on_start {
            // Squares between king and rook must be empty
            // For queenside: B, C, D squares (for white: B1=1, C1=2, D1=3)
            let between_squares = if side == Sides::WHITE {
                (1u64 << Squares::B1) | (1u64 << Squares::C1) | (1u64 << Squares::D1)
            } else {
                (1u64 << Squares::B8) | (1u64 << Squares::C8) | (1u64 << Squares::D8)
            };

            let squares_empty = (occupancy & between_squares) == 0;

            if squares_empty {
                let king_to_square = if side == Sides::WHITE {
                    Squares::C1
                } else {
                    Squares::C8
                };

                let d_square = if side == Sides::WHITE {
                    Squares::D1
                } else {
                    Squares::D8
                };

                let enemy_side = 1 - side;
                // King must not be in check
                let king_in_check = board.is_in_check(side);
                // D square (king moves through) must not be attacked
                let d_attacked = board.is_square_attacked(d_square, enemy_side);
                // C square (king ends up on) must not be attacked
                let c_attacked = board.is_square_attacked(king_to_square, enemy_side);

                if !king_in_check && !d_attacked && !c_attacked {
                    // Create and add the castling move
                    moves.push(Move {
                        from: king_square,
                        to: king_to_square,
                        piece: Pieces::KING,
                        capture: None,
                        promotion: None,
                        flags: MoveFlags {
                            is_capture: false,
                            is_promotion: false,
                            is_en_passant: false,
                            is_castling: true,
                            is_double_push: false,
                        },
                    });
                }
            }
        }
    }
}

// ============================================================================
// ROOK MOVE GENERATION
// ============================================================================

/// Generate all pseudo-legal rook moves for the given side
///
/// Rooks move along ranks and files (horizontally and vertically).
/// They are sliding pieces, so they can move multiple squares until blocked.
///
/// Steps:
/// 1. Gets all rooks for the active side from the board
/// 2. Gets full occupancy bitboard (all pieces on board)
/// 3. For each rook, uses get_rook_attacks() with occupancy to find target squares
///    (this function handles blockers and stops at first piece)
/// 4. Filters out squares occupied by friendly pieces
/// 5. Creates Move structs for each valid target (empty squares or enemy captures)
pub fn generate_rook_moves(board: &Board, side: Side, moves: &mut Vec<Move>) {
    // Get all rooks for this side from the piece bitboards
    let rook = board.bb_pieces[side][Pieces::ROOK];

    // Get enemy pieces bitboard (for detecting captures)
    let enemy_pieces = board.get_enemy_pieces(side);

    // Get friendly pieces bitboard (to filter out squares we can't move to)
    let friendly_pieces = board.get_friendly_pieces(side);

    // Get full occupancy bitboard (all pieces on board) - needed for sliding piece attack calculation
    let occupancy = board.get_occupancy();

    // Iterate over each rook square using the bitboard iterator
    for from_square in BitboardIter::new(rook) {
        // Get all squares this rook can attack using pre-computed ray attacks with occupancy
        // This function handles blockers automatically and stops at the first piece
        let attacks = get_rook_attacks(from_square, occupancy);

        // Filter out squares occupied by friendly pieces (can't capture our own pieces)
        // This gives us valid target squares: empty squares OR enemy pieces (captures)
        let valid_targets = attacks & !friendly_pieces;

        // Iterate over each valid target square
        for to_square in BitboardIter::new(valid_targets) {
            // Check if this move is a capture (target square has enemy piece)
            let is_capture = (enemy_pieces & (1u64 << to_square)) != 0;

            // If it's a capture, get the captured piece type for the Move struct
            let captured_piece = if is_capture {
                Some(board.get_piece_on_square(to_square))
            } else {
                None
            };

            // Create and add the move to the moves vector
            moves.push(Move {
                from: from_square,
                to: to_square,
                piece: Pieces::ROOK,
                capture: captured_piece,
                promotion: None, // Rooks never promote
                flags: MoveFlags {
                    is_capture,
                    is_promotion: false,
                    is_en_passant: false,
                    is_castling: false,
                    is_double_push: false,
                },
            });
        }
    }
}

// ============================================================================
// BISHOP MOVE GENERATION
// ============================================================================

/// Generate all pseudo-legal bishop moves for the given side
///
/// Bishops move along diagonals (4 diagonal directions).
/// They are sliding pieces, so they can move multiple squares until blocked.
///
/// Steps:
/// 1. Gets all bishops for the active side from the board
/// 2. Gets full occupancy bitboard (all pieces on board)
/// 3. For each bishop, uses get_bishop_attacks() with occupancy to find target squares
///    (this function handles blockers and stops at first piece)
/// 4. Filters out squares occupied by friendly pieces
/// 5. Creates Move structs for each valid target (empty squares or enemy captures)
pub fn generate_bishop_moves(board: &Board, side: Side, moves: &mut Vec<Move>) {
    // Get all bishops for this side from the piece bitboards
    let bishop = board.bb_pieces[side][Pieces::BISHOP];

    // Get enemy pieces bitboard (for detecting captures)
    let enemy_pieces = board.get_enemy_pieces(side);

    // Get friendly pieces bitboard (to filter out squares we can't move to)
    let friendly_pieces = board.get_friendly_pieces(side);

    // Get full occupancy bitboard (all pieces on board) - needed for sliding piece attack calculation
    let occupancy = board.get_occupancy();

    // Iterate over each bishop square using the bitboard iterator
    for from_square in BitboardIter::new(bishop) {
        // Get all squares this bishop can attack using pre-computed ray attacks with occupancy
        // This function handles blockers automatically and stops at the first piece
        let attacks = get_bishop_attacks(from_square, occupancy);

        // Filter out squares occupied by friendly pieces (can't capture our own pieces)
        // This gives us valid target squares: empty squares OR enemy pieces (captures)
        let valid_targets = attacks & !friendly_pieces;

        // Iterate over each valid target square
        for to_square in BitboardIter::new(valid_targets) {
            // Check if this move is a capture (target square has enemy piece)
            let is_capture = (enemy_pieces & (1u64 << to_square)) != 0;

            // If it's a capture, get the captured piece type for the Move struct
            let captured_piece = if is_capture {
                Some(board.get_piece_on_square(to_square))
            } else {
                None
            };

            // Create and add the move to the moves vector
            moves.push(Move {
                from: from_square,
                to: to_square,
                piece: Pieces::BISHOP,
                capture: captured_piece,
                promotion: None, // Bishops never promote
                flags: MoveFlags {
                    is_capture,
                    is_promotion: false,
                    is_en_passant: false,
                    is_castling: false,
                    is_double_push: false,
                },
            });
        }
    }
}

// ============================================================================
// QUEEN MOVE GENERATION
// ============================================================================

/// Generate all pseudo-legal queen moves for the given side
///
/// Queens combine the movement of rooks and bishops.
/// They can move along ranks, files, and diagonals.
///
/// Steps:
/// 1. Gets all queens for the active side from the board
/// 2. Gets full occupancy bitboard (all pieces on board)
/// 3. For each queen, uses get_queen_attacks() with occupancy to find target squares
///    (this internally combines rook and bishop attacks)
/// 4. Filters out squares occupied by friendly pieces
/// 5. Creates Move structs for each valid target (empty squares or enemy captures)
pub fn generate_queen_moves(board: &Board, side: Side, moves: &mut Vec<Move>) {
    // Get all queens for this side from the piece bitboards
    let queen = board.bb_pieces[side][Pieces::QUEEN];

    // Get enemy pieces bitboard (for detecting captures)
    let enemy_pieces = board.get_enemy_pieces(side);

    // Get friendly pieces bitboard (to filter out squares we can't move to)
    let friendly_pieces = board.get_friendly_pieces(side);

    // Get full occupancy bitboard (all pieces on board) - needed for sliding piece attack calculation
    let occupancy = board.get_occupancy();

    // Iterate over each queen square using the bitboard iterator
    for from_square in BitboardIter::new(queen) {
        // Get all squares this queen can attack using pre-computed ray attacks with occupancy
        // This function internally combines rook and bishop attacks
        // It handles blockers automatically and stops at the first piece
        let attacks = get_queen_attacks(from_square, occupancy);

        // Filter out squares occupied by friendly pieces (can't capture our own pieces)
        // This gives us valid target squares: empty squares OR enemy pieces (captures)
        let valid_targets = attacks & !friendly_pieces;

        // Iterate over each valid target square
        for to_square in BitboardIter::new(valid_targets) {
            // Check if this move is a capture (target square has enemy piece)
            let is_capture = (enemy_pieces & (1u64 << to_square)) != 0;

            // If it's a capture, get the captured piece type for the Move struct
            let captured_piece = if is_capture {
                Some(board.get_piece_on_square(to_square))
            } else {
                None
            };

            // Create and add the move to the moves vector
            moves.push(Move {
                from: from_square,
                to: to_square,
                piece: Pieces::QUEEN,
                capture: captured_piece,
                promotion: None, // Queens never promote
                flags: MoveFlags {
                    is_capture,
                    is_promotion: false,
                    is_en_passant: false,
                    is_castling: false,
                    is_double_push: false,
                },
            });
        }
    }
}

// ============================================================================
// PAWN MOVE GENERATION
// ============================================================================

/// Generate all pseudo-legal pawn moves for the given side
///
/// Pawns are the most complex piece to generate moves for:
/// - Forward moves (single push, double push)
/// - Diagonal captures
/// - En passant captures
/// - Promotion (when reaching 8th/1st rank)
///
/// Steps:
/// 1. Gets all pawns for the active side from the board
/// 2. For each pawn:
///    a. Generates diagonal captures (using attack table)
///    b. Generates forward pushes (single and double)
///    c. Handles promotion (creates 4 moves: Q, R, B, N)
///    d. Generates en passant captures (if available)
pub fn generate_pawn_moves(board: &Board, side: Side, moves: &mut Vec<Move>) {
    // Get all pawns for this side from the piece bitboards
    let pawns = board.bb_pieces[side][Pieces::PAWN];

    // Get enemy pieces bitboard (for detecting captures)
    let enemy_pieces = board.get_enemy_pieces(side);

    // Get friendly pieces bitboard (to filter out squares we can't move to)
    let friendly_pieces = board.get_friendly_pieces(side);

    // Get full occupancy bitboard (all pieces on board) - needed to check if squares are empty
    let occupancy = board.get_occupancy();

    // Determine ranks and direction based on side
    // Ranks are 0-indexed: R1=0, R2=1, ..., R8=7
    let (starting_rank, ep_capture_rank) = if side == Sides::WHITE {
        (Ranks::R2, Ranks::R4) // White: starts on rank 2, captures EP from rank 4 (one rank below EP square on rank 5)
    } else {
        (Ranks::R7, Ranks::R3) // Black: starts on rank 7, captures EP from rank 3 (one rank above EP square on rank 2)
    };

    // Iterate over each pawn square using the bitboard iterator
    for from_square in BitboardIter::new(pawns) {
        let rank = from_square / 8;
        let is_on_starting_rank = rank == starting_rank;
        let is_on_ep_capture_rank = rank == ep_capture_rank;

        // ========== PAWN CAPTURES (diagonal attacks) ==========

        // Get diagonal attack squares using pre-computed attack table
        let capture_attacks = get_pawn_attacks(from_square, side);

        // Filter to only enemy pieces (captures only, not empty squares)
        let capture_targets = capture_attacks & enemy_pieces;

        // Iterate over each capture target square
        for to_square in BitboardIter::new(capture_targets) {
            // Get the captured piece type for the Move struct
            let captured_piece = Some(board.get_piece_on_square(to_square));

            // Check if this capture results in promotion
            // Promotion happens when moving TO the promotion rank (rank 8 for white, rank 1 for black)
            let to_rank = to_square / 8;
            let promotes = if side == Sides::WHITE {
                to_rank == Ranks::R8
            } else {
                to_rank == Ranks::R1
            };

            if promotes {
                // Create 4 promotion moves: Queen, Rook, Bishop, Knight
                let promotion_pieces =
                    [Pieces::QUEEN, Pieces::ROOK, Pieces::BISHOP, Pieces::KNIGHT];

                for &promo_piece in &promotion_pieces {
                    moves.push(Move {
                        from: from_square,
                        to: to_square,
                        piece: Pieces::PAWN,
                        capture: captured_piece,
                        promotion: Some(promo_piece),
                        flags: MoveFlags {
                            is_capture: true,
                            is_promotion: true,
                            is_en_passant: false,
                            is_castling: false,
                            is_double_push: false,
                        },
                    });
                }
            } else {
                // Regular capture (no promotion)
                moves.push(Move {
                    from: from_square,
                    to: to_square,
                    piece: Pieces::PAWN,
                    capture: captured_piece,
                    promotion: None,
                    flags: MoveFlags {
                        is_capture: true,
                        is_promotion: false,
                        is_en_passant: false,
                        is_castling: false,
                        is_double_push: false,
                    },
                });
            }
        }

        // ========== PAWN PUSHES (forward moves) ==========

        // Single push: one square forward
        let single_push_square = if side == Sides::WHITE {
            from_square + 8 // White moves up (higher square numbers)
        } else {
            from_square - 8 // Black moves down (lower square numbers)
        };

        // Check if single push square is on board and empty
        if single_push_square < 64 && (occupancy & (1u64 << single_push_square)) == 0 {
            // Check if this push results in promotion
            // Promotion happens when moving TO the promotion rank
            let to_rank = single_push_square / 8;
            let promotes = if side == Sides::WHITE {
                to_rank == Ranks::R8
            } else {
                to_rank == Ranks::R1
            };

            if promotes {
                // Create 4 promotion moves: Queen, Rook, Bishop, Knight
                let promotion_pieces =
                    [Pieces::QUEEN, Pieces::ROOK, Pieces::BISHOP, Pieces::KNIGHT];

                for &promo_piece in &promotion_pieces {
                    moves.push(Move {
                        from: from_square,
                        to: single_push_square,
                        piece: Pieces::PAWN,
                        capture: None,
                        promotion: Some(promo_piece),
                        flags: MoveFlags {
                            is_capture: false,
                            is_promotion: true,
                            is_en_passant: false,
                            is_castling: false,
                            is_double_push: false,
                        },
                    });
                }
            } else {
                // Regular single push (no promotion)
                moves.push(Move {
                    from: from_square,
                    to: single_push_square,
                    piece: Pieces::PAWN,
                    capture: None,
                    promotion: None,
                    flags: MoveFlags {
                        is_capture: false,
                        is_promotion: false,
                        is_en_passant: false,
                        is_castling: false,
                        is_double_push: false,
                    },
                });
            }

            // Double push: two squares forward (only from starting rank)
            if is_on_starting_rank {
                let double_push_square = if side == Sides::WHITE {
                    from_square + 16 // White moves up 2 ranks
                } else {
                    from_square - 16 // Black moves down 2 ranks
                };

                // Check if double push square is on board and empty
                if double_push_square < 64 && (occupancy & (1u64 << double_push_square)) == 0 {
                    moves.push(Move {
                        from: from_square,
                        to: double_push_square,
                        piece: Pieces::PAWN,
                        capture: None,
                        promotion: None,
                        flags: MoveFlags {
                            is_capture: false,
                            is_promotion: false,
                            is_en_passant: false,
                            is_castling: false,
                            is_double_push: true,
                        },
                    });
                }
            }
        }

        // ========== EN PASSANT ==========

        // En passant can only happen if:
        // 1. En passant square is set in game state
        // 2. Pawn is on the correct rank (5th for white, 4th for black)
        // 3. The en passant square is diagonally forward from the pawn

        if let Some(ep_square_opt) = board.gamestate.enpassant {
            if ep_square_opt > 0 && is_on_ep_capture_rank {
                let ep_square = ep_square_opt as Square;

                // Get file and rank of current pawn and EP square
                let from_file = from_square % 8;
                let from_rank = from_square / 8;
                let ep_file = ep_square % 8;
                let ep_rank = ep_square / 8;

                // Check if EP square is diagonally forward from this pawn
                // Must be: adjacent file (left or right) AND one rank forward
                let file_diff = ep_file.abs_diff(from_file);

                let rank_diff = if side == Sides::WHITE {
                    ep_rank - from_rank // White moves up (higher rank)
                } else {
                    from_rank - ep_rank // Black moves down (lower rank)
                };

                // Valid en passant: adjacent file (diff = 1) and one rank forward (diff = 1)
                let can_capture_ep = file_diff == 1 && rank_diff == 1;

                if can_capture_ep {
                    // Create and add the en passant move
                    moves.push(Move {
                        from: from_square,
                        to: ep_square,
                        piece: Pieces::PAWN,
                        capture: Some(Pieces::PAWN), // Always captures a pawn
                        promotion: None,
                        flags: MoveFlags {
                            is_capture: true,
                            is_promotion: false,
                            is_en_passant: true,
                            is_castling: false,
                            is_double_push: false,
                        },
                    });
                }
            }
        }
    }
}

/// Generate all pseudo-legal moves for the active side
///
/// Pseudo-legal moves are moves that follow piece movement rules but may leave
/// the king in check. Legal move filtering (removing moves that leave king in check)
/// should be done separately.
///
/// This function generates moves for all piece types in order:
/// 1. Pawns (most complex: pushes, captures, en passant, promotion)
/// 2. Knights (simple: L-shaped moves)
/// 3. Bishops (sliding: diagonal moves)
/// 4. Rooks (sliding: rank/file moves)
/// 5. Queens (sliding: combines rook + bishop)
/// 6. Kings (simple moves + castling)
pub fn generate_psuedo_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let side = board.gamestate.active_side;

    // Generate moves for each piece type
    // Order matters for move ordering optimization later, but for now any order works
    generate_pawn_moves(board, side, &mut moves);
    generate_knight_moves(board, side, &mut moves);
    generate_bishop_moves(board, side, &mut moves);
    generate_rook_moves(board, side, &mut moves);
    generate_queen_moves(board, side, &mut moves);
    generate_king_moves(board, side, &mut moves);

    moves
}

/// Generate all legal moves for the active side
///
/// Legal moves are pseudo-legal moves that don't leave the king in check.
/// This function filters out any moves that would leave the king in check.
pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
    let pseudo_legal = generate_psuedo_legal_moves(board);
    let mut legal = Vec::new();

    for mv in pseudo_legal {
        // Make the move
        let mut test_board = board.clone();
        let _ = test_board.make_move(&mv);

        // Check if the move leaves our king in check
        // After make_move, the active side has switched, so we check the previous side
        let side = 1 - test_board.gamestate.active_side;
        if !test_board.is_in_check(side) {
            legal.push(mv);
        }
    }

    legal
}
