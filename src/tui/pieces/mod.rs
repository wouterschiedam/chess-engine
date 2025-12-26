pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

pub use bishop::Bishop;
pub use king::King;
pub use knight::Knight;
pub use pawn::Pawn;
pub use queen::Queen;
pub use rook::Rook;

use crate::defs::Side;

#[derive(Clone, Copy, PartialEq)]
pub enum PieceSize {
    Small,
    Compact,
    Extended,
    Large,
}

pub trait PieceArt {
    fn get_art(size: PieceSize, side: Side) -> String;
}
