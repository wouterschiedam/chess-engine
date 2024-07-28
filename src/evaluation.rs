use material::count;

use crate::{board::Board, defs::Sides};

pub mod defs;
pub mod material;
pub mod psqt;

pub fn evaluate_position(board: &Board) -> i16 {
    //const KING_ONLY: i16 = 20; // PSQT-points

    let side = board.gamestate.active_color as usize;

    let w_psqt = board.gamestate.psqt[Sides::WHITE];
    let b_psqt = board.gamestate.psqt[Sides::BLACK];

    let mut value = w_psqt - b_psqt;

    let (w_material, b_material) = count(board);

    value += w_material as i16 - b_material as i16;

    // If one of the sides is down to a bare king, apply the KING_EDGE PSQT
    // to drive that king to the edge and mate it.
    // if w_material < KING_ONLY || b_material < KING_ONLY {
    //     let w_king_edge = KING_EDGE[board.king_square(Sides::WHITE)] as i16;
    //     let b_king_edge = KING_EDGE[board.king_square(Sides::BLACK)] as i16;
    //     value += w_king_edge - b_king_edge;
    // }

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
