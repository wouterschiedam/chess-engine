// Performance testing (perft) implementation
//
// Perft (Performance Test) is used to verify move generation correctness
// by counting the number of legal move sequences at each depth and comparing
// against known values from chess programming resources.

use crate::board::types::Pieces;
use crate::board::{Board, types::SQUARE_NAME};
use crate::movegen::{Move, generate_legal_moves};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn format_move(mv: &Move) -> String {
    let from = SQUARE_NAME[mv.from];
    let to = SQUARE_NAME[mv.to];

    if let Some(promo) = mv.promotion {
        let promo_char = match promo {
            Pieces::QUEEN => "q",
            Pieces::ROOK => "r",
            Pieces::BISHOP => "b",
            Pieces::KNIGHT => "n",
            _ => "",
        };
        format!("{}{}{}", from, to, promo_char)
    } else {
        format!("{}{}", from, to)
    }
}

/// Convert Board to FEN string
fn board_to_fen(board: &Board) -> String {
    use crate::board::types::{Pieces, SQUARE_NAME};
    use crate::defs::{Castling, Sides};

    let mut fen = String::new();

    // Piece placement - iterate through bitboards to find pieces
    for rank in (0..8).rev() {
        let mut empty_count = 0;
        for file in 0..8 {
            let square = rank * 8 + file;
            let square_mask = 1u64 << square;

            // Find which piece (if any) is on this square
            let mut found_piece = false;
            for piece_type in 0..crate::defs::NrOf::PIECE_TYPES {
                if (board.bb_pieces[Sides::WHITE][piece_type] & square_mask) != 0 {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    let piece_char = match piece_type {
                        Pieces::KING => 'K',
                        Pieces::QUEEN => 'Q',
                        Pieces::ROOK => 'R',
                        Pieces::BISHOP => 'B',
                        Pieces::KNIGHT => 'N',
                        Pieces::PAWN => 'P',
                        _ => '?',
                    };
                    fen.push(piece_char);
                    found_piece = true;
                    break;
                } else if (board.bb_pieces[Sides::BLACK][piece_type] & square_mask) != 0 {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    let piece_char = match piece_type {
                        Pieces::KING => 'k',
                        Pieces::QUEEN => 'q',
                        Pieces::ROOK => 'r',
                        Pieces::BISHOP => 'b',
                        Pieces::KNIGHT => 'n',
                        Pieces::PAWN => 'p',
                        _ => '?',
                    };
                    fen.push(piece_char);
                    found_piece = true;
                    break;
                }
            }

            if !found_piece {
                empty_count += 1;
            }
        }
        if empty_count > 0 {
            fen.push_str(&empty_count.to_string());
        }
        if rank > 0 {
            fen.push('/');
        }
    }

    // Active color
    fen.push(' ');
    fen.push(if board.gamestate.active_side == Sides::WHITE {
        'w'
    } else {
        'b'
    });

    // Castling rights
    fen.push(' ');
    let mut castling = String::new();
    if board.gamestate.castling & Castling::WK != 0 {
        castling.push('K');
    }
    if board.gamestate.castling & Castling::WQ != 0 {
        castling.push('Q');
    }
    if board.gamestate.castling & Castling::BK != 0 {
        castling.push('k');
    }
    if board.gamestate.castling & Castling::BQ != 0 {
        castling.push('q');
    }
    if castling.is_empty() {
        fen.push('-');
    } else {
        fen.push_str(&castling);
    }

    // En passant
    fen.push(' ');
    if let Some(ep) = board.gamestate.enpassant {
        if ep > 0 && ep < 64 {
            let ep_square = ep as usize;
            fen.push_str(SQUARE_NAME[ep_square]);
        } else {
            fen.push('-');
        }
    } else {
        fen.push('-');
    }

    // Halfmove clock
    fen.push(' ');
    fen.push_str(&board.gamestate.halfclockmove.to_string());

    // Fullmove number
    fen.push(' ');
    fen.push_str(&board.gamestate.fullmovenumber.to_string());

    fen
}

