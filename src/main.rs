#![allow(dead_code)]
#![allow(unused)]

mod board;
mod cli;
mod defs;
mod movegen;
mod tui;
mod uci;
mod utils;

use clap::CommandFactory;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse_args();

    match cli.command {
        Some(Commands::Play { fen, black, depth }) => {
            cli::commands::play_game(fen.as_deref(), black, depth);
        }

        Some(Commands::Show { fen }) => {
            cli::commands::show_position(Some(&fen));
        }

        Some(Commands::Perft {
            fen,
            depth,
            verbose,
            divide,
            compare,
            known,
            drill,
            position,
        }) => {
            cli::commands::run_perft(
                fen.as_deref(),
                depth,
                verbose,
                divide,
                compare,
                known,
                drill,
                position,
            );
        }

        Some(Commands::Uci) => {
            uci::run_uci();
        }

        Some(Commands::Tournament) => {
            cli::commands::run_tournament();
        }

        None => {
            Cli::command().print_help().expect("Failed to print help");
        }
    }
}
