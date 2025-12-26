use super::{PieceArt, PieceSize};
use crate::defs::{Side, Sides};

pub struct Knight;

impl PieceArt for Knight {
    fn get_art(size: PieceSize, side: Side) -> String {
        match size {
            PieceSize::Small => match side {
                Sides::WHITE => "♘".to_string(),
                _ => "♞".to_string(),
            },
            PieceSize::Compact => "\n ▟█\n ██".to_string(),
            PieceSize::Extended => "   \n ▟█ \n █▌ \n ██".to_string(),
            PieceSize::Large => "\
    \n\
    ▟▛██▙\n\
   ▟█████\n\
   ▀▀▟██▌\n\
    ▟████\n\
    "
            .to_string(),
        }
    }
}
