use crate::defs::{Side, Sides};

use super::{PieceArt, PieceSize};

pub struct Bishop;

impl PieceArt for Bishop {
    fn get_art(size: PieceSize, side: Side) -> String {
        match size {
            PieceSize::Small => match side {
                Sides::WHITE => "♗".to_string(),
                _ => "♝".to_string(),
            },
            PieceSize::Compact => {
                // Simple 2-line design for medium-sized cells
                "\n ⭘\n ███".to_string()
            }
            PieceSize::Extended => {
                // Extended 3-4 line design - more solid and consistent
                " ⭘\n █▄█\n ███\n ███".to_string()
            }
            PieceSize::Large => "\
    \n\
       ⭘\n\
      █✝█\n\
      ███\n\
    ▗█████▖\n\
    "
            .to_string(),
        }
    }
}
