mod board;
mod defs;
mod cli;
mod utils;
mod movegen;

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse_args();
    
    match cli.command {
        Commands::Play { fen, black, depth } => {
            cli::commands::play_game(fen.as_deref(), black, depth);
        }
        
        Commands::Show { fen } => {
            cli::commands::show_position(Some(&fen));
        }
    }
}
