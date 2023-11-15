use crate::defs::Bitboard;
pub use crate::move_gen::CastlingType;
use colored::Colorize;
use std::{
    fmt::{self},
    str::FromStr,
};

use crate::zobrist::ZobristHasher;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn opposite(self) -> Self {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    King,
    Queen,
}

impl PieceKind {
    pub fn index(self) -> usize {
        match self {
            PieceKind::King => 0,
            PieceKind::Queen => 1,
            PieceKind::Rook => 2,
            PieceKind::Bishop => 3,
            PieceKind::Knight => 4,
            PieceKind::Pawn => 5,
        }
    }

    pub fn name(self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Rook => 'r',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::King => 'k',
            PieceKind::Queen => 'q',
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Piece {
    pub color: PieceColor,
    pub kind: PieceKind,
}

impl Piece {
    pub fn pawn(color: PieceColor) -> Piece {
        Self {
            color,
            kind: PieceKind::Pawn,
        }
    }

    pub fn rook(color: PieceColor) -> Piece {
        Self {
            color,
            kind: PieceKind::Rook,
        }
    }

    pub fn knight(color: PieceColor) -> Piece {
        Self {
            color,
            kind: PieceKind::Knight,
        }
    }

    pub fn bishop(color: PieceColor) -> Piece {
        Self {
            color,
            kind: PieceKind::Bishop,
        }
    }

    pub fn king(color: PieceColor) -> Piece {
        Self {
            color,
            kind: PieceKind::King,
        }
    }

    pub fn queen(color: PieceColor) -> Piece {
        Self {
            color,
            kind: PieceKind::Queen,
        }
    }

    pub fn index(self) -> usize {
        self.kind.index()
    }

    fn fenstring_char(self) -> char {
        match (self.color, self.kind) {
            (PieceColor::White, PieceKind::Pawn) => 'P',
            (PieceColor::White, PieceKind::Knight) => 'N',
            (PieceColor::White, PieceKind::Bishop) => 'B',
            (PieceColor::White, PieceKind::Rook) => 'R',
            (PieceColor::White, PieceKind::Queen) => 'Q',
            (PieceColor::White, PieceKind::King) => 'K',
            (PieceColor::Black, PieceKind::Pawn) => 'p',
            (PieceColor::Black, PieceKind::Knight) => 'n',
            (PieceColor::Black, PieceKind::Bishop) => 'b',
            (PieceColor::Black, PieceKind::Rook) => 'r',
            (PieceColor::Black, PieceKind::Queen) => 'q',
            (PieceColor::Black, PieceKind::King) => 'k',
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Square {
    // Represents empty square
    Empty,
    // Occupied quare with piece
    Full(Piece),
    // A non-board square; the board data structure contains squares not present on the
    // actual board in order to make move calculation easier, and all such squares have
    // this variant.
    Boundary,
}

impl Square {
    pub fn is_empty(self) -> bool {
        self == Square::Empty
    }

    pub fn is_color(self, color: PieceColor) -> bool {
        match self {
            Square::Full(Piece {
                color: square_color,
                ..
            }) => color == square_color,
            _ => false,
        }
    }

    pub fn fenstring_char(self) -> char {
        match self {
            Square::Full(piece) => piece.fenstring_char(),
            _ => '.',
        }
    }
}

impl From<Piece> for Square {
    // Generate a square with the given piece
    fn from(piece: Piece) -> Self {
        Square::Full(piece)
    }
}

pub const BOARD_START: usize = 2;
pub const BOARD_END: usize = 10;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Point(pub usize, pub usize);

// Implement how to convert string notition to point
impl FromStr for Point {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err("Error: Invalid length of point string");
        }

        // Get point from string
        let x = s.chars().next().unwrap();
        let y = s.chars().nth(1).unwrap();

        // Convert to position
        let column = match x {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Err("Invalid column"),
        };

        let row: usize = BOARD_END - (y.to_digit(10).unwrap() as usize);

        // Check if position in on the board
        if !(BOARD_START..BOARD_END).contains(&row) {
            return Err("Invalid row");
        }

        Ok(Point(row, column + BOARD_START))
    }
}

// Implement how to print points
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            match self.1 {
                2 => "a",
                3 => "b",
                4 => "c",
                5 => "d",
                6 => "e",
                7 => "f",
                8 => "g",
                9 => "h",
                _ => "h",
            },
            match self.0 {
                2 => "8",
                3 => "7",
                4 => "6",
                5 => "5",
                6 => "4",
                7 => "3",
                8 => "2",
                9 => "1",
                _ => "1",
            },
        )
    }
}

