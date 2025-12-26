use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
};

use crate::{
    board::{
        Board,
        types::{Pieces, SQUARE_NAME},
    },
    defs::{Bitboard, NrOf, Sides},
    movegen::Move,
};

pub type AsciiBoard = [char; NrOf::SQUARES];

pub const CHAR_ES: char = ' ';
const CHAR_WK: char = 'K';
const CHAR_WQ: char = 'Q';
const CHAR_WR: char = 'R';
const CHAR_WB: char = 'B';
const CHAR_WN: char = 'N';
const CHAR_WP: char = 'P';
const CHAR_BK: char = 'k';
const CHAR_BQ: char = 'q';
const CHAR_BR: char = 'r';
const CHAR_BB: char = 'b';
const CHAR_BN: char = 'n';
const CHAR_BP: char = 'p';

pub fn print_position(board: &Board, highlight_bitmask: bool, bitmask: Option<Bitboard>) {
    let mut ascii_board: AsciiBoard = [CHAR_ES; NrOf::SQUARES];

    if !highlight_bitmask {
        board_to_ascii(board, &mut ascii_board);
    }

    // Print the board with grid lines
    for rank in (0..8).rev() {
        print!("\n +---+---+---+---+---+---+---+---+\n");
        for file in 0..8 {
            let index = rank * 8 + file;
            let piece_char = if highlight_bitmask {
                if (bitmask.unwrap() & (1 << index)) != 0 {
                    '*'
                } else {
                    ' '
                }
            } else {
                ascii_board[index]
            };
            print!(" | {}", piece_char);
        }
        if rank == 0 {
            println!(" | {}", rank + 1);
        } else {
            print!(" | {}", rank + 1);
        }
    }
    println!(" +---+---+---+---+---+---+---+---+");
    print!("   a   b   c   d   e   f   g   h\n\n");
}

pub fn board_to_string(board: &Board) -> String {
    let mut ascii_board: AsciiBoard = [CHAR_ES; NrOf::SQUARES];
    board_to_ascii(board, &mut ascii_board);

    let mut result = String::new();
    for rank in (0..8).rev() {
        result.push_str("\n +---+---+---+---+---+---+---+---+\n");
        for file in 0..8 {
            let index = rank * 8 + file;
            result.push_str(&format!(" | {}", ascii_board[index]));
        }
        result.push_str(&format!(" | {}\n", rank + 1));
    }
    result.push_str(" +---+---+---+---+---+---+---+---+\n");
    result.push_str("   a   b   c   d   e   f   g   h");
    result
}

pub fn board_to_ascii(board: &Board, ascii_board: &mut AsciiBoard) {
    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        match piece {
            Pieces::KING => {
                put_character_on_square(*w, ascii_board, CHAR_WK);
                put_character_on_square(*b, ascii_board, CHAR_BK);
            }
            Pieces::QUEEN => {
                put_character_on_square(*w, ascii_board, CHAR_WQ);
                put_character_on_square(*b, ascii_board, CHAR_BQ);
            }
            Pieces::ROOK => {
                put_character_on_square(*w, ascii_board, CHAR_WR);
                put_character_on_square(*b, ascii_board, CHAR_BR);
            }
            Pieces::BISHOP => {
                put_character_on_square(*w, ascii_board, CHAR_WB);
                put_character_on_square(*b, ascii_board, CHAR_BB);
            }
            Pieces::KNIGHT => {
                put_character_on_square(*w, ascii_board, CHAR_WN);
                put_character_on_square(*b, ascii_board, CHAR_BN);
            }
            Pieces::PAWN => {
                put_character_on_square(*w, ascii_board, CHAR_WP);
                put_character_on_square(*b, ascii_board, CHAR_BP);
            }
            _ => (),
        }
    }
}

pub fn board_to_unicode(board: &Board, unicode_board: &mut AsciiBoard) {
    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    const CHAR_WK_U: char = '♔';
    const CHAR_WQ_U: char = '♕';
    const CHAR_WR_U: char = '♖';
    const CHAR_WB_U: char = '♗';
    const CHAR_WN_U: char = '♘';
    const CHAR_WP_U: char = '♙';
    const CHAR_BK_U: char = '♚';
    const CHAR_BQ_U: char = '♛';
    const CHAR_BR_U: char = '♜';
    const CHAR_BB_U: char = '♝';
    const CHAR_BN_U: char = '♞';
    const CHAR_BP_U: char = '♟';

    for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        match piece {
            Pieces::KING => {
                put_character_on_square(*w, unicode_board, CHAR_WK_U);
                put_character_on_square(*b, unicode_board, CHAR_BK_U);
            }
            Pieces::QUEEN => {
                put_character_on_square(*w, unicode_board, CHAR_WQ_U);
                put_character_on_square(*b, unicode_board, CHAR_BQ_U);
            }
            Pieces::ROOK => {
                put_character_on_square(*w, unicode_board, CHAR_WR_U);
                put_character_on_square(*b, unicode_board, CHAR_BR_U);
            }
            Pieces::BISHOP => {
                put_character_on_square(*w, unicode_board, CHAR_WB_U);
                put_character_on_square(*b, unicode_board, CHAR_BB_U);
            }
            Pieces::KNIGHT => {
                put_character_on_square(*w, unicode_board, CHAR_WN_U);
                put_character_on_square(*b, unicode_board, CHAR_BN_U);
            }
            Pieces::PAWN => {
                put_character_on_square(*w, unicode_board, CHAR_WP_U);
                put_character_on_square(*b, unicode_board, CHAR_BP_U);
            }
            _ => (),
        }
    }
}

pub fn put_character_on_square(bitboard: Bitboard, ascii_board: &mut AsciiBoard, character: char) {
    for (i, square) in ascii_board.iter_mut().enumerate() {
        if (bitboard >> i) & 1 == 1 {
            *square = character;
        }
    }
}

pub fn format_move(mv: &Move) -> String {
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
