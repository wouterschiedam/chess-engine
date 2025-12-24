use crate::board::Board;
use crate::utils::display::print_position;
use crate::utils::perft::{divide, divide_with_known, drill_down, perft, perft_verbose};

/// Interactive game loop
pub fn play_game(starting_fen: Option<&str>, _engine_plays_black: bool, _depth: u8) {
    // Initialize your board type here
    let _board = Board::build(starting_fen);
}

/// Display a board position
pub fn show_position(fen: Option<&str>) {
    let board = Board::build(fen);

    if let Some(f) = fen {
        println!("FEN: {}", f);
    }

    print_position(&board, false, None);
}

/// Run perft on a position with various debugging options
///
/// This function handles all perft modes:
/// - Basic: Just show node count and time
/// - Verbose: Show each move and its count
/// - Divide: Show breakdown (standard debugging)
/// - Known: Compare against known values (best for debugging)
/// - Drill: Deep dive into a specific move
/// - Compare: Quick check against known totals
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

    // Display position info
    if let Some(f) = fen {
        println!("FEN: {}", f);
    } else {
        println!("FEN: {}", crate::defs::FEN_START_POSITION);
    }

    println!("Depth: {}", depth);
    println!();

    // ========== DRILL MODE ==========
    // Deep dive into a specific move
    // Can be combined with --known to compare against Stockfish
    if let Some(move_str) = drill {
        drill_down(&mut board, &move_str, depth, known, position);
        return;
    }

    // ========== COMPARE MODE ==========
    // Note: --compare is deprecated, use --known instead which uses Stockfish
    if compare {
        println!("Note: --compare is deprecated. Use --known to compare against Stockfish values.");
        println!("Falling back to basic perft...\n");
    }

    // ========== KNOWN MODE ==========
    // Compare each move against known values (best debugging tool)
    if known {
        divide_with_known(&mut board, depth, position);
        return;
    }

    // ========== DIVIDE MODE ==========
    // Show breakdown for each move (standard debugging)
    if divide_mode {
        println!("Divide results:");
        println!("{}", "=".repeat(50));
        divide(&mut board, depth);
        return;
    }

    // ========== VERBOSE MODE ==========
    // Show each move and its count
    if verbose {
        println!("Perft results (verbose):");
        println!("{}", "=".repeat(50));
        let total = perft_verbose(&mut board, depth);
        println!("{}", "=".repeat(50));
        println!("Total nodes: {}", total);
        return;
    }

    // ========== BASIC MODE ==========
    // Just show node count, time, and performance
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
