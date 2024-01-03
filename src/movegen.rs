mod create;
pub mod defs;
mod init;
mod magics;
mod movelist;

use crate::{
    board::{
        defs::{Pieces, Squares, BB_RANKS, BB_SQUARES, SQUARE_NAME},
        Board,
    },
    defs::{Bitboard, Castling, NrOf, Piece, Side, Sides, Square, EMPTY},
    extra::bits,
};

use crate::movegen::magics::Magic;

use self::defs::{print_bitboard, Move, MoveList, MoveType, Shift};

const PROMOTION_PIECES: [usize; 4] = [Pieces::QUEEN, Pieces::ROOK, Pieces::KNIGHT, Pieces::BISHOP];

pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.
pub struct MoveGenerator {
    pub king: [Bitboard; NrOf::SQUARES],
    pub knight: [Bitboard; NrOf::SQUARES],
    pub pawns: [[Bitboard; NrOf::SQUARES]; Sides::BOTH],
    pub rook: Vec<Bitboard>,
    pub bishop: Vec<Bitboard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
}

impl MoveGenerator {
    pub fn new() -> Self {
        let magics: Magic = Default::default();

        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NrOf::SQUARES],
            bishop_magics: [magics; NrOf::SQUARES],
        };

        mg.init_king_attack();
        mg.init_knight_attack();
        mg.init_pawn_attack();
        mg.init_magics(Pieces::ROOK);
        mg.init_magics(Pieces::BISHOP);
        mg
    }

    // Generates moves for the side that is to move. The MoveType parameter
    // determines if all moves, or only captures need to be generated.
    pub fn generate_moves(&self, board: &Board, move_list: &mut MoveList, move_type: MoveType) {
        self.piece(board, Pieces::KING, move_list, move_type);
        self.piece(board, Pieces::KNIGHT, move_list, move_type);
        self.piece(board, Pieces::ROOK, move_list, move_type);
        self.piece(board, Pieces::BISHOP, move_list, move_type);
        self.piece(board, Pieces::QUEEN, move_list, move_type);
        self.pawn(board, move_list, move_type);
        if move_type == MoveType::All || move_type == MoveType::Quiet {
            self.castling(board, move_list);
        }
    }

    pub fn get_non_slider_moves(&self, piece: Piece, square: Square) -> Bitboard {
        match piece {
            Pieces::KING => self.king[square],
            Pieces::KNIGHT => self.knight[square],
            _ => panic!("Not a king or knight {piece}"),
        }
    }

    pub fn get_slider_moves(&self, piece: Piece, square: Square, occupancy: Bitboard) -> Bitboard {
        match piece {
            Pieces::ROOK => {
                let index = self.rook_magics[square].get_index(occupancy);
                self.rook[index]
            }
            Pieces::BISHOP => {
                let index = self.bishop_magics[square].get_index(occupancy);
                self.bishop[index]
            }
            Pieces::QUEEN => {
                let r_index = self.rook_magics[square].get_index(occupancy);
                let b_index = self.bishop_magics[square].get_index(occupancy);
                self.rook[r_index] ^ self.bishop[b_index]
            }
            _ => panic!("Not a ROOK or BISHOP but {piece}"),
        }
    }

    pub fn get_pawn_attacks(&self, side: Side, square: Square) -> Bitboard {
        self.pawns[side][square]
    }
}
// GET ACTUAL PSEUDO LEGAL MOVES!!!!
impl MoveGenerator {
    pub fn piece(
        &self,
        board: &Board,
        piece: Piece,
        move_list: &mut MoveList,
        move_type: MoveType,
    ) {
        let player = board.side_to_move();
        let bb_occupancy = board.occupancy();
        // find all square that are empty occupied by our pieces and opponent pieces
        let bb_empty = !bb_occupancy;
        let bb_own_pieces = board.bb_side[player];
        let bb_opponent_pieces = board.bb_side[board.side_to_not_move()];
        let mut bb_pieces = board.get_pieces(piece, player);
        // Generate all moves for each piece
        while bb_pieces > 0 {
            let from = bits::next(&mut bb_pieces);
            let bb_target = match piece {
                Pieces::KING | Pieces::KNIGHT => self.get_non_slider_moves(piece, from),
                Pieces::ROOK | Pieces::BISHOP | Pieces::QUEEN => {
                    self.get_slider_moves(piece, from, bb_occupancy)
                }
                _ => panic!("Not a valid piece {piece}"),
            };
            // generate move according to movetype
            let bb_moves = match move_type {
                MoveType::All => bb_target & !bb_own_pieces,
                MoveType::Quiet => bb_target & bb_empty,
                MoveType::Capture => bb_target & bb_opponent_pieces,
            };
            self.add_move(board, piece, from, bb_moves, move_list);
        }
    }

