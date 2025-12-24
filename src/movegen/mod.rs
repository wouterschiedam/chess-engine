// Move generation module
// This module contains move generation logic and attack tables

pub mod movegen;
pub mod attacks;
pub mod magic;
pub mod moves;

// Re-export commonly used functions
pub use movegen::{
    generate_bishop_moves, generate_king_moves, generate_knight_moves, generate_legal_moves,
    generate_pawn_moves, generate_psuedo_legal_moves, generate_queen_moves, generate_rook_moves,
};
pub use moves::{Move, MoveFlags, MoveInfo};

