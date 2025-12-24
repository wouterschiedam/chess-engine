use std::ops::RangeInclusive;

use super::types::{BB_SQUARES, Files, Pieces, Ranks, SQUARE_NAME, Squares};
use crate::{
    board::bitboard::BitboardIter,
    defs::{
        Bitboard, Castling, EMPTY, FEN_START_POSITION, MAX_GAME_MOVES, MAX_MOVE_RULE, NrOf, Piece,
        Side, Sides, Square,
    },
    movegen::{
        Move,
        attacks::{
            get_bishop_attacks, get_king_attacks, get_knight_attacks, get_pawn_attacks,
            get_queen_attacks, get_rook_attacks,
        },
        moves::MoveInfo,
    },
};

#[derive(Clone)]
pub struct GameState {
    pub active_side: usize,
    pub castling: u8,
    pub enpassant: Option<u8>,
    pub halfclockmove: u8,
    pub fullmovenumber: u16,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            active_side: 0,
            castling: 0,
            enpassant: Some(0),
            halfclockmove: 0,
            fullmovenumber: 0,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub bb_pieces: [[Bitboard; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub bb_side: [Bitboard; Sides::BOTH],
    pub piece_list: [Piece; NrOf::SQUARES],
    pub gamestate: GameState,
}

impl Board {
    pub fn new() -> Self {
        Self {
            bb_pieces: [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH],
            bb_side: [EMPTY; Sides::BOTH],
            piece_list: [Pieces::NONE; NrOf::SQUARES],
            gamestate: GameState::new(),
        }
    }

    pub fn build(fen: Option<&str>) -> Self {
        let mut board = Self::new();
        let _ = board.read_fen(fen);
        board
    }

    fn reset(&mut self) {
        self.bb_pieces = [[0; NrOf::PIECE_TYPES]; Sides::BOTH];
        self.bb_side = [EMPTY; Sides::BOTH];
        self.piece_list = [Pieces::NONE; NrOf::SQUARES];
        self.gamestate = GameState::new()
    }

    // Initialize board
    pub fn init(&mut self) {
        // Get bitboard for both white and black based on starting pos
        let pieces_per_side = self.init_pieces_per_side_bb();
        self.bb_side[Sides::WHITE] = pieces_per_side.0;
        self.bb_side[Sides::BLACK] = pieces_per_side.1;
    }

    fn init_pieces_per_side_bb(&self) -> (Bitboard, Bitboard) {
        let mut bb_white: Bitboard = 0;
        let mut bb_black: Bitboard = 0;

        for (bb_w, bb_b) in self.bb_pieces[Sides::WHITE]
            .iter()
            .zip(self.bb_pieces[Sides::BLACK].iter())
        {
            bb_white |= *bb_w;
            bb_black |= *bb_b;
        }

        (bb_white, bb_black)
    }
}

// FEN parsing implementation
#[allow(dead_code)]
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
#[allow(dead_code)]
const SPLITTER: char = '/';
#[allow(dead_code)]
const DASH: char = '-';
#[allow(dead_code)]
const EM_DASH: char = '–';
#[allow(dead_code)]
const SPACE: char = ' ';
#[allow(dead_code)]
const W_OR_B: &str = "wb";
#[allow(dead_code)]
const NUMBER_OF_FEN_PARTS: usize = 6;
#[allow(dead_code)]
const EP_SQUARES_WHITE: RangeInclusive<Square> = Squares::A3..=Squares::H3;
#[allow(dead_code)]
const EP_SQUARES_BLACK: RangeInclusive<Square> = Squares::A6..=Squares::H6;

#[allow(dead_code)]
type FenResult = Result<(), u8>;
#[allow(dead_code)]
type FenParser = fn(board: &mut Board, part: &str) -> bool;

impl Board {
    pub fn get_occupancy(&self) -> Bitboard {
        self.bb_side[Sides::WHITE] | self.bb_side[Sides::BLACK]
    }

    pub fn get_friendly_pieces(&self, side: Side) -> Bitboard {
        self.bb_side[side]
    }

    pub fn get_enemy_pieces(&self, side: Side) -> Bitboard {
        self.bb_side[1 - side] // Flip side: 0->1, 1->0
    }

    pub fn is_square_empty(&self, square: Square) -> bool {
        (self.get_occupancy() & (1u64 << square)) == 0
    }

    pub fn is_enemy_square(&self, square: Square, side: Side) -> bool {
        (self.get_enemy_pieces(side) & (1u64 << square)) == 0
    }

    pub fn get_piece_on_square(&self, square: Square) -> Piece {
        self.piece_list[square]
    }

    // ============================================================================
    // CHECK DETECTION
    // ============================================================================

    /// Check if a square is attacked by the given side
    ///
    /// This function checks if any piece of the attacking side can attack
    /// the target square. Used for check detection and castling validation.
    ///
    /// # Arguments
    /// * `square` - The square to check
    /// * `by_side` - The side that might be attacking (0=white, 1=black)
    ///
    /// # Returns
    /// `true` if the square is attacked by any piece of the given side
    pub fn is_square_attacked(&self, square: Square, by_side: Side) -> bool {
        let enemy_pieces = self.bb_pieces[by_side];
        let occupancy = self.get_occupancy();

        // Check pawn attacks
        let pawn_attacks = get_pawn_attacks(square, by_side);
        if (pawn_attacks & enemy_pieces[Pieces::PAWN]) != 0 {
            return true;
        }

        // Check knight attacks
        let knight_attacks = get_knight_attacks(square);
        if (knight_attacks & enemy_pieces[Pieces::KNIGHT]) != 0 {
            return true;
        }

        // Check king attacks
        let king_attacks = get_king_attacks(square);
        if (king_attacks & enemy_pieces[Pieces::KING]) != 0 {
            return true;
        }

        // Check rook/queen attacks (ranks and files)
        let rook_attacks = get_rook_attacks(square, occupancy);
        if (rook_attacks & (enemy_pieces[Pieces::ROOK] | enemy_pieces[Pieces::QUEEN])) != 0 {
            return true;
        }

        // Check bishop/queen attacks (diagonals)
        let bishop_attacks = get_bishop_attacks(square, occupancy);
        if (bishop_attacks & (enemy_pieces[Pieces::BISHOP] | enemy_pieces[Pieces::QUEEN])) != 0 {
            return true;
        }

        false
    }

    /// Check if the given side's king is in check
    ///
    /// This function finds the king square and checks if it's attacked
    /// by any enemy piece.
    ///
    /// # Arguments
    /// * `side` - The side to check (0=white, 1=black)
    ///
    /// # Returns
    /// `true` if the king is in check
    pub fn is_in_check(&self, side: Side) -> bool {
        // Find the king square
        let king_bb = self.bb_pieces[side][Pieces::KING];
        if king_bb == 0 {
            // No king found (shouldn't happen in valid positions, but handle gracefully)
            return false;
        }

        // Get the first (and should be only) king square
        let king_square = BitboardIter::new(king_bb)
            .next()
            .expect("King bitboard should have exactly one bit set");

        // Check if the king square is attacked by the enemy
        let enemy_side = 1 - side;
        self.is_square_attacked(king_square, side)
    }

    pub fn make_move(&mut self, mv: &Move) -> MoveInfo {
        let side: Side = self.gamestate.active_side;
        let from = mv.from;
        let to = mv.to;

        // Store information for undo
        let mut move_info = MoveInfo {
            captured_piece: mv.capture,
            captured_square: if mv.flags.is_capture { Some(to) } else { None },
            previous_castling: self.gamestate.castling,
            previous_enpassant: self.gamestate.enpassant,
            previous_halfmove_clock: self.gamestate.halfclockmove,
            promotion_piece: mv.promotion,
            rook_from: None,
            rook_to: None,
            en_passant_captured_square: None,
        };

        // ========== REMOVE PIECE FROM SOURCE SQUARE ==========
        // Clear piece from source square in bitboards
        let from_mask = 1u64 << from;
        self.bb_pieces[side][mv.piece] &= !from_mask;
        self.bb_side[side] &= !from_mask;
        self.piece_list[from] = Pieces::NONE;

        // ========== HANDLE CAPTURES ==========
        // Note: En passant is handled separately below, so skip regular capture handling for EP
        if mv.flags.is_capture && !mv.flags.is_en_passant {
            if let Some(captured_piece) = mv.capture {
                // Validate captured_piece is a valid piece type (0-5, not Pieces::NONE which is 6)
                if captured_piece < NrOf::PIECE_TYPES {
                    // Remove captured piece from bitboards
                    let to_mask = 1u64 << to;
                    self.bb_pieces[1 - side][captured_piece] &= !to_mask;
                    self.bb_side[1 - side] &= !to_mask;
                    self.piece_list[to] = Pieces::NONE;
                }
            }
        }

        // ========== HANDLE EN PASSANT ==========
        if mv.flags.is_en_passant {
            // En passant: captured pawn is on the square behind the destination
            let captured_square = if side == Sides::WHITE {
                to - 8 // White captures down
            } else {
                to + 8 // Black captures up
            };

            move_info.en_passant_captured_square = Some(captured_square);

            // Remove captured pawn
            let captured_mask = 1u64 << captured_square;
            self.bb_pieces[1 - side][Pieces::PAWN] &= !captured_mask;
            self.bb_side[1 - side] &= !captured_mask;
            self.piece_list[captured_square] = Pieces::NONE;
        }

        // ========== PLACE PIECE ON DESTINATION SQUARE ==========
        let to_mask = 1u64 << to;
        let piece_to_place = mv.promotion.unwrap_or(mv.piece);

        self.bb_pieces[side][piece_to_place] |= to_mask;
        self.bb_side[side] |= to_mask;
        self.piece_list[to] = piece_to_place;

        // ========== HANDLE CASTLING ==========
        if mv.flags.is_castling {
            // Determine rook squares based on side and castling direction
            let (rook_from, rook_to) = if side == Sides::WHITE {
                if to == Squares::G1 {
                    // Kingside: rook from H1 to F1
                    (Squares::H1, Squares::F1)
                } else {
                    // Queenside: rook from A1 to D1
                    (Squares::A1, Squares::D1)
                }
            } else {
                // Black side
                if to == Squares::G8 {
                    // Kingside: rook from H8 to F8
                    (Squares::H8, Squares::F8)
                } else {
                    // Queenside: rook from A8 to D8
                    (Squares::A8, Squares::D8)
                }
            };

            move_info.rook_from = Some(rook_from);
            move_info.rook_to = Some(rook_to);

            // Move rook
            let rook_from_mask = 1u64 << rook_from;
            let rook_to_mask = 1u64 << rook_to;

            self.bb_pieces[side][Pieces::ROOK] &= !rook_from_mask;
            self.bb_pieces[side][Pieces::ROOK] |= rook_to_mask;
            self.bb_side[side] &= !rook_from_mask;
            self.bb_side[side] |= rook_to_mask;
            self.piece_list[rook_from] = Pieces::NONE;
            self.piece_list[rook_to] = Pieces::ROOK;
        }

        // ========== UPDATE CASTLING RIGHTS ==========
        // If king moves, lose all castling rights
        if mv.piece == Pieces::KING {
            if side == Sides::WHITE {
                self.gamestate.castling &= !(Castling::WK | Castling::WQ);
            } else {
                self.gamestate.castling &= !(Castling::BK | Castling::BQ);
            }
        }

        // If rook moves from starting square, lose that castling right
        if mv.piece == Pieces::ROOK {
            if side == Sides::WHITE {
                if from == Squares::H1 {
                    self.gamestate.castling &= !Castling::WK;
                } else if from == Squares::A1 {
                    self.gamestate.castling &= !Castling::WQ;
                }
            } else {
                // Black side
                if from == Squares::H8 {
                    self.gamestate.castling &= !Castling::BK;
                } else if from == Squares::A8 {
                    self.gamestate.castling &= !Castling::BQ;
                }
            }
        }

        // If enemy rook is captured, lose that castling right
        if mv.flags.is_capture {
            if to == Squares::H1 {
                self.gamestate.castling &= !Castling::WK;
            } else if to == Squares::A1 {
                self.gamestate.castling &= !Castling::WQ;
            } else if to == Squares::H8 {
                self.gamestate.castling &= !Castling::BK;
            } else if to == Squares::A8 {
                self.gamestate.castling &= !Castling::BQ;
            }
        }

        // ========== UPDATE EN PASSANT ==========
        if mv.flags.is_double_push {
            // Set en passant square (square behind the pawn that just moved)
            let ep_square = if side == Sides::WHITE {
                to - 8 // Square behind white pawn
            } else {
                to + 8 // Square behind black pawn
            };
            self.gamestate.enpassant = Some(ep_square as u8);
        } else {
            self.gamestate.enpassant = None;
        }

        // ========== UPDATE HALFMOVE CLOCK ==========
        if mv.flags.is_capture || mv.piece == Pieces::PAWN {
            // Reset on capture or pawn move
            self.gamestate.halfclockmove = 0;
        } else {
            self.gamestate.halfclockmove += 1;
        }

        // ========== UPDATE FULLMOVE NUMBER ==========
        if side == Sides::BLACK {
            self.gamestate.fullmovenumber += 1;
        }

        // ========== SWITCH ACTIVE SIDE ==========
        self.gamestate.active_side = 1 - side;

        move_info
    }

    /// Restores the board to the state before the move was made
    pub fn unmake_move(&mut self, mv: &Move, move_info: &MoveInfo) {
        let side = 1 - self.gamestate.active_side; // Previous side (before switch)
        let from = mv.from;
        let to = mv.to;

        // ========== SWITCH ACTIVE SIDE BACK ==========
        self.gamestate.active_side = side;

        // ========== RESTORE FULLMOVE NUMBER ==========
        if side == Sides::BLACK {
            self.gamestate.fullmovenumber -= 1;
        }

        // ========== RESTORE HALFMOVE CLOCK ==========
        self.gamestate.halfclockmove = move_info.previous_halfmove_clock;

        // ========== RESTORE EN PASSANT ==========
        self.gamestate.enpassant = move_info.previous_enpassant;

        // ========== RESTORE CASTLING RIGHTS ==========
        self.gamestate.castling = move_info.previous_castling;

        // ========== REMOVE PIECE FROM DESTINATION SQUARE ==========
        let to_mask = 1u64 << to;
        let piece_to_remove = move_info.promotion_piece.unwrap_or(mv.piece);

        self.bb_pieces[side][piece_to_remove] &= !to_mask;
        self.bb_side[side] &= !to_mask;
        self.piece_list[to] = Pieces::NONE;

        // ========== RESTORE PIECE TO SOURCE SQUARE ==========
        let from_mask = 1u64 << from;
        self.bb_pieces[side][mv.piece] |= from_mask;
        self.bb_side[side] |= from_mask;
        self.piece_list[from] = mv.piece;

        // ========== RESTORE CASTLING ROOK ==========
        if mv.flags.is_castling {
            if let (Some(rook_from), Some(rook_to)) = (move_info.rook_from, move_info.rook_to) {
                let rook_from_mask = 1u64 << rook_from;
                let rook_to_mask = 1u64 << rook_to;

                self.bb_pieces[side][Pieces::ROOK] &= !rook_to_mask;
                self.bb_pieces[side][Pieces::ROOK] |= rook_from_mask;
                self.bb_side[side] &= !rook_to_mask;
                self.bb_side[side] |= rook_from_mask;
                self.piece_list[rook_to] = Pieces::NONE;
                self.piece_list[rook_from] = Pieces::ROOK;
            }
        }

        // ========== RESTORE CAPTURED PIECE ==========
        if mv.flags.is_capture {
            if let Some(captured_piece) = move_info.captured_piece {
                // Validate captured_piece is a valid piece type (0-5, not Pieces::NONE which is 6)
                if captured_piece < NrOf::PIECE_TYPES {
                    let captured_square = if mv.flags.is_en_passant {
                        move_info.en_passant_captured_square.unwrap()
                    } else {
                        to
                    };

                    let captured_mask = 1u64 << captured_square;
                    self.bb_pieces[1 - side][captured_piece] |= captured_mask;
                    self.bb_side[1 - side] |= captured_mask;
                    self.piece_list[captured_square] = captured_piece;
                }
            }
        }
    }

    pub fn read_fen(&mut self, fen_string: Option<&str>) -> FenResult {
        // Split the string into parts. There should be 6 parts.
        let fen_parts: Vec<String> = match fen_string {
            Some(f) => f,
            None => FEN_START_POSITION,
        }
        .replace(EM_DASH, DASH.encode_utf8(&mut [0; 4]))
        .split(SPACE)
        .map(|s| s.to_string())
        .collect();

        // Check the number of fen parts.
        let nr_of_parts_ok = fen_parts.len() == NUMBER_OF_FEN_PARTS;

        // Set the initial result.
        let mut result: FenResult = if nr_of_parts_ok { Ok(()) } else { Err(0) };
        if nr_of_parts_ok {
            let fen_parser: [FenParser; 6] = [
                pieces,
                color,
                castling,
                enpassant,
                halfmoveclock,
                fullmovenumber,
            ];

            let mut new_board = self.clone();
            new_board.reset();

            let mut i: usize = 0;
            // validate all parts of the FEN string
            while i < NUMBER_OF_FEN_PARTS && result == Ok(()) {
                let parser = &fen_parser[i];
                let part = &fen_parts[i];
                let is_ok = parser(&mut new_board, part);
                result = if is_ok { Ok(()) } else { Err(i as u8) };
                i += 1;
            }

            if result == Ok(()) {
                new_board.init();
                *self = new_board;
            }
        }
        result
    }
}

#[allow(dead_code)]
pub fn algebraic_from_str(square: &str) -> Option<usize> {
    SQUARE_NAME.iter().position(|&element| element == square)
}

#[allow(dead_code)]
fn pieces(board: &mut Board, part: &str) -> bool {
    let mut rank = Ranks::R8 as usize;
    let mut file = Files::A as usize;

    let mut result = true;

    for c in part.chars() {
        let square = rank * 8 + file;

        match c {
            'k' => board.bb_pieces[Sides::BLACK][Pieces::KING] |= BB_SQUARES[square],
            'q' => board.bb_pieces[Sides::BLACK][Pieces::QUEEN] |= BB_SQUARES[square],
            'r' => board.bb_pieces[Sides::BLACK][Pieces::ROOK] |= BB_SQUARES[square],
            'b' => board.bb_pieces[Sides::BLACK][Pieces::BISHOP] |= BB_SQUARES[square],
            'n' => board.bb_pieces[Sides::BLACK][Pieces::KNIGHT] |= BB_SQUARES[square],
            'p' => board.bb_pieces[Sides::BLACK][Pieces::PAWN] |= BB_SQUARES[square],
            'K' => board.bb_pieces[Sides::WHITE][Pieces::KING] |= BB_SQUARES[square],
            'Q' => board.bb_pieces[Sides::WHITE][Pieces::QUEEN] |= BB_SQUARES[square],
            'R' => board.bb_pieces[Sides::WHITE][Pieces::ROOK] |= BB_SQUARES[square],
            'B' => board.bb_pieces[Sides::WHITE][Pieces::BISHOP] |= BB_SQUARES[square],
            'N' => board.bb_pieces[Sides::WHITE][Pieces::KNIGHT] |= BB_SQUARES[square],
            'P' => board.bb_pieces[Sides::WHITE][Pieces::PAWN] |= BB_SQUARES[square],
            '1'..='8' => {
                if let Some(x) = c.to_digit(10) {
                    file += x as usize;
                }
            }
            SPLITTER => {
                result = file == 8;
                rank -= 1;
                file = 0;
            }
            _ => result = false,
        }

        if LIST_OF_PIECES.contains(c) {
            file += 1;
        }

        if !result {
            break;
        }
    }

    result
}

#[allow(dead_code)]
fn color(board: &mut Board, part: &str) -> bool {
    let mut result = false;

    if part.len() == 1 {
        if let Some(x) = part.chars().next() {
            if W_OR_B.contains(x) {
                match x {
                    'w' => board.gamestate.active_side = Sides::WHITE, // Set active color
                    'b' => board.gamestate.active_side = Sides::BLACK,
                    _ => (),
                }

                result = true;
            }
        }
    }

    result
}

#[allow(dead_code)]
fn castling(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut correct = 0;

    if (1..=4).contains(&length) {
        for c in part.chars() {
            match c {
                'K' => {
                    board.gamestate.castling |= Castling::WK;
                    correct += 1;
                }
                'Q' => {
                    board.gamestate.castling |= Castling::WQ;
                    correct += 1;
                }
                'k' => {
                    board.gamestate.castling |= Castling::BK;
                    correct += 1;
                }
                'q' => {
                    board.gamestate.castling |= Castling::BQ;
                    correct += 1;
                }
                '-' if length == 1 => {
                    // Only dash is valid for no castling
                    correct += 1;
                }
                _ => {
                    // Invalid character
                }
            }
        }
    }

    (length >= 1) && (correct == length)
}

#[allow(dead_code)]
fn enpassant(board: &mut Board, part: &str) -> bool {
    let length = part.len();

    let mut correct = 0;

    if length == 1 {
        if let Some(x) = part.chars().next() {
            if x == DASH {
                correct += 1;
            }
        }
    }

    if length == 2 {
        let square = algebraic_from_str(part);
        match square {
            Some(s) if EP_SQUARES_BLACK.contains(&s) || EP_SQUARES_WHITE.contains(&s) => {
                board.gamestate.enpassant = Some(s as u8);
                correct += 2;
            }
            Some(_) | None => (),
        }
    }

    (length == 1 || length == 2) && (length == correct)
}

#[allow(dead_code)]
fn halfmoveclock(board: &mut Board, part: &str) -> bool {
    let length = part.len();

    let mut result = false;

    if length == 1 || length == 2 {
        if let Ok(x) = part.parse::<u8>() {
            if x < MAX_MOVE_RULE {
                board.gamestate.halfclockmove = x;
                result = true;
            }
        }
    }

    result
}

#[allow(dead_code)]
fn fullmovenumber(board: &mut Board, part: &str) -> bool {
    let length = part.len();

    let mut result = false;
    if (1..=4).contains(&length) {
        if let Ok(x) = part.parse::<u16>() {
            if x < MAX_GAME_MOVES as u16 {
                board.gamestate.fullmovenumber = x;
                result = true;
            }
        } else {
            println!("Could not parse string to u16");
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::Pieces;
    use crate::defs::{Castling, FEN_KIWIPETE_POSITION, FEN_START_POSITION, Sides};

    // Test positions from chessprogrammingwiki and common test suites
    const FEN_EMPTY_BOARD: &str = "8/8/8/8/8/8/8/8 w - - 0 1";
    const FEN_EN_PASSANT_WHITE: &str =
        "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2";
    const FEN_EN_PASSANT_BLACK: &str =
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e3 0 1";
    const FEN_CASTLING_RIGHTS: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    const FEN_NO_CASTLING: &str = "r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1";
    const FEN_RUY_LOPEZ: &str = "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
    const FEN_PROMOTION_POSITION: &str = "8/PPPP4/8/8/8/8/pppp4/8 w - - 0 1";
    const FEN_MIDDLE_GAME: &str =
        "r2qkb1r/pp2pppp/2n2n2/3p4/2PP4/2N2N2/PP2PPPP/R2QKB1R w KQkq - 4 5";
    const FEN_ENDGAME: &str = "8/8/8/8/8/8/4K3/4k3 w - - 0 1";
    const FEN_COMPLEX: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";

    fn assert_piece_at_square(board: &Board, side: usize, piece: usize, square: usize) {
        let bit = 1u64 << square;
        assert!(
            (board.bb_pieces[side][piece] & bit) != 0,
            "Expected {:?} {:?} at square {}, but bitboard is {:064b}",
            if side == Sides::WHITE {
                "White"
            } else {
                "Black"
            },
            piece,
            square,
            board.bb_pieces[side][piece]
        );
    }

    fn assert_no_piece_at_square(board: &Board, side: usize, piece: usize, square: usize) {
        let bit = 1u64 << square;
        assert!(
            (board.bb_pieces[side][piece] & bit) == 0,
            "Expected no {:?} {:?} at square {}",
            if side == Sides::WHITE {
                "White"
            } else {
                "Black"
            },
            piece,
            square
        );
    }

    #[test]
    fn test_starting_position() {
        let board = Board::build(Some(FEN_START_POSITION));

        // Check white pieces
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 0); // a1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KNIGHT, 1); // b1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::BISHOP, 2); // c1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::QUEEN, 3); // d1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KING, 4); // e1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::BISHOP, 5); // f1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KNIGHT, 6); // g1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 7); // h1

        // Check white pawns
        for i in 8..16 {
            assert_piece_at_square(&board, Sides::WHITE, Pieces::PAWN, i);
        }

        // Check black pieces
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 56); // a8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KNIGHT, 57); // b8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::BISHOP, 58); // c8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::QUEEN, 59); // d8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 60); // e8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::BISHOP, 61); // f8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KNIGHT, 62); // g8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 63); // h8

        // Check black pawns
        for i in 48..56 {
            assert_piece_at_square(&board, Sides::BLACK, Pieces::PAWN, i);
        }

        // Check game state
        assert_eq!(board.gamestate.active_side, Sides::WHITE);
        assert_eq!(board.gamestate.castling, Castling::ALL);
        assert_eq!(board.gamestate.enpassant, Some(0)); // No en passant
        assert_eq!(board.gamestate.halfclockmove, 0);
        assert_eq!(board.gamestate.fullmovenumber, 1);
    }

    #[test]
    fn test_kiwipete_position() {
        let board = Board::build(Some(FEN_KIWIPETE_POSITION));

        // Kiwipete is a well-known test position
        // FEN: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R"
        // Verify some key pieces
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KING, 4); // e1
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 60); // e8
        assert_piece_at_square(&board, Sides::WHITE, Pieces::QUEEN, 21); // f3
        // Black queen is at e7 (square 52), not 53
        assert_piece_at_square(&board, Sides::BLACK, Pieces::QUEEN, 52); // e7

        // Check game state
        assert_eq!(board.gamestate.active_side, Sides::WHITE);
        assert_eq!(board.gamestate.castling, Castling::ALL);
    }

