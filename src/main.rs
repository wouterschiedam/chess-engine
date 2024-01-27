// use board::{defs::SQUARE_NAME, Board};
// use movegen::{
//     defs::{MoveList, MoveType},
//     MoveGenerator,
// };
//
// fn main() {
//     let mut board = Board::new();
//     board.read_fen(Some(
//         "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
//     ));
//
//     let move_gen = MoveGenerator::new();
//     let mut move_list = MoveList::new();
//
//     move_gen.generate_moves(&board, &mut move_list, MoveType::All);
//
//     for mv in move_list.moves.iter() {
//         let the_move = format!("{}{}", SQUARE_NAME[mv.from()], SQUARE_NAME[mv.to()]);
//         if mv.data > 0 {
//             println!("{}", the_move);
//         }
//     }
// }

use engine::Engine;
mod board;
mod comm;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod movegen;
mod search;
pub mod ui;
//
fn main() {
    // TODO: SPLIT THE PROGRAMS INTO TWO AND COPY BAORD MOVEGEN HELPERS
    // Start thread for the chess engine
    let mut engine = Engine::new();
    let _ = engine.run();
}