pub struct ChessBoard {
    pub board: [[Square; 12]; 12],
    pub bb_pieces: [[Bitboard; 6]; 2],
    pub bb_side: [Bitboard; 2],
    pub turn: PieceColor,
    pub pawn_double_move: Option<Point>,
    pub white_king_loc: Point,
    pub black_king_loc: Point,
    pub wksc: bool,
    pub wqsc: bool,
    pub bksc: bool,
    pub bqsc: bool,
    pub last_move: Option<(Point, Point)>,
    pub pawn_promotion: Option<Piece>,
    pub zobrist_key: u64,
}

impl ChessBoard {
    pub fn fenstring_board(fen: &str) -> Result<ChessBoard, &str> {
        let mut board = [[Square::Boundary; 12]; 12];
        let mut bb_pieces = [[0; 6]; 2];
        let mut bb_side = [0; 2];
        let fen = fen.to_string();
        let zobrist_hasher = ZobristHasher::create_zobrist_hash();
        let mut zobrist_key = 0;

        let fen_conf: Vec<&str> = fen.split(' ').collect();
        // validate fen string
        if fen_conf.len() != 6 {
            return Err("Could not parse FEN string: Invalid");
        }

        // based on fen string check which side to start
        let to_move = match fen_conf[1] {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => return Err("Next player to move not provided in FEN string"),
        };

        zobrist_key = zobrist_hasher.to_move_val();

        let castling = fen_conf[2];
        let en_passant = fen_conf[3];

        let halfmoves = fen_conf[4].parse::<u8>();
        if halfmoves.is_err() {
            return Err("Could not parse halfmoves. Invalid FEN string");
        }

        let fullmoves = fen_conf[5].parse::<u8>();
        if fullmoves.is_err() {
            return Err("Could not parse fullmoves. Invalid FEN string");
        }

        let fen_rows: Vec<&str> = fen_conf[0].split('/').collect();
        if fen_rows.len() != 8 {
            return Err("Could not parse FEN rows. Invalid FEN string");
        }

        let mut row: usize = BOARD_START;
        let mut col: usize = BOARD_START;
        let mut white_king_location = Point(0, 0);
        let mut black_king_location = Point(0, 0);

        for fen_row in fen_rows {
            for square in fen_row.chars() {
                if row >= BOARD_END || col >= BOARD_END {
                    return Err("Could not parse FEN string. Index out of bounds");
                }

                // Valid empty square in fen string. (
                if square.is_digit(10) {
                    let squares_to_skip = square.to_digit(10).unwrap() as usize;

                    // If len is bigger then board size panic
                    if squares_to_skip + col > BOARD_END {
                        return Err("Could not parse FEN string. Index out of bound");
                    }

                    for _ in 0..squares_to_skip {
                        board[row][col] = Square::Empty;
                        col += 1;
                    }
                } else {
                    board[row][col] = match Self::piece_from_fen_string_char(square) {
                        Some(piece) => Square::Full(piece),
                        None => return Err("Could not parse FEN string: Invalid character"),
                    };

                    if let Square::Full(Piece { kind, color }) = board[row][col] {
                        zobrist_key ^=
                            zobrist_hasher.piece_val(Piece { kind, color }, Point(row, col));

                        if kind == PieceKind::King {
                            match color {
                                PieceColor::White => white_king_location = Point(row, col),
                                PieceColor::Black => black_king_location = Point(row, col),
                            };
                        }
                    }

                    col += 1;
                }
            }

            // Reset for new row
            if col != BOARD_END {
                return Err("Could not parse fen string: Complete row was not specified");
            }
            row += 1;
            col = BOARD_START;
        }

        // en_passant moves

        let mut en_passant_pos: Option<Point> = None;

        if en_passant.len() != 2 {
            if en_passant != "-" {
                return Err("Could not parse FEN string: Invalid en_passant string");
            }
        } else {
            en_passant_pos = en_passant.parse().ok();

            if let Some(point) = &en_passant_pos {
                zobrist_key ^= zobrist_hasher.en_passant_val(point.1);
            }
        }

        let mut board = ChessBoard {
            board,
            turn: to_move,
            bb_side,
            bb_pieces,
            pawn_double_move: en_passant_pos,
            white_king_loc: white_king_location,
            black_king_loc: black_king_location,
            bksc: castling.find('K') != None,
            bqsc: castling.find('Q') != None,
            wksc: castling.find('k') != None,
            wqsc: castling.find('q') != None,
            last_move: None,
            pawn_promotion: None,
            zobrist_key,
        };

        // Update casteling in zobrist_hasher
        if board.wksc {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::WhiteKingSide);
        }
        if board.wqsc {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::WhiteQueenSide);
        }
        if board.bksc {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::BlackKingSide)
        }
        if board.bqsc {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::BlackQueenSide);
        }

        Ok(board)
    }

    fn piece_from_fen_string_char(piece: char) -> Option<Piece> {
        match piece {
            'r' => Some(Piece {
                color: PieceColor::Black,
                kind: PieceKind::Rook,
            }),
            'n' => Some(Piece {
                color: PieceColor::Black,
                kind: PieceKind::Knight,
            }),
            'b' => Some(Piece {
                color: PieceColor::Black,
                kind: PieceKind::Bishop,
            }),
            'q' => Some(Piece {
                color: PieceColor::Black,
                kind: PieceKind::Queen,
            }),
            'k' => Some(Piece {
                color: PieceColor::Black,
                kind: PieceKind::King,
            }),
            'p' => Some(Piece {
                color: PieceColor::Black,
                kind: PieceKind::Pawn,
            }),
            'R' => Some(Piece {
                color: PieceColor::White,
                kind: PieceKind::Rook,
            }),
            'N' => Some(Piece {
                color: PieceColor::White,
                kind: PieceKind::Knight,
            }),
            'B' => Some(Piece {
                color: PieceColor::White,
                kind: PieceKind::Bishop,
            }),
            'Q' => Some(Piece {
                color: PieceColor::White,
                kind: PieceKind::Queen,
            }),
            'K' => Some(Piece {
                color: PieceColor::White,
                kind: PieceKind::King,
            }),
            'P' => Some(Piece {
                color: PieceColor::White,
                kind: PieceKind::Pawn,
            }),
            _ => None,
        }
    }

    pub fn pretty_print_board(&self) {
        println!("a b c d e f g h");
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                let square = self.board[i][j];
                let cell = format!("{} ", square.fenstring_char());
                let cell = match square {
                    Square::Full(Piece {
                        color: PieceColor::White,
                        ..
                    }) => cell.white(),
                    Square::Full(Piece {
                        color: PieceColor::Black,
                        ..
                    }) => cell.black(),
                    _ => cell.white(),
                };

                let cell = if (i + j) % 2 != 0 {
                    cell.on_truecolor(158, 93, 30)
                } else {
                    cell.on_truecolor(205, 170, 125)
                };

                print!("{}", cell);
            }
            println!(" {}", 10 - i);
        }
    }

    pub fn swap_color(&mut self, zobrist_hasher: &ZobristHasher) {
        match self.turn {
            PieceColor::Black => self.turn = PieceColor::White,
            PieceColor::White => self.turn = PieceColor::Black,
        }

        self.zobrist_key ^= zobrist_hasher.to_move_val();
    }

    pub fn get_pieces(&self, side: PieceColor, piece: Option<PieceKind>) -> Vec<(Piece, &str)> {
        let mut pieces: Vec<(Piece, &str)> = Vec::new();
        // Match on optional param
        // println!("{:?}", self.board);
        match piece {
            Some(piece) => {
                for row in BOARD_START..BOARD_END {
                    for column in BOARD_START..BOARD_END {
                        if let Square::Full(square) = self.board[row][column] {
                            if square.color == side {
                                if square.kind == piece {
                                    if column % 2 == 0 {
                                        pieces.push((square, "white"));
                                    } else {
                                        pieces.push((square, "black"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => {
                for row in BOARD_START..BOARD_END {
                    for column in BOARD_START..BOARD_END {
                        if let Square::Full(square) = self.board[row][column] {
                            if square.color == side {
                                if column % 2 == 0 {
                                    pieces.push((square, "white"));
                                } else {
                                    pieces.push((square, "black"));
                                }
                            }
                        }
                    }
                }
            }
        }

        pieces
    }

    pub fn has_bishop_pair(&self, side: PieceColor) -> bool {
        let mut bishops = self.get_pieces(side, Some(PieceKind::Bishop));
        let mut white_square = 0;
        let mut black_square = 0;

        if bishops.iter().count() >= 2 {
            white_square = bishops.iter().filter(|a| a.1 == "white").count();
            black_square = bishops.iter().filter(|a| a.1 == "black").count();
        }

        white_square >= 1 && black_square >= 1
    }

    pub fn sufficient_material_to_force_checkmate(&self, side: PieceColor) -> bool {
        self.has_bishop_pair(side)
            || self.get_pieces(side, Some(PieceKind::Queen)).iter().count() > 0
            || self.get_pieces(side, Some(PieceKind::Rook)).iter().count() > 0
            || self.get_pieces(side, Some(PieceKind::Pawn)).iter().count() > 0
            || self
                .get_pieces(side, Some(PieceKind::Knight))
                .iter()
                .count()
                >= 3
            || self
                .get_pieces(side, Some(PieceKind::Bishop))
                .iter()
                .count()
                > 0
                && self.get_pieces(side, Some(PieceKind::King)).iter().count() > 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn recognize_pieces() {
        assert_eq!(Piece::bishop(PieceColor::White).color, PieceColor::White);
        assert_eq!(Piece::rook(PieceColor::White).color, PieceColor::White);
        assert_eq!(Piece::king(PieceColor::White).color, PieceColor::White);
        assert_eq!(Piece::pawn(PieceColor::White).color, PieceColor::White);
        assert_eq!(Piece::queen(PieceColor::White).color, PieceColor::White);
        assert_eq!(Piece::knight(PieceColor::White).color, PieceColor::White);

        assert_eq!(Piece::bishop(PieceColor::Black).color, PieceColor::Black);
        assert_eq!(Piece::rook(PieceColor::Black).color, PieceColor::Black);
        assert_eq!(Piece::king(PieceColor::Black).color, PieceColor::Black);
        assert_eq!(Piece::pawn(PieceColor::Black).color, PieceColor::Black);
        assert_eq!(Piece::queen(PieceColor::Black).color, PieceColor::Black);
        assert_eq!(Piece::knight(PieceColor::Black).color, PieceColor::Black);

        assert_eq!(Piece::bishop(PieceColor::Black).kind, PieceKind::Bishop);
        assert_eq!(Piece::rook(PieceColor::Black).kind, PieceKind::Rook);
        assert_eq!(Piece::king(PieceColor::Black).kind, PieceKind::King);
        assert_eq!(Piece::pawn(PieceColor::Black).kind, PieceKind::Pawn);
        assert_eq!(Piece::queen(PieceColor::Black).kind, PieceKind::Queen);
        assert_eq!(Piece::knight(PieceColor::Black).kind, PieceKind::Knight);

        assert!(Square::Empty.is_empty());
        assert!(!Square::Full(Piece::king(PieceColor::White)).is_empty());
    }

    // Fen string tests
    #[test]
    fn empty_board() {
        let b = ChessBoard::fenstring_board("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], Square::Empty);
            }
        }
        // println!("{}", b.zobrist_key);
        // ChessBoard::pretty_print_board(&b);
        assert_eq!(b.zobrist_key, 7723405160658224286);
    }

    #[test]
    fn starting_pos() {
        let b =
            ChessBoard::fenstring_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        assert_eq!(b.board[2][2], Square::from(Piece::rook(PieceColor::Black)));
        assert_eq!(
            b.board[2][3],
            Square::from(Piece::knight(PieceColor::Black))
        );
        assert_eq!(
            b.board[2][4],
            Square::from(Piece::bishop(PieceColor::Black))
        );
        assert_eq!(b.board[2][5], Square::from(Piece::queen(PieceColor::Black)));
        assert_eq!(b.board[2][6], Square::from(Piece::king(PieceColor::Black)));
        assert_eq!(
            b.board[2][7],
            Square::from(Piece::bishop(PieceColor::Black))
        );
        assert_eq!(
            b.board[2][8],
            Square::from(Piece::knight(PieceColor::Black))
        );
        assert_eq!(b.board[2][9], Square::from(Piece::rook(PieceColor::Black)));

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[3][i], Square::from(Piece::pawn(PieceColor::Black)));
        }

        for i in 4..8 {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], Square::Empty);
            }
        }

        assert_eq!(b.board[9][2], Square::from(Piece::rook(PieceColor::White)));
        assert_eq!(
            b.board[9][3],
            Square::from(Piece::knight(PieceColor::White))
        );
        assert_eq!(
            b.board[9][4],
            Square::from(Piece::bishop(PieceColor::White))
        );
        assert_eq!(b.board[9][5], Square::from(Piece::queen(PieceColor::White)));
        assert_eq!(b.board[9][6], Square::from(Piece::king(PieceColor::White)));
        assert_eq!(
            b.board[9][7],
            Square::from(Piece::bishop(PieceColor::White))
        );
        assert_eq!(
            b.board[9][8],
            Square::from(Piece::knight(PieceColor::White))
        );
        assert_eq!(b.board[9][9], Square::from(Piece::rook(PieceColor::White)));

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[8][i], Square::from(Piece::pawn(PieceColor::White)));
        }
        ChessBoard::pretty_print_board(&b);
        assert_eq!(b.zobrist_key, 8112390767216820594);
    }
}