    pub fn pawn(&self, board: &Board, move_list: &mut MoveList, move_type: MoveType) {
        const UP: i8 = 8;
        const DOWN: i8 = -8;

        let player = board.side_to_move();
        let bb_opponent_pieces = board.bb_side[board.side_to_not_move()];
        let bb_empty = !board.occupancy();
        let bb_fourth = BB_RANKS[Board::fourth_rank(player)];
        let direction = if player == Sides::WHITE { UP } else { DOWN };
        let rotation_count = (NrOf::SQUARES as i8 + direction) as u32;
        let mut bb_pawn = board.get_pieces(Pieces::PAWN, player);
        while bb_pawn > 0 {
            let from = bits::next(&mut bb_pawn);
            let to = (from as i8 + direction) as usize;
            if to > 64 {
                break;
            }

            let mut bb_moves = 0;
            // generate pawn pushes
            if move_type == MoveType::All || move_type == MoveType::Quiet {
                let bb_push = BB_SQUARES[to];
                let bb_one_step = bb_push & bb_empty;
                let bb_two_step = bb_one_step.rotate_left(rotation_count) & bb_empty & bb_fourth;
                bb_moves |= bb_one_step | bb_two_step
            }

            // generate pawn captures
            if move_type == MoveType::All || move_type == MoveType::Capture {
                let bb_targets = self.get_pawn_attacks(player, from);
                let bb_captures = bb_targets & bb_opponent_pieces;
                let bb_ep_captures = match board.gamestate.en_passant {
                    Some(ep) => bb_targets & BB_SQUARES[ep as usize],
                    None => 0,
                };
                // print_bitboard(bb_targets);
                bb_moves |= bb_captures | bb_ep_captures
            }

            self.add_move(board, Pieces::PAWN, from, bb_moves, move_list);
        }
    }

    pub fn castling(&self, board: &Board, move_list: &mut MoveList) -> bool {
        let player = board.side_to_move();
        let opponent = board.side_to_not_move();

        let castle_perm_w = (board.gamestate.castling & (Castling::WK | Castling::WQ)) > 0;
        let castle_perm_b = (board.gamestate.castling & (Castling::BK | Castling::BQ)) > 0;

        let bb_occupancy = board.occupancy();

        let mut bb_king = board.get_pieces(Pieces::KING, player);
        let from = bits::next(&mut bb_king);
        // White castling
        if castle_perm_w && player == Sides::WHITE {
            // kingside
            if board.gamestate.castling & Castling::WK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F1] | BB_SQUARES[Squares::G1];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::F1)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::KING, from, to, move_list);
                }
            }
            // Queenside
            if board.gamestate.castling & Castling::WQ > 0 {
                let bb_kingside_blockers =
                    BB_SQUARES[Squares::B1] | BB_SQUARES[Squares::C1] | BB_SQUARES[Squares::D1];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::D1)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::KING, from, to, move_list);
                }
            }
        }

        // Black castling
        if castle_perm_b && player == Sides::BLACK {
            // kingside
            if board.gamestate.castling & Castling::BK > 0 {
                let bb_queenside_blockers = BB_SQUARES[Squares::F8] | BB_SQUARES[Squares::G8];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::F8)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::QUEEN, from, to, move_list);
                }
            }
            // Queenside
            if board.gamestate.castling & Castling::BQ > 0 {
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B8] | BB_SQUARES[Squares::C8] | BB_SQUARES[Squares::D8];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::D8)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::QUEEN, from, to, move_list);
                }
            }
        }

        move_list.len() > 0
    }

    pub fn add_move(
        &self,
        board: &Board,
        piece: Piece,
        from: Square,
        to: Bitboard,
        move_list: &mut MoveList,
    ) {
        let mut bb_to = to;
        let player = board.side_to_move();
        let promotion_rank = Board::promotion_rank(player);
        let is_pawn = piece == Pieces::PAWN;

        while bb_to > 0 {
            let to_square = bits::next(&mut bb_to);
            let capture = board.piece_list[to_square];
            let en_passant = match board.gamestate.en_passant {
                Some(square) => is_pawn && (square as usize == to_square),
                None => false,
            };

            let promotion = is_pawn && Board::square_on_rank(to_square, promotion_rank);
            let double_push = is_pawn && ((to_square as i8 - from as i8).abs() == 16);
            let castling = (piece == Pieces::KING) && ((to_square as i8 - from as i8).abs() == 2);
            // if piece == Pieces::KING
            //     && self.square_attacked(&board, board.side_to_not_move(), to_square)
            // {
            //     continue;
            // }

            // add all data into a 64 bit variable
            let mut move_data = (piece)
                | from << Shift::FROM_SQ
                | to_square << Shift::TO_SQ
                | capture << Shift::CAPTURE
                | (en_passant as usize) << Shift::EN_PASSANT
                | (double_push as usize) << Shift::DOUBLE_STEP
                | (castling as usize) << Shift::CASTLING;

            if !promotion {
                move_data |= Pieces::NONE << Shift::PROMOTION;
                move_list.push(Move::new(move_data));
            } else {
                PROMOTION_PIECES.iter().for_each(|piece| {
                    let promotion_piece = *piece << Shift::PROMOTION;
                    move_list.push(Move::new(move_data | promotion_piece));
                });
            }
        }
    }
}

