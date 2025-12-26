// UCI Protocol implementation
// Based on UCI protocol specification

use crate::board::Board;
use crate::board::board::algebraic_from_str;
use crate::board::types::{Pieces, SQUARE_NAME};
use crate::defs::{self, About, Sides};
use crate::movegen::movegen::generate_legal_moves;
use crate::movegen::moves::Move;
use crate::uci::protocol::UciCommand;
use crate::utils::format_move;
use std::io::{self, BufRead, Write};

mod protocol;

/// UCI engine state
pub struct UciEngine {
    board: Board,
    search_depth: u32,
    hash_size: usize,
    should_stop: bool,
}

impl UciEngine {
    pub fn new() -> Self {
        Self {
            board: Board::build(None),
            search_depth: 6,
            hash_size: 16,
            should_stop: false,
        }
    }

    /// Main UCI loop - reads commands from stdin and responds
    pub fn run(&mut self) {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();

        for line in stdin.lock().lines() {
            match line {
                Ok(input) => {
                    let input = input.trim();
                    if let Some(command) = UciCommand::parse(input) {
                        self.handle_command(&mut stdout_lock, command);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                }
            }
        }
    }

    /// Handle a UCI command
    fn handle_command(&mut self, output: &mut impl Write, command: UciCommand) {
        match command {
            UciCommand::Uci => self.cmd_uci(output),
            UciCommand::IsReady => self.cmd_isready(output),
            UciCommand::SetOption { name, value } => {
                self.cmd_setoption(output, &name, value.as_deref())
            }
            UciCommand::UciNewGame => self.cmd_ucinewgame(),
            UciCommand::Position { fen, moves } => self.cmd_position(fen, &moves),
            UciCommand::Go(params) => self.cmd_go(output, params),
            UciCommand::Stop => self.cmd_stop(),
            UciCommand::Quit => std::process::exit(0),

            // Custom debug commands
            UciCommand::Moves { square } => self.cmd_moves(output, square),
            UciCommand::Bitboard { piece_type, color } => {
                self.cmd_bitboard(output, piece_type, color)
            }
            UciCommand::Bitboards => self.cmd_bitboards(output),
            UciCommand::Debug { enable } => self.cmd_debug(output, enable),
        }
    }

    // ========== UCI COMMANDS ==========

    fn cmd_uci(&self, output: &mut impl Write) {
        writeln!(output, "id name {}", defs::About::ENGINE).ok();
        writeln!(output, "id author {}", defs::About::AUTHOR).ok();
        writeln!(
            output,
            "option name Hash type spin default 16 min 1 max 1024"
        )
        .ok();
        writeln!(output, "uciok").ok();
        output.flush().ok();
    }

    fn cmd_isready(&self, output: &mut impl Write) {
        writeln!(output, "readyok").ok();
        output.flush().ok();
    }

    fn cmd_setoption(&mut self, output: &mut impl Write, name: &str, value: Option<&str>) {
        match name {
            "Hash" => {
                if let Some(v) = value {
                    if let Ok(size) = v.parse::<usize>() {
                        self.hash_size = size;
                        eprintln!("Hash size set to {} MB", size);
                    }
                }
            }
            _ => {
                eprintln!("Setting option: {} (not implemented)", name);
            }
        }
        output.flush().ok();
    }

    fn cmd_ucinewgame(&mut self) {
        self.board = Board::build(None);
    }

    fn cmd_position(&mut self, fen: Option<String>, moves: &[String]) {
        // Set position from FEN (if provided), otherwise use starting position
        self.board = Board::build(fen.as_deref());

        // Apply moves to reach current position
        for move_str in moves {
            // Extract source square (first 2 characters)
            if move_str.len() < 4 {
                eprintln!("Invalid move: {}", move_str);
                continue;
            }

            let from_square_str = &move_str[0..2];
            let from_square = match algebraic_from_str(from_square_str) {
                Some(sq) => sq,
                None => {
                    eprintln!("Invalid move (source): {}", move_str);
                    continue;
                }
            };

            // Parse destination (characters 2-3) and promotion (character 4+)
            let to_square_str = &move_str[2..4];
            let to_square = match algebraic_from_str(to_square_str) {
                Some(sq) => sq,
                None => {
                    eprintln!("Invalid move (destination): {}", move_str);
                    continue;
                }
            };

            // Parse promotion piece if present
            let promotion = if move_str.len() >= 5 {
                match move_str.chars().nth(4) {
                    Some('q') => Some(Pieces::QUEEN),
                    Some('r') => Some(Pieces::ROOK),
                    Some('b') => Some(Pieces::BISHOP),
                    Some('n') => Some(Pieces::KNIGHT),
                    _ => None,
                }
            } else {
                None
            };

            // Create Move struct and apply it
            let chess_move = Move {
                from: from_square,
                to: to_square,
                piece: self.board.piece_list[from_square],
                capture: if self.board.piece_list[to_square] != Pieces::NONE {
                    Some(self.board.piece_list[to_square])
                } else {
                    None
                },
                promotion,
                flags: Default::default(),
            };

            let _ = self.board.make_move(&chess_move);
        }

        eprintln!("Position set with {} moves", moves.len());
    }

    fn cmd_go(&mut self, output: &mut impl Write, params: GoParams) {
        // Reset stop flag
        self.should_stop = false;

        // Determine search depth from params
        let depth = params.depth.unwrap_or(self.search_depth);

        // Generate all legal moves for the current position
        let legal_moves = generate_legal_moves(&self.board);

        // Output search info
        writeln!(output, "info depth {} nodes {}", depth, legal_moves.len()).ok();

        // Check for game over
        if legal_moves.is_empty() {
            if self.board.is_in_check(self.board.gamestate.active_side) {
                writeln!(output, "info score mate 0").ok();
                writeln!(output, "bestmove (none)").ok();
            } else {
                writeln!(output, "info score cp 0").ok();
                writeln!(output, "bestmove (none)").ok();
            }
            output.flush().ok();
            return;
        }

        // Simple material evaluation for move ordering
        let piece_values = [
            Pieces::PAWN,    // 1
            Pieces::KNIGHT,  // 2
            Pieces::BISHOP,  // 3
            Pieces::ROOK,    // 4
            Pieces::QUEEN,   // 5
            Pieces::KING,    // 6
        ];

        let mut scored_moves: Vec<(i32, &Move)> = legal_moves
            .iter()
            .map(|mv| {
                let mut score = fastrand::i32(0..100); // Random base for variety

                // Prioritize captures
                if mv.capture.is_some() {
                    score += 100;
                    // MVV-LVA: capture more valuable pieces with less valuable ones
                    if let Some(captured) = mv.capture {
                        let victim_value = piece_values.iter().position(|&p| p == captured).unwrap_or(0);
                        let attacker_value = piece_values.iter().position(|&p| p == mv.piece).unwrap_or(0);
                        score += (victim_value as i32 - attacker_value as i32) * 10;
                    }
                }

                // Prioritize promotions to queen
                if mv.promotion == Some(Pieces::QUEEN) {
                    score += 200;
                }

                // Prioritize checks
                let mut test_board = self.board.clone();
                let _ = test_board.make_move(mv);
                let enemy_side = 1 - test_board.gamestate.active_side;
                let enemy_king_bb = test_board.bb_pieces[enemy_side][Pieces::KING];
                if enemy_king_bb != 0 {
                    let enemy_king_pos = enemy_king_bb.trailing_zeros() as usize;
                    if test_board.is_square_attacked(enemy_king_pos, test_board.gamestate.active_side) {
                        score += 50;
                    }
                }

                (score, mv)
            })
            .collect();

        // Sort by score (highest first)
        scored_moves.sort_by(|a, b| b.0.cmp(&a.0));

        // Select best move (highest score)
        let chosen_move = scored_moves[0].1;

        // Calculate simple evaluation score for info
        let mut test_board = self.board.clone();
        let _ = test_board.make_move(chosen_move);

        // Simple material count
        let side = 1 - test_board.gamestate.active_side;
        let mut score = 0;
        let piece_material = [100, 320, 330, 500, 900, 20000];

        for piece in 0..6 {
            let white_count = test_board.bb_pieces[Sides::WHITE][piece].count_ones() as i32;
            let black_count = test_board.bb_pieces[Sides::BLACK][piece].count_ones() as i32;
            score += (white_count - black_count) * piece_material[piece];
        }

        if test_board.is_in_check(side) {
            writeln!(output, "info depth {} score cp {} nodes {} pv {}", depth, score, legal_moves.len(), format_move(chosen_move)).ok();
        } else {
            writeln!(output, "info depth {} score cp {} nodes {} pv {}", depth, score, legal_moves.len(), format_move(chosen_move)).ok();
        }

        // Convert move to UCI format (e.g., "e2e4" or "e7e8q" for promotion)
        let uci_move = format_move(chosen_move);

        writeln!(output, "bestmove {}", uci_move).ok();
        output.flush().ok();
    }

    fn cmd_stop(&mut self) {
        self.should_stop = true;
        eprintln!("Stop command received");
    }

    // ========== CUSTOM DEBUG COMMANDS ==========

    fn cmd_moves(&mut self, output: &mut impl Write, square: String) {
        // Parse source square
        if let Some(from_square) = algebraic_from_str(&square) {
            // Generate all legal moves for current position
            let legal_moves = generate_legal_moves(&self.board);

            // Filter moves to only those from the specified square
            let moves_from_square: Vec<String> = legal_moves
                .iter()
                .filter(|mv| mv.from == from_square)
                .map(|mv| SQUARE_NAME[mv.to].to_string())
                .collect();

            // Return moves in format: "moves e2 e3 e4"
            let moves_str = format!("moves {} {}", square, moves_from_square.join(" "));
            writeln!(output, "{}", moves_str).ok();
        } else {
            // Invalid square, return just the square as placeholder
            writeln!(output, "moves {}", square).ok();
        }

        output.flush().ok();
    }

    fn cmd_bitboard(&mut self, output: &mut impl Write, piece_type: String, color: String) {
        // Parse piece type and color
        let piece = match piece_type.as_str() {
            "pawn" => Pieces::PAWN,
            "knight" => Pieces::KNIGHT,
            "bishop" => Pieces::BISHOP,
            "rook" => Pieces::ROOK,
            "queen" => Pieces::QUEEN,
            "king" => Pieces::KING,
            _ => {
                eprintln!("Unknown piece type: {}", piece_type);
                writeln!(output, "info bitboard {} {} 0", piece_type, color).ok();
                output.flush().ok();
                return;
            }
        };

        let side = match color.as_str() {
            "white" => Sides::WHITE,
            "black" => Sides::BLACK,
            _ => {
                eprintln!("Unknown color: {}", color);
                writeln!(output, "info bitboard {} {} 0", piece_type, color).ok();
                output.flush().ok();
                return;
            }
        };

        // Get bitboard from internal board representation
        let bitboard = self.board.bb_pieces[side][piece];

        // Output in hex format
        writeln!(
            output,
            "info bitboard {} {} {:016x}",
            piece_type, color, bitboard
        )
        .ok();
        output.flush().ok();
    }

    fn cmd_bitboards(&mut self, output: &mut impl Write) {
        // Get bitboards for all piece types and colors
        let pieces = ["pawn", "knight", "bishop", "rook", "queen", "king"];
        let colors = ["white", "black"];

        for color in &colors {
            let side = match *color {
                "white" => Sides::WHITE,
                "black" => Sides::BLACK,
                _ => continue,
            };

            for piece in &pieces {
                let piece_type = match *piece {
                    "pawn" => Pieces::PAWN,
                    "knight" => Pieces::KNIGHT,
                    "bishop" => Pieces::BISHOP,
                    "rook" => Pieces::ROOK,
                    "queen" => Pieces::QUEEN,
                    "king" => Pieces::KING,
                    _ => continue,
                };

                let bitboard = self.board.bb_pieces[side][piece_type];

                writeln!(
                    output,
                    "info bitboard {} {} {:016x}",
                    piece, color, bitboard
                )
                .ok();
            }
        }
        output.flush().ok();
    }

    fn cmd_debug(&mut self, output: &mut impl Write, enable: bool) {
        // TODO: Implement debug mode
        eprintln!("Debug mode: {}", enable);
        writeln!(output, "info debug {}", enable).ok();
        output.flush().ok();
    }
}
// ========== PUBLIC ENTRY POINT ==========

pub fn run_uci() {
    let mut engine = UciEngine::new();
    engine.run();
}

// ========== TYPES ==========

/// Parameters for 'go' command
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GoParams {
    pub depth: Option<u32>,
    pub movetime: Option<u32>,
    pub wtime: Option<u32>,
    pub btime: Option<u32>,
    pub winc: Option<u32>,
    pub binc: Option<u32>,
    pub nodes: Option<u32>,
    pub infinite: bool,
}
