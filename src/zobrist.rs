use crate::board::{Piece, PieceColor::*, Point};
pub use crate::move_gen::CastlingType;
use rand_chacha::rand_core::{RngCore, SeedableRng};
const BOARD_SIZE: usize = 12;

// 6 pieces per color
const PIECE_TYPES: usize = 12;

pub type ZobristKey = u64;

pub struct ZobristHasher {
    // Indexed by [piece][file][rank]
    piece_square_table: [[[ZobristKey; BOARD_SIZE]; BOARD_SIZE]; PIECE_TYPES],
    to_move: ZobristKey,
    black_queen_side_castle: ZobristKey,
    black_king_side_castle: ZobristKey,
    white_queen_side_castle: ZobristKey,
    white_king_side_castle: ZobristKey,
    // Indexed by file
    en_passant_files: [ZobristKey; BOARD_SIZE],
}

impl ZobristHasher {
    pub fn create_zobrist_hash() -> ZobristHasher {
        // Here we use a seed so if you have to recreate the hasher you will always get the same values
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(27 * 9 * 1998);

        let mut piece_square_table = [[[0; BOARD_SIZE]; BOARD_SIZE]; PIECE_TYPES];

        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                for piece in 0..PIECE_TYPES {
                    piece_square_table[piece][file][rank] = rng.next_u64();
                }
            }
        }

        let mut en_passant_files = [0; BOARD_SIZE];

        for file in 0..BOARD_SIZE {
            en_passant_files[file] = rng.next_u64();
        }

        ZobristHasher {
            piece_square_table,
            to_move: rng.next_u64(),
            black_queen_side_castle: rng.next_u64(),
            black_king_side_castle: rng.next_u64(),
            white_queen_side_castle: rng.next_u64(),
            white_king_side_castle: rng.next_u64(),
            en_passant_files,
        }
    }

    pub fn piece_val(&self, piece: Piece, point: Point) -> ZobristKey {
        let index = piece.index() + if piece.color == White { 0 } else { 6 };

        self.piece_square_table[index][point.1][point.0]
    }

    pub fn castling_val() {}

    pub fn to_move_val(&self) -> ZobristKey {
        self.to_move
    }

    pub fn en_passant_val(&self, file: usize) -> ZobristKey {
        self.en_passant_files[file]
    }
    pub fn get_val_for_castling(&self, castling_type: CastlingType) -> ZobristKey {
        match castling_type {
            CastlingType::WhiteKingSide => self.white_king_side_castle,
            CastlingType::WhiteQueenSide => self.white_queen_side_castle,
            CastlingType::BlackKingSide => self.black_king_side_castle,
            CastlingType::BlackQueenSide => self.black_queen_side_castle,
        }
    }
}