impl MoveGenerator {
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    // Determine if a square is attacked by 'attacker', on the given board.
    pub fn square_attacked(&self, board: &Board, attacker: Side, square: Square) -> bool {
        let attackers = board.bb_pieces[attacker];

        // Use the super-piece method: get the moves for each piece,
        // starting from the given square. This provides the squares where
        // a piece has to be, to be able to reach the given square.
        let occupancy = board.occupancy();
        let bb_king = self.get_non_slider_moves(Pieces::KING, square);
        let bb_rook = self.get_slider_moves(Pieces::ROOK, square, occupancy);
        let bb_bishop = self.get_slider_moves(Pieces::BISHOP, square, occupancy);
        let bb_knight = self.get_non_slider_moves(Pieces::KNIGHT, square);
        let bb_pawns = self.get_pawn_attacks(attacker ^ 1, square);
        let bb_queen = bb_rook | bb_bishop;

        // Then determine if such a piece is actually there: see if a rook
        // is on one of the squares a rook has to be to reach the given
        // square. Same for the queen, knight, etc... As soon as one is
        // found, the square is attacked.
        (bb_king & attackers[Pieces::KING] > 0)
            || (bb_rook & attackers[Pieces::ROOK] > 0)
            || (bb_queen & attackers[Pieces::QUEEN] > 0)
            || (bb_bishop & attackers[Pieces::BISHOP] > 0)
            || (bb_knight & attackers[Pieces::KNIGHT] > 0)
            || (bb_pawns & attackers[Pieces::PAWN] > 0)
    }
}

