use engine::Engine;
mod board;
mod comm;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod movegen;
mod search;

fn main() {
    // Start thread for the chess engine
    let mut engine = Engine::new();
    let _ = engine.run();
}