    #[test]
    fn test_empty_board() {
        let board = Board::build(Some(FEN_EMPTY_BOARD));

        // All bitboards should be empty
        for side in 0..Sides::BOTH {
            for piece in 0..NrOf::PIECE_TYPES {
                assert_eq!(board.bb_pieces[side][piece], 0);
            }
        }

        assert_eq!(board.gamestate.active_side, Sides::WHITE);
        assert_eq!(board.gamestate.castling, 0);
    }

    #[test]
    fn test_en_passant_white_target() {
        let board = Board::build(Some(FEN_EN_PASSANT_WHITE));

        // d3 is the en passant target square (square 19)
        assert_eq!(board.gamestate.enpassant, Some(19));
        assert_eq!(board.gamestate.active_side, Sides::BLACK);
    }

    #[test]
    fn test_en_passant_black_target() {
        let board = Board::build(Some(FEN_EN_PASSANT_BLACK));

        // e3 is the en passant target square (square 20)
        assert_eq!(board.gamestate.enpassant, Some(20));
        assert_eq!(board.gamestate.active_side, Sides::WHITE);
    }

    #[test]
    fn test_no_en_passant() {
        let board = Board::build(Some(FEN_START_POSITION));
        // When no en passant, should be Some(0) based on current implementation
        // This might need adjustment based on your design
        assert_eq!(board.gamestate.enpassant, Some(0));
    }

