pub mod display;
pub mod perft;

pub use crate::board::types::{Pieces, SQUARE_NAME};
pub use crate::movegen::moves::Move;

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


