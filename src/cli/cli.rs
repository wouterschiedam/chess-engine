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
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

