use crate::board::Board;
use crate::utils::perft::{divide, divide_with_known, drill_down, perft, perft_verbose};

pub fn run_perft(
    fen: Option<&str>,
    depth: usize,
    verbose: bool,
    divide_mode: bool,
    compare: bool,
    known: bool,
    drill: Option<String>,
    position: bool,
) {
    let mut board = Board::build(fen);

    if let Some(f) = fen {
        println!("FEN: {}", f);
    } else {
        println!("FEN: {}", crate::defs::FEN_START_POSITION);
    }

    println!("Depth: {}", depth);

    if let Some(ref move_str) = drill {
        drill_down(&mut board, move_str, depth, known, position);
        return;
    }

    if compare {
        println!("Note: --compare is deprecated. Use --known to compare against Stockfish values.");
        println!("Falling back to basic perft...\n");
        return;
    }

    if known {
        divide_with_known(&mut board, depth, position);
        return;
    }

    if divide_mode {
        println!("Divide results:");
        println!("{}", "=".repeat(50));
        divide(&mut board, depth);
        return;
    }

    if verbose {
        println!("Perft results (verbose):");
        println!("{}", "=".repeat(50));
        let total = perft_verbose(&mut board, depth);
        println!("{}", "=".repeat(50));
        println!("Total nodes: {}", total);
        return;
    }

    let start = std::time::Instant::now();
    let nodes = perft(&mut board, depth);
    let elapsed = start.elapsed();

    println!("Nodes: {}", nodes);
    println!("Time: {:.3}s", elapsed.as_secs_f64());
    if elapsed.as_secs_f64() > 0.0 {
        let nps = nodes as f64 / elapsed.as_secs_f64();
        println!("Nodes/sec: {:.0}", nps);
    }
}
