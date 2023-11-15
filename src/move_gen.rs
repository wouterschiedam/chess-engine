pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
use crate::zobrist::ZobristHasher;

const KNIGHT_MOVES: [(i8, i8); 8] = [
    (1, 2),
    (1, -2),
    (2, 1),
    (2, -1),
    (-1, 2),
    (-1, -2),
    (-2, 1),
    (-2, -1),
];

const MVV_LVA: [[u8; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 0],       // victim K, attacker K, Q, R, B, N, P, None
    [50, 51, 52, 53, 54, 55, 0], // victim Q, attacker K, Q, R, B, N, P, None
    [40, 41, 42, 43, 44, 45, 0], // victim R, attacker K, Q, R, B, N, P, None
    [30, 31, 32, 33, 34, 35, 0], // victim B, attacker K, Q, R, B, N, P, None
    [20, 21, 22, 23, 24, 25, 0], // victim N, attacker K, Q, R, B, N, P, None
    [10, 11, 12, 13, 14, 15, 0], // victim P, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim None, attacker K, Q, R, B, N, P, None
];

pub enum CastlingType {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
}

#[derive(PartialEq)]
pub enum LegalMoves {
    AllMoves,
    CaptureMoves,
}

pub fn generate_move(board: &ChessBoard, legal_moves: LegalMoves, zobrish_hasher: &ZobristHasher) {}

fn knight_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &ChessBoard,
    moves: &mut Vec<Point>,
    legal_moves: LegalMoves,
) {
    for (x, y) in &KNIGHT_MOVES {
        let row = (row as i8 + x) as usize;
        let col = (col as i8 + y) as usize;
        let square = board.board[row][col];

        if square.is_color(piece.color.opposite()) {
            if legal_moves == LegalMoves::CaptureMoves {
                if !square.is_empty() {
                    moves.push(Point(row, col));
                }
            } else {
                moves.push(Point(row, col));
            }
        }
    }
}

fn pawn_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &ChessBoard,
    moves: &mut Vec<Point>,
    legal_moves: LegalMoves,
) {
    match piece.color {
        // White moves up the board
        White => {
            // Check for CaptureMoves
            let left_capture = board.board[row - 1][col - 1];
            let right_capture = board.board[row - 1][col + 1];

            if let Square::Full(Piece { color: Black, .. }) = left_capture {
                moves.push(Point(row - 1, col - 1));
            }
            if let Square::Full(Piece { color: White, .. }) = right_capture {
                moves.push(Point(row - 1, col + 1));
            }

            // Check for normal push
            if legal_moves == LegalMoves::AllMoves && board.board[row - 1][col].is_empty() {
                moves.push(Point(row - 1, col));

                if row == 8 && board.board[row - 2][col].is_empty() {
                    moves.push(Point(row - 2, col));
                }
            }
        }
        // Black moves down the board
        Black => {
            // Check capture moves
            let left_capture = board.board[row + 1][col + 1];
            let right_capture = board.board[row + 1][col - 1];

            if let Square::Full(Piece { color: Black, .. }) = left_capture {
                moves.push(Point(row + 1, col + 1));
            }
            if let Square::Full(Piece { color: White, .. }) = right_capture {
                moves.push(Point(row + 1, col - 1));
            }

            // Check for normal push
            if legal_moves == LegalMoves::AllMoves && board.board[row + 1][col].is_empty() {
                moves.push(Point(row + 1, col));

                if row == 3 && board.board[row + 2][col].is_empty() {
                    moves.push(Point(row + 2, col));
                }
            }
        }
    }
}

fn pawn_moves_en_passant(
    piece: Piece,
    row: usize,
    col: usize,
    board: &ChessBoard,
) -> Option<Point> {
    if let Some(double_moved_pawn) = board.pawn_double_move {
        let left_cap: Point;
        let right_cap: Point;

        match piece.color {
            Black if row == BOARD_START + 3 => {
                left_cap = Point(row - 1, col - 1);
                right_cap = Point(row - 1, col + 1);
            }
            White if row == BOARD_START + 4 => {
                left_cap = Point(row + 1, col + 1);
                right_cap = Point(row + 1, col - 1);
            }
            _ => return None,
        }

        if left_cap == double_moved_pawn {
            return Some(left_cap);
        } else if right_cap == double_moved_pawn {
            return Some(right_cap);
        }
    }

    None
}

fn king_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &ChessBoard,
    moves: &mut Vec<Point>,
    legal_moves: LegalMoves,
) {
    for i in 0..3 {
        let row = row + i - 1;
        for j in 0..3 {
            let col = col + j - 1;

            let square = board.board[row][col];

            if square.is_color(piece.color.opposite()) {
                if legal_moves == LegalMoves::CaptureMoves {
                    if !square.is_empty() {
                        moves.push(Point(row, col));
                    }
                } else {
                    moves.push(Point(row, col));
                }
            }
        }
    }
}

fn rook_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &ChessBoard,
    moves: &mut Vec<Point>,
    legal_moves: LegalMoves,
) {
}
