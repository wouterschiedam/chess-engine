use crate::{
    board::{
        defs::{Pieces, SQUARE_NAME},
        Board,
    },
    defs::{Bitboard, NrOf, Sides},
    evaluation::material::count,
    movegen::defs::castling_as_string,
};
type AsciiBoard = [char; NrOf::SQUARES];

const CHAR_ES: char = ' ';
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
            print!(" | {}\n", rank + 1);
        } else {
            print!(" | {}", rank + 1);
        }
    }
    print!(" +---+---+---+---+---+---+---+---+\n");
    print!("   a   b   c   d   e   f   g   h\n\n");

    game_metadata(board);
}

fn board_to_ascii(board: &Board, ascii_board: &mut AsciiBoard) {
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

fn put_character_on_square(bitboard: Bitboard, ascii_board: &mut AsciiBoard, character: char) {
    for (i, square) in ascii_board.iter_mut().enumerate() {
        if (bitboard >> i) & 1 == 1 {
            *square = character;
        }
    }
}

fn game_metadata(board: &Board) {
    let is_white = (board.gamestate.active_color as usize) == Sides::WHITE;
    let turn = if is_white { "White" } else { "Black" };
    let castling = castling_as_string(board.gamestate.castling);

    let en_passant = match board.gamestate.en_passant {
        Some(ep) => SQUARE_NAME[ep as usize],
        _ => "-",
    };

    let half_moveclock = board.gamestate.halfclock_move;
    let full_movenumber = board.gamestate.fullmove_number;

    let eval = count(&board);
    println!("{}{}", "Fen: ", board.create_fen());
    println!("{:<20}{}", "White eval:", eval.0);
    println!("{:<20}{}", "Black eval:", eval.1);

    println!("{:<20}{:x}", "Zobrist key:", board.gamestate.zobrist_key);
    println!("{:<20}{}", "Active Color:", turn);
    println!("{:<20}{}", "Castling:", castling);
    println!("{:<20}{}", "En Passant:", en_passant);
    println!("{:<20}{}", "Half-move clock:", half_moveclock);
    println!("{:<20}{}", "Full-move number:", full_movenumber);
    println!();
}