/// Get perft values from Stockfish for a given position and depth
fn get_stockfish_perft(fen: &str, depth: usize) -> Option<HashMap<String, usize>> {
    let mut child = match Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            eprintln!("Error: Could not start Stockfish. Make sure 'stockfish' is in your PATH.");
            return None;
        }
    };

    let mut stdin = child.stdin.take()?;

    // Send UCI commands to Stockfish
    writeln!(stdin, "uci").ok()?;
    writeln!(stdin, "position fen {}", fen).ok()?;
    writeln!(stdin, "go perft {}", depth).ok()?;
    writeln!(stdin, "quit").ok()?;
    drop(stdin);

    // Wait for child to finish and get output
    let output = child.wait_with_output().ok()?;
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut perft_values = HashMap::new();

    for line in stdout_str.lines() {
        let line = line.trim();

        // Skip UCI protocol lines and empty lines
        if line.is_empty()
            || line.starts_with("info")
            || line.starts_with("Stockfish")
            || line.starts_with("id ")
            || line.starts_with("option ")
            || line.starts_with("uciok")
            || line.starts_with("readyok")
            || line.starts_with("Nodes searched:")
            || line.starts_with("Available processors:")
            || line.starts_with("Using ")
            || line.starts_with("NNUE evaluation")
        {
            continue;
        }

        // Stockfish outputs perft in format: "a2a3: 380"
        if let Some(colon_pos) = line.find(':') {
            let move_str = line[..colon_pos].trim().to_string();
            // Only parse lines that look like moves (start with a-h, 4+ chars)
            if move_str.len() >= 4 {
                let first_char = move_str.chars().next().unwrap();
                if first_char.is_ascii_lowercase() && ('a'..='h').contains(&first_char) {
                    if let Ok(count) = line[colon_pos + 1..].trim().parse::<usize>() {
                        perft_values.insert(move_str, count);
                    }
                }
            }
        }
    }

    if perft_values.is_empty() {
        None
    } else {
        Some(perft_values)
    }
}

// ============================================================================
// CORE PERFT FUNCTION
// ============================================================================

/// Perft (Performance Test) - counts the number of legal moves at each depth
pub fn perft(board: &mut Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board);

    if depth == 1 {
        return moves.len();
    }

    let mut nodes = 0;
    for mv in moves {
        let move_info = board.make_move(&mv);
        nodes += perft(board, depth - 1);
        board.unmake_move(&mv, &move_info);
    }

    nodes
}

// ============================================================================
// DEBUGGING FUNCTIONS
// ============================================================================

/// Shows each move and its node count
pub fn perft_verbose(board: &mut Board, depth: usize) -> usize {
    let start = Instant::now();

    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board);

    if depth == 1 {
        for mv in &moves {
            println!("{}: 1", format_move(mv));
        }
        let elapsed = start.elapsed();
        let nodes_per_sec = if elapsed.as_secs_f64() > 0.0 {
            moves.len() as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        println!("\nNodes: {}", moves.len());
        println!("Time: {:.3}s", elapsed.as_secs_f64());
        println!("Nodes/sec: {:.0}", nodes_per_sec);
        return moves.len();
    }

    let mut total_nodes = 0;
    for mv in moves {
        let move_info = board.make_move(&mv);
        let nodes = perft(board, depth - 1);
        total_nodes += nodes;

        println!("{}: {}", format_move(&mv), nodes);

        board.unmake_move(&mv, &move_info);
    }

    let elapsed = start.elapsed();
    let nodes_per_sec = if elapsed.as_secs_f64() > 0.0 {
        total_nodes as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };
    println!("\nNodes: {}", total_nodes);
    println!("Time: {:.3}s", elapsed.as_secs_f64());
    println!("Nodes/sec: {:.0}", nodes_per_sec);

    total_nodes
}

/// Shows breakdown for each move (standard debugging tool)
pub fn divide(board: &mut Board, depth: usize) -> usize {
    let start = Instant::now();

    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board);
    let mut total_nodes = 0;

    for mv in moves {
        let move_info = board.make_move(&mv);
        let nodes = perft(board, depth - 1);
        total_nodes += nodes;

        println!("{}: {}", format_move(&mv), nodes);

        board.unmake_move(&mv, &move_info);
    }

    let elapsed = start.elapsed();
    let nodes_per_sec = if elapsed.as_secs_f64() > 0.0 {
        total_nodes as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    println!("\nTotal: {}", total_nodes);
    println!("Time: {:.3}s", elapsed.as_secs_f64());
    println!("Nodes/sec: {:.0}", nodes_per_sec);

    total_nodes
}

