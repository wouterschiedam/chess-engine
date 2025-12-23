use std::ops::RangeInclusive;

use crate::defs::{Bitboard, Castling, NrOf, Piece, Side, Sides, Square, EMPTY, FEN_START_POSITION, MAX_GAME_MOVES, MAX_MOVE_RULE};
use super::types::{Files, Pieces, Ranks, Squares, BB_SQUARES, SQUARE_NAME};

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
    use crate::defs::{Castling, FEN_START_POSITION, FEN_KIWIPETE_POSITION, Sides};
    use crate::board::types::Pieces;

    // Test positions from chessprogrammingwiki and common test suites
    const FEN_EMPTY_BOARD: &str = "8/8/8/8/8/8/8/8 w - - 0 1";
    const FEN_EN_PASSANT_WHITE: &str = "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2";
    const FEN_EN_PASSANT_BLACK: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e3 0 1";
    const FEN_CASTLING_RIGHTS: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    const FEN_NO_CASTLING: &str = "r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1";
    const FEN_RUY_LOPEZ: &str = "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
    const FEN_PROMOTION_POSITION: &str = "8/PPPP4/8/8/8/8/pppp4/8 w - - 0 1";
    const FEN_MIDDLE_GAME: &str = "r2qkb1r/pp2pppp/2n2n2/3p4/2PP4/2N2N2/PP2PPPP/R2QKB1R w KQkq - 4 5";
    const FEN_ENDGAME: &str = "8/8/8/8/8/8/4K3/4k3 w - - 0 1";
    const FEN_COMPLEX: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";

    fn assert_piece_at_square(board: &Board, side: usize, piece: usize, square: usize) {
        let bit = 1u64 << square;
        assert!(
            (board.bb_pieces[side][piece] & bit) != 0,
            "Expected {:?} {:?} at square {}, but bitboard is {:064b}",
            if side == Sides::WHITE { "White" } else { "Black" },
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
            if side == Sides::WHITE { "White" } else { "Black" },
            piece,
            square
        );
    }

    #[test]
    fn test_starting_position() {
        let board = Board::build(Some(FEN_START_POSITION));
        
        // Check white pieces
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 0);   // a1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KNIGHT, 1); // b1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::BISHOP, 2); // c1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::QUEEN, 3);  // d1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KING, 4);    // e1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::BISHOP, 5);  // f1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KNIGHT, 6); // g1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 7);    // h1
        
        // Check white pawns
        for i in 8..16 {
            assert_piece_at_square(&board, Sides::WHITE, Pieces::PAWN, i);
        }
        
        // Check black pieces
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 56);   // a8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KNIGHT, 57); // b8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::BISHOP, 58); // c8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::QUEEN, 59);  // d8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 60);   // e8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::BISHOP, 61); // f8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KNIGHT, 62); // g8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 63);   // h8
        
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
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KING, 4);    // e1
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 60);   // e8
        assert_piece_at_square(&board, Sides::WHITE, Pieces::QUEEN, 21);  // f3
        // Black queen is at e7 (square 52), not 53
        assert_piece_at_square(&board, Sides::BLACK, Pieces::QUEEN, 52);  // e7
        
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
        assert_piece_at_square(&board, Sides::BLACK, Pieces::PAWN, 36);  // e5
        
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
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 4);  // e1
        
        // Verify no other pieces
        for side in 0..Sides::BOTH {
            for piece in 0..NrOf::PIECE_TYPES {
                if piece == Pieces::KING {
                    continue;
                }
                assert_eq!(board.bb_pieces[side][piece], 0, 
                    "Unexpected piece {:?} for side {:?}", piece, side);
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
                    board1.bb_pieces[side][piece],
                    board2.bb_pieces[side][piece],
                    "Mismatch at side {} piece {}",
                    side,
                    piece
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
        let result = board.read_fen(Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra"));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fen_invalid_piece() {
        let mut board = Board::new();
        let result = board.read_fen(Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fen_invalid_color() {
        let mut board = Board::new();
        let result = board.read_fen(Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fen_invalid_castling() {
        let mut board = Board::new();
        let result = board.read_fen(Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w XQkq - 0 1"));
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
        let board = Board::build(Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1"));
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
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 0);   // a1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::KING, 4);   // e1
        assert_piece_at_square(&board, Sides::WHITE, Pieces::ROOK, 7);   // h1
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 56);  // a8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::KING, 60);   // e8
        assert_piece_at_square(&board, Sides::BLACK, Pieces::ROOK, 63);  // h8
    }
}