// ------------------ Test for movegeneration -------------------- \\

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn pawn_moves_start_pos() {
        let mut board = Board::new();
        let _ = Board::read_fen(
            &mut board,
            Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        );
        let mut move_list = MoveList::new();
        let move_generator = MoveGenerator::new();
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 16);
    }

    #[test]
    fn white_moves_random_pos() {
        let mut board = Board::new();
        let _ = Board::read_fen(
            &mut board,
            Some("Q7/6R1/P5b1/1P6/1P3kP1/p3qPb1/K4p2/2B1N3 w - - 0 1"),
        );
        let move_generator = MoveGenerator::new();
        // ----- Pawns ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 3);

        // ----- Rook ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::ROOK, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 9);

        // ----- Bishop ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::BISHOP, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 4);

        // ----- Knight ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::KNIGHT, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 3);

        // ----- Queen ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::QUEEN, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 12);

        // ----- King ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::KING, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 5);
    }

    #[test]
    fn black_moves_random_pos() {
        let mut board = Board::new();
        let _ = Board::read_fen(
            &mut board,
            Some("2b3rB/2P1k3/6P1/1Q2p1PP/1P2r3/2K1P3/1p4n1/8 b - - 0 1"),
        );
        let move_generator = MoveGenerator::new();
        // ----- Pawns ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        // let result: Vec<&Move> = move_list.moves.iter().filter(|x| x.data > 0).collect();
        // for x in &result {
        //     println!(
        //         "{}{}{}",
        //         PIECE_CHAR_CAPS[x.piece()],
        //         SQUARE_NAME[x.from()],
        //         SQUARE_NAME[x.to()],
        //     );
        // }
        // 4 Because 1 move with 4 possible outcomes of promoting the pawn.
        assert_eq!(move_list.len(), 4);

        // ----- Rook ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::ROOK, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 13);

        // ----- Bishop ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::BISHOP, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 7);

        // ----- Knight ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::KNIGHT, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 4);

        // ----- Queen ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::QUEEN, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 0);

        // ----- King ----- \\
        let mut move_list = MoveList::new();
        let _ = move_generator.piece(&board, Pieces::KING, &mut move_list, MoveType::All);
        assert_eq!(move_list.len(), 8);
    }

    #[test]
    fn white_king_side_castle() {
        let mut board = Board::new();
        let move_generator = MoveGenerator::new();

        // White castling
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/8/8/4K2R w KQkq - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // White castling rook under attack
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // Can't castle out of check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("4r3/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle through check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/6Pb/5P1P/4K2R w KAha - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/6PN/5P1P/4KP1R w KAha - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/6PN/5P1P/4K1PR w KAha - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));
    }

    #[test]
    fn white_queen_side_castle() {
        let mut board = Board::new();
        let move_generator = MoveGenerator::new();

        // White castling
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/8/8/R3K3 w HQha - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // White castling
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/2P5/PP1P4/R3K1N1 w HQha - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // Can't castle out of check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/2P2n2/PP1P4/R3K1N1 w HQha - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle through check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/2n5/PP1P4/R3K1N1 w HQha - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/2P5/PP1P4/R2QK1N1 w HQha - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/8/2P5/PP1P4/Rn2K1N1 w HQha - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));
    }

    #[test]
    fn black_king_side_castle() {
        let mut board = Board::new();
        let move_generator = MoveGenerator::new();

        // Black castling
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("1p2k2r/8/8/8/8/8/8/8 b HAka - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // Black castling
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("1p2k2r/4bp1p/6p1/8/8/8/8/1P4P1 b HAka - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // Can't castle out of check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("1p2k2r/4bp1p/6p1/8/B7/8/8/1P4P1 b HAka - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle through check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("1p2k2r/4bp1p/6pB/8/8/8/8/1P4P1 b HAka - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("1p2k1nr/4bp1p/6pn/8/8/8/8/1P4P1 b HAka - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("1p2kN1r/4bp1p/6pn/3n4/8/8/8/1P4P1 b KQkq - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));
    }

    #[test]
    fn black_queen_side_castle() {
        let mut board = Board::new();
        let move_generator = MoveGenerator::new();

        // Black castling
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("r3k3/8/8/8/8/8/8/8 b KQkq - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // Black castling
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("r3k3/qpb5/3n4/8/8/8/8/8 b HAhq - 0 1"));
        assert!(move_generator.castling(&board, &mut move_list));

        // Can't castle out of check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("r3k3/qpb5/3n4/8/8/8/8/4Q3 b KQkq - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle through check
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("r3k3/qpb5/3n4/8/7Q/8/8/8 b HAhq - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't castle piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("r2Pk3/qpb5/3n4/8/8/8/8/P7 b HAhq - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));

        // Can't piece in the way
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("rn2k3/qpb5/3n4/8/8/8/8/P7 b HAhq - 0 1"));
        assert!(!move_generator.castling(&board, &mut move_list));
    }

    #[test]
    fn white_en_passant() {
        let mut board = Board::new();
        let move_generator = MoveGenerator::new();

        // White left ep
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/3pP3/8/8/8/8 w - d6 0 1"));
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert!(move_list.len() == 2);

        // White right ep
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/4Pp2/8/8/8/8 w - f6 0 1"));
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert!(move_list.len() == 2);

        // White ep not possible
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/4Pp2/8/8/8/8 w - - 0 1"));
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert!(move_list.len() == 1);
    }

    #[test]
    fn black_en_passant() {
        let mut board = Board::new();
        let move_generator = MoveGenerator::new();

        // Black left ep
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/1Pp5/8/8/8 b - b3 0 1"));
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert!(move_list.len() == 2);

        // White right ep
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/pP6/8/8/8 b - b3 0 1"));
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert!(move_list.len() == 2);

        // White ep not possible
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("8/8/8/8/pP6/8/8/8 b - - 0 1"));
        let _ = move_generator.pawn(&board, &mut move_list, MoveType::All);
        assert!(move_list.len() == 1);
    }

    #[test]
    fn white_legal_moves() {
        let mut board = Board::new();
        let move_generator = MoveGenerator::new();

        let possible_moves = [
            "b5a4", "b5b4", "b5c4", "b5a5", "b5c5", "b5a6", "b5b6", "b5c6", "h1f2", "h1g3", "d6c4",
            "d6e4", "d6f5", "d6b7", "d6c8", "d6e8", "g2g1", "g2b2", "g2c2", "g2d2", "g2e2", "g2f2",
            "g2h2", "g2g3", "g2g4", "g2g5", "g8g7", "g8a8", "g8b8", "g8c8", "g8d8", "g8e8", "g8f8",
            "g8h8", "h5d1", "h5e2", "h5f3", "h5g4", "c3c4", "g6g7", "e7e8q", "e7e8r", "e7e8n",
            "e7e8b", "f7f8q", "f7f8r", "f7f8n", "f7f8b",
        ];
        // Generate random legal moves
        let mut move_list = MoveList::new();
        let _ = board.read_fen(Some("b5R1/4PP2/3Nk1P1/1K5B/8/1pP5/1p4Rp/6bN w - - 0 1"));
        let _ = move_generator.generate_moves(&board, &mut move_list, MoveType::All);
        for x in 0..move_list.moves.len() {
            if move_list.moves[x].data != 0 {
                assert_eq!(move_list.moves[x].as_string(), possible_moves[x]);
            }
        }
    }
}