/// Compares each move against Stockfish perft values (best debugging tool)
pub fn divide_with_known(board: &mut Board, depth: usize, print_position: bool) {
    let fen = board_to_fen(board);

    println!("Querying Stockfish for perft values...");
    let known_values = match get_stockfish_perft(&fen, depth) {
        Some(values) => values,
        None => {
            println!("Error: Could not get perft values from Stockfish.");
            println!("Falling back to regular divide:");
            println!("{}", "=".repeat(50));
            divide(board, depth);
            return;
        }
    };

    println!("Divide with Stockfish values (depth {}):", depth);
    println!("{}", "=".repeat(50));

    let start = Instant::now();
    let moves = generate_legal_moves(board);
    let mut total_nodes = 0;
    let mut correct_moves = 0;
    let mut incorrect_moves = 0;
    let mut expected_total = 0;

    for mv in moves {
        let move_str = format_move(&mv);
        let move_info = board.make_move(&mv);
        let nodes = perft(board, depth - 1);
        total_nodes += nodes;

        let mut has_discrepancy = false;
        if let Some(&expected) = known_values.get(&move_str) {
            expected_total += expected;
            if nodes == expected {
                println!("{}: {} ✓", move_str, nodes);
                correct_moves += 1;
            } else {
                println!(
                    "{}: {} ✗ (expected {}) - DIFF: {}",
                    move_str,
                    nodes,
                    expected,
                    nodes as i64 - expected as i64
                );
                incorrect_moves += 1;
                has_discrepancy = true;
            }
        } else {
            println!("{}: {} (no Stockfish value)", move_str, nodes);
        }

        board.unmake_move(&mv, &move_info);
    }

    let elapsed = start.elapsed();
    let nodes_per_sec = if elapsed.as_secs_f64() > 0.0 {
        total_nodes as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    println!("\nTotal: {}", total_nodes);
    if expected_total > 0 {
        if total_nodes == expected_total {
            println!("✓ Total matches Stockfish: {}", expected_total);
        } else {
            println!(
                "✗ Total expected: {} (difference: {})",
                expected_total,
                total_nodes as i64 - expected_total as i64
            );
        }
    }

    println!("Time: {:.3}s", elapsed.as_secs_f64());
    println!("Nodes/sec: {:.0}", nodes_per_sec);

    if incorrect_moves > 0 {
        println!(
            "\nSummary: {} correct, {} incorrect",
            correct_moves, incorrect_moves
        );
    }
}

/// Shows breakdown after making a specific move
pub fn drill_down(
    board: &mut Board,
    move_str: &str,
    depth: usize,
    use_stockfish: bool,
    print_position: bool,
) {
    let moves = generate_legal_moves(board);
    let target_move = moves.iter().find(|mv| format_move(mv) == move_str);

    match target_move {
        Some(mv) => {
            println!("Drilling down into move: {}", move_str);
            println!("After {}:", move_str);
            println!("{}", "=".repeat(50));

            let move_info = board.make_move(mv);

            // Print position if requested
            if print_position {
                use crate::utils::display::print_position;
                println!("\nPosition after {}:", move_str);
                print_position(board, false, None);
                println!();
            }

            // Get Stockfish values if requested
            let stockfish_values = if use_stockfish {
                let fen = board_to_fen(board);
                get_stockfish_perft(&fen, depth.saturating_sub(1))
            } else {
                None
            };

            let start = Instant::now();
            let next_moves = generate_legal_moves(board);
            let mut total = 0;
            let mut expected_total = 0;
            let mut correct_moves = 0;
            let mut incorrect_moves = 0;

            for next_mv in next_moves {
                let next_move_str = format_move(&next_mv);
                let next_move_info = board.make_move(&next_mv);
                // After making drill move and next move, we have depth-2 remaining
                let remaining_depth = depth.saturating_sub(2);
                let nodes = perft(board, remaining_depth);
                total += nodes;

                // Compare with Stockfish if available
                let mut has_discrepancy = false;
                if let Some(ref stockfish) = stockfish_values {
                    if let Some(&expected) = stockfish.get(&next_move_str) {
                        expected_total += expected;
                        if nodes == expected {
                            println!("{}: {} ✓", next_move_str, nodes);
                            correct_moves += 1;
                        } else {
                            let diff = nodes as i64 - expected as i64;
                            println!(
                                "{}: {} ✗ (expected {}) - DIFF: {}",
                                next_move_str, nodes, expected, diff
                            );
                            incorrect_moves += 1;
                            has_discrepancy = true;
                        }
                    } else {
                        println!("{}: {} (no Stockfish value)", next_move_str, nodes);
                    }
                } else {
                    println!("{}: {}", next_move_str, nodes);
                }

                board.unmake_move(&next_mv, &next_move_info);
            }

            let elapsed = start.elapsed();
            let nodes_per_sec = if elapsed.as_secs_f64() > 0.0 {
                total as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            println!("\nTotal: {}", total);
            if use_stockfish && expected_total > 0 {
                if total == expected_total {
                    println!("✓ Total matches Stockfish: {}", expected_total);
                } else {
                    println!(
                        "✗ Total expected: {} (difference: {})",
                        expected_total,
                        total as i64 - expected_total as i64
                    );
                }
                if incorrect_moves > 0 {
                    println!(
                        "Summary: {} correct, {} incorrect",
                        correct_moves, incorrect_moves
                    );
                }
            }
            println!("Time: {:.3}s", elapsed.as_secs_f64());
            println!("Nodes/sec: {:.0}", nodes_per_sec);
            board.unmake_move(mv, &move_info);
        }
        None => {
            println!("Move '{}' not found in current position", move_str);
            println!("\nAvailable moves:");
            for mv in moves {
                println!("  {}", format_move(&mv));
            }
        }
    }
}
