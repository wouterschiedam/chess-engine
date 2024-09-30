use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Write, BufWriter};
use csv::Reader;
use serde::Deserialize;
use std::error::Error;

use crate::board::Board;
use crate::movegen::{self, MoveGenerator};
use crate::search::defs::{SearchRefs, INF};
use crate::search::Search;

/// Struct to hold puzzle data (FEN and the correct moves)
#[derive(Debug)]
pub struct Puzzle {
    pub fen: String,
    pub solution_moves: Vec<String>, // Moves in UCI format, e.g. ["e2e4", "d7d5"]
}

#[derive(Debug, Deserialize)]
struct PuzzleRecord {
    PuzzleId: String,
    FEN: String,
    Moves: String, // UCI format moves, comma-separated
    Rating: String,
    RatingDeviation: String,
    Popularity: String,
    NbPlays: String,
    Themes: String,
    GameUrl: String,
}

impl Puzzle {

    pub fn new (fen: String, solution_moves: Vec<String>) -> Self {
        Self { fen, solution_moves }
    }

    pub fn read_puzzles_from_csv(file_path: &str) -> Result<Vec<Puzzle>, Box<dyn Error>> {
        // Open the CSV file
        let mut rdr = csv::Reader::from_path(file_path)?;

        // Vector to store Puzzle objects
        let mut puzzles = Vec::new();

        // Iterate over each record and create Puzzle objects
        for result in rdr.deserialize::<PuzzleRecord>() {
            match result {
                Ok(record) => {
                    let record = PuzzleRecord {
                        PuzzleId: record.PuzzleId,
                        FEN: record.FEN,
                        Moves: record.Moves,
                        Rating: record.Rating,
                        RatingDeviation: record.RatingDeviation,
                        Popularity: record.Popularity,
                        NbPlays: record.NbPlays,
                        Themes: record.Themes,
                        GameUrl: record.GameUrl,
                    };

                    // Convert moves from CSV (comma-separated string) to Vec<String>
                    let solution_moves: Vec<String> = record.Moves.split_whitespace().map(String::from).collect();

                    // Create a new Puzzle object and add it to the puzzles vector
                    let puzzle = Puzzle::new(record.FEN, solution_moves);
                    puzzles.push(puzzle);
                }
                Err(e) => {
                    eprintln!("Error reading CSV: {}", e);
                    return Err(Box::new(e)); // Properly box the error
                }
            }
        }

        Ok(puzzles)
    }


}