    #[test]
    fn test_castling_rights_all() {
        let board = Board::build(Some(FEN_CASTLING_RIGHTS));

        assert_eq!(board.gamestate.castling, Castling::ALL);
        assert_eq!(board.gamestate.active_side, Sides::WHITE);
    }

    #[test]
    fn test_no_castling_rights() {
        let board = Board::build(Some(FEN_NO_CASTLING));

        assert_eq!(board.gamestate.castling, 0);
    }

    #[test]
    fn test_ruy_lopez_position() {
        let board = Board::build(Some(FEN_RUY_LOPEZ));

        // Verify key pieces in Ruy Lopez position
        assert_piece_at_square(&board, Sides::WHITE, Pieces::BISHOP, 33); // b5
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KNIGHT, 21); // f3
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KNIGHT, 42); // c6
        assert_piece_at_square(&board, Sides::BLACK, Pieces::PAWN, 36); // e5

        assert_eq!(board.gamestate.active_side, Sides::WHITE);
        assert_eq!(board.gamestate.halfclockmove, 4);
        assert_eq!(board.gamestate.fullmovenumber, 4);
    }

    #[test]
    fn test_endgame_position() {
        let board = Board::build(Some(FEN_ENDGAME));

        // FEN: "8/8/8/8/8/8/4K3/4k3" means:
        // Rank 2 (squares 8-15): "4K3" = e2 has white king (square 12)
        // Rank 1 (squares 0-7): "4k3" = e1 has black king (square 4)
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KING, 12); // e2
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 4); // e1

        // Verify no other pieces
        for side in 0..Sides::BOTH {
            for piece in 0..NrOf::PIECE_TYPES {
                if piece == Pieces::KING {
                    continue;
                }
                assert_eq!(
                    board.bb_pieces[side][piece], 0,
                    "Unexpected piece {:?} for side {:?}",
                    piece, side
                );
            }
        }
    }

    #[test]
    fn test_complex_position() {
        let board = Board::build(Some(FEN_COMPLEX));

        // This is a complex position with many pieces
        // Just verify it parses correctly
        assert_eq!(board.gamestate.active_side, Sides::WHITE);
        assert_eq!(board.gamestate.castling, Castling::BK | Castling::BQ);
        assert_eq!(board.gamestate.halfclockmove, 0);
        assert_eq!(board.gamestate.fullmovenumber, 1);
    }

    #[test]
    fn test_default_fen() {
        // Test that None uses default starting position
        let board1 = Board::build(None);
        let board2 = Board::build(Some(FEN_START_POSITION));

        // Compare bitboards
        for side in 0..Sides::BOTH {
            for piece in 0..NrOf::PIECE_TYPES {
                assert_eq!(
                    board1.bb_pieces[side][piece], board2.bb_pieces[side][piece],
                    "Mismatch at side {} piece {}",
                    side, piece
                );
            }
        }
    }

    #[test]
    fn test_invalid_fen_too_few_parts() {
        let mut board = Board::new();
        let result = board.read_fen(Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq"));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fen_too_many_parts() {
        let mut board = Board::new();
        let result = board.read_fen(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra",
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fen_invalid_piece() {
        let mut board = Board::new();
        let result = board.read_fen(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1",
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fen_invalid_color() {
        let mut board = Board::new();
        let result = board.read_fen(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fen_invalid_castling() {
        let mut board = Board::new();
        let result = board.read_fen(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w XQkq - 0 1",
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_halfmove_clock() {
        let board = Board::build(Some("8/8/8/8/8/8/8/8 w - - 50 75"));
        assert_eq!(board.gamestate.halfclockmove, 50);
        assert_eq!(board.gamestate.fullmovenumber, 75);
    }

    #[test]
    fn test_black_to_move() {
        let board = Board::build(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        ));
        assert_eq!(board.gamestate.active_side, Sides::BLACK);
    }

    #[test]
    fn test_partial_castling_rights() {
        // Only white kingside
        let board = Board::build(Some("r3k2r/8/8/8/8/8/8/R3K2R w K - 0 1"));
        assert_eq!(board.gamestate.castling, Castling::WK);

        // Only white queenside
        let board = Board::build(Some("r3k2r/8/8/8/8/8/8/R3K2R w Q - 0 1"));
        assert_eq!(board.gamestate.castling, Castling::WQ);

        // Only black kingside
        let board = Board::build(Some("r3k2r/8/8/8/8/8/8/R3K2R w k - 0 1"));
        assert_eq!(board.gamestate.castling, Castling::BK);

        // Only black queenside
        let board = Board::build(Some("r3k2r/8/8/8/8/8/8/R3K2R w q - 0 1"));
        assert_eq!(board.gamestate.castling, Castling::BQ);
    }

    #[test]
    fn test_ranks_with_only_numbers() {
        // Test positions with many empty squares
        let board = Board::build(Some("8/8/8/8/8/8/8/8 w - - 0 1"));
        for side in 0..Sides::BOTH {
            for piece in 0..NrOf::PIECE_TYPES {
                assert_eq!(board.bb_pieces[side][piece], 0);
            }
        }
    }

    #[test]
    fn test_mixed_ranks() {
        // Test ranks with both pieces and numbers
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let board = Board::build(Some(fen));

        // Verify rooks and kings are in correct positions
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 0); // a1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KING, 4); // e1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 7); // h1
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 56); // a8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 60); // e8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 63); // h8
    }
}
