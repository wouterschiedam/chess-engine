use material::count;

use crate::{
    board::{defs::Pieces, Board},
    defs::Sides,
    evaluation::{defs::PIECE_VALUES, psqt::KING_EDGE},
};

pub mod defs;
pub mod material;
pub mod psqt;

pub fn evaluate_position(board: &Board) -> i16 {
    const KING_ONLY: i16 = 20; // PSQT-points
    const QUEEN_LOSS_PENALTY: i16 = 9000;
    const ROOK_LOSS_PENALTY: i16 = 500;
    const BISHOP_LOSS_PENALTY: i16 = 320;
    const KNIGHT_LOSS_PENALTY: i16 = 310;

    let side = board.gamestate.active_color as usize;

    let w_psqt = board.gamestate.psqt[Sides::WHITE];
    let b_psqt = board.gamestate.psqt[Sides::BLACK];

    let mut value = w_psqt - b_psqt;

    let (w_material, b_material) = count(board);

    println!("{}", w_material);
    println!("{}", b_material);

    value += w_material as i16 - b_material as i16;

    // If one of the sides is down to a bare king, apply the KING_EDGE PSQT
    // to drive that king to the edge and mate it.
    if w_psqt < KING_ONLY || b_psqt < KING_ONLY {
        let w_king_edge = KING_EDGE[board.king_square(Sides::WHITE)] as i16;
        let b_king_edge = KING_EDGE[board.king_square(Sides::BLACK)] as i16;
        value += w_king_edge - b_king_edge;
    }

    // Calculate penalties for losing high-value pieces
    let material_difference = w_material as i16 - b_material as i16;

    if material_difference > 900 {
        // Queen is lost
        value -= QUEEN_LOSS_PENALTY;
    } else if material_difference > 500 {
        // Rook is lost
        value -= ROOK_LOSS_PENALTY;
    } else if material_difference > 320 {
        // Bishop is lost
        value -= BISHOP_LOSS_PENALTY;
    } else if material_difference > 310 {
        // Knight is lost
        value -= KNIGHT_LOSS_PENALTY;
    }

    // This function calculates the evaluation from white's point of view:
    // a positive value means "white is better", a negative value means
    // "black is better". Alpha/Beta requires the value returned from the
    // viewpoint of the side that is being evaluated. Therefore if it is
    // black to move, the value must first be flipped to black's viewpoint
    // before it can be returned.
    if side == Sides::BLACK {
        value = -value;
    }

    value / 100
}
