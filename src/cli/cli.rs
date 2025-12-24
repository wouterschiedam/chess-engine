use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "chess-engine")]
#[command(version = "0.1.0")]
#[command(about = "A chess engine with CLI interface for testing and debugging", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start an interactive game against the engine
    Play {
        /// Start from a FEN position instead of initial position
        #[arg(short, long)]
        fen: Option<String>,

        /// Engine plays as black (you play white)
        #[arg(short, long)]
        black: bool,

        /// Search depth for engine moves
        #[arg(short, long, default_value = "6")]
        depth: u8,
    },

    /// Display a board position from FEN
    Show {
        /// FEN string to display
        fen: String,
    },

    /// Run perft (performance test) to verify move generation correctness
    ///
    /// Perft counts the number of legal move sequences at each depth.
    /// Use this to debug and verify your move generation implementation.
    Perft {
        /// FEN string for the position (default: starting position)
        ///
        /// Examples:
        ///   -f "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        ///   -f "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        #[arg(short, long)]
        fen: Option<String>,

        /// Depth to search (default: 1)
        ///
        /// Higher depths take exponentially longer. Recommended:
        ///   - Depth 1-2: Instant
        ///   - Depth 3: < 1 second
        ///   - Depth 4: Several seconds
        ///   - Depth 5+: Minutes to hours
        #[arg(short, long, default_value = "1")]
        depth: usize,

        /// Verbose output - show move counts for each move
        ///
        /// Useful for seeing the distribution of moves and identifying
        /// which moves generate the most nodes.
        #[arg(short, long)]
        verbose: bool,

        /// Divide mode - shows breakdown for each move (RECOMMENDED for debugging)
        ///
        /// This is the standard perft debugging tool. It shows:
        ///   - Each move and its node count
        ///   - Total at the end
        ///
        /// Use this when you want to see which moves are being generated
        /// and how many nodes each leads to.
        #[arg(long)]
        divide: bool,

        /// Compare with known perft values (DEPRECATED: use --known instead)
        ///
        /// This flag is deprecated. Use --known to compare against Stockfish values
        /// which works for any position and depth.
        #[arg(short, long, hide = true)]
        compare: bool,

        /// Divide with Stockfish values - compares each move against expected counts (BEST for debugging)
        ///
        /// This is the most powerful debugging tool. It shows:
        ///   - Each move with ✓ if correct, ✗ if wrong (with difference)
        ///   - Highlights which specific moves are generating incorrect counts
        ///   - Works for any position and depth (uses Stockfish for reference values)
        ///
        /// Use this when you have incorrect perft results and need to find
        /// which specific move is causing the problem.
        ///
        /// Note: Requires Stockfish to be installed and available in PATH.
        #[arg(long)]
        known: bool,

        /// Drill down into a specific move - shows what happens after that move
        ///
        /// Makes the specified move, then shows the divide output for the
        /// resulting position. Useful for deep debugging of problematic moves.
        ///
        /// Can be combined with --known to compare results against Stockfish.
        ///
        /// Example: --drill h2h3
        ///   This will make h2h3, then show all moves from that position.
        ///
        /// Example: --drill a2a4 --known
        ///   This will make a2a4, then show all moves with Stockfish comparison.
        #[arg(long, value_name = "MOVE")]
        drill: Option<String>,

        /// Print board positions for problematic moves
        ///
        /// When used with --known: Shows positions after moves that have discrepancies
        /// When used with --drill: Shows position after the drill move, and positions
        ///   after any subsequent moves that have discrepancies (if --known is also used)
        ///
        /// This helps identify which specific positions are causing perft differences.
        #[arg(long)]
        position: bool,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
