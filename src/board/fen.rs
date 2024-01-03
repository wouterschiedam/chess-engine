use super::{
    defs::{Files, Pieces, Ranks, Squares, BB_SQUARES},
    Board,
};
use crate::{
    defs::{Castling, Sides, Square, FEN_START_POSITION, MAX_GAME_MOVES, MAX_MOVE_RULE},
    movegen::defs::{algebraic_from_str, print_bitboard},
};
use std::ops::RangeInclusive;

// Define some fen things
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const W_OR_B: &str = "wb";
const NR_OF_FEN_PARTS: usize = 6;
const EP_SQUARES_WHITE: RangeInclusive<Square> = Squares::A3..=Squares::H3;
const EP_SQUARES_BLACK: RangeInclusive<Square> = Squares::A6..=Squares::H6;
const SPLITTER: char = '/';
const DASH: char = '-';
const EM_DASH: char = '–';
const SPACE: char = ' ';

type FenResult = Result<(), u8>;
type FenParser = fn(board: &mut Board, part: &str) -> bool;

impl Board {
    pub fn read_fen(&mut self, fen_string: Option<&str>) -> FenResult {
        // Split the string into parts. There should be 6 parts.
        let mut fen_parts: Vec<String> = match fen_string {
            Some(f) => f,
            None => FEN_START_POSITION,
        }
        .replace(EM_DASH, DASH.encode_utf8(&mut [0; 4]))
        .split(SPACE)
        .map(|s| s.to_string())
        .collect();

        // Check the number of fen parts.
        let nr_of_parts_ok = fen_parts.len() == NR_OF_FEN_PARTS;

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
            while i < NR_OF_FEN_PARTS && result == Ok(()) {
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

fn color(board: &mut Board, part: &str) -> bool {
    let mut result = false;

    if part.len() == 1 {
        if let Some(x) = part.chars().next() {
            if W_OR_B.contains(x) {
                match x {
                    'w' => board.gamestate.active_color = Sides::WHITE as u8, // Set active color
                    'b' => board.gamestate.active_color = Sides::BLACK as u8,
                    _ => (),
                }

                result = true;
            }
        }
    }

    result
}

fn castling(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut correct = 0;

    if (1..=4).contains(&length) {
        for c in part.chars() {
            match c {
                'K' => board.gamestate.castling |= Castling::WK,
                'Q' => board.gamestate.castling |= Castling::WQ,
                'k' => board.gamestate.castling |= Castling::BK,
                'q' => board.gamestate.castling |= Castling::BQ,
                _ => (),
            }
            correct += 1;
        }
    }

    (length >= 1) && (correct == length)
}

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
                board.gamestate.en_passant = Some(s as u8);
                correct += 2;
            }
            Some(_) | None => (),
        }
    }

    (length == 1 || length == 2) && (length == correct)
}

fn halfmoveclock(board: &mut Board, part: &str) -> bool {
    let length = part.len();

    let mut result = false;

    if length == 1 || length == 2 {
        if let Ok(x) = part.parse::<u8>() {
            if x < MAX_MOVE_RULE {
                board.gamestate.halfclock_move = x;
                result = true;
            }
        }
    }

    result
}

fn fullmovenumber(board: &mut Board, part: &str) -> bool {
    let length = part.len();

    let mut result = false;
    if length >= 1 || length <= 4 {
        if let Ok(x) = part.parse::<u16>() {
            if x < MAX_GAME_MOVES as u16 {
                board.gamestate.fullmove_number = x;
                result = true;
            }
        } else {
            println!("Could not parse string to u16");
        }
    }

    result
}
