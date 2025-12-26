// UCI Protocol command parsing
// Handles parsing of UCI commands from stdin

use crate::uci::GoParams;

/// UCI command enum - represents all possible UCI commands
#[derive(Debug, Clone, PartialEq)]
pub enum UciCommand {
    Uci,
    Debug {
        enable: bool,
    },
    IsReady,
    SetOption {
        name: String,
        value: Option<String>,
    },
    UciNewGame,
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
    Go(GoParams),
    Stop,
    Quit,

    // Custom debug commands (not standard UCI)
    Moves {
        square: String,
    },
    Bitboard {
        piece_type: String,
        color: String,
    },
    Bitboards,
}

impl UciCommand {
    /// Parse a UCI command string into a UciCommand enum
    pub fn parse(input: &str) -> Option<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "uci" => Some(UciCommand::Uci),
            "debug" => {
                if parts.len() >= 2 {
                    let enable = parts[1] == "on";
                    Some(UciCommand::Debug { enable })
                } else {
                    None
                }
            }
            "isready" => Some(UciCommand::IsReady),
            "setoption" => {
                // Format: setoption name <name> [value <value>]
                let name = parts
                    .iter()
                    .position(|&p| p == "name")
                    .and_then(|i| parts.get(i + 1));

                let value = parts
                    .iter()
                    .position(|&p| p == "value")
                    .and_then(|i| parts.get(i + 1))
                    .map(|s| s.to_string());

                name.map(|n| UciCommand::SetOption {
                    name: n.to_string(),
                    value,
                })
            }
            "ucinewgame" => Some(UciCommand::UciNewGame),
            "position" => {
                // Format: position [fen <fen> | startpos] [moves <move1> ...]
                let mut fen: Option<String> = None;
                let mut moves: Vec<String> = Vec::new();
                let mut in_moves = false;
                let mut in_fen = false;
                let mut fen_parts: Vec<String> = Vec::new();

                for (i, &part) in parts.iter().enumerate() {
                    if part == "fen" {
                        in_fen = true;
                    } else if part == "startpos" {
                        in_fen = false;
                        fen = None; // Default position
                    } else if part == "moves" {
                        in_fen = false;
                        in_moves = true;
                    } else if in_fen {
                        fen_parts.push(part.to_string());
                    } else if in_moves {
                        moves.push(part.to_string());
                    }
                }

                // Combine FEN parts
                if !fen_parts.is_empty() {
                    fen = Some(fen_parts.join(" "));
                }

                Some(UciCommand::Position { fen, moves })
            }
            "go" => Some(UciCommand::Go(parse_go_params(&parts))),
            "stop" => Some(UciCommand::Stop),
            "quit" => Some(UciCommand::Quit),

            // Custom debug commands
            "moves" => {
                if parts.len() >= 2 {
                    Some(UciCommand::Moves {
                        square: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }

            "bitboard" => {
                // Format: bitboard <piece_type> <color>
                // Example: bitboard pawn white
                if parts.len() >= 3 {
                    Some(UciCommand::Bitboard {
                        piece_type: parts[1].to_string(),
                        color: parts[2].to_string(),
                    })
                } else {
                    None
                }
            }

            "bitboards" => Some(UciCommand::Bitboards),

            _ => None,
        }
    }
}

/// Parse parameters for the 'go' command
fn parse_go_params(parts: &[&str]) -> GoParams {
    let mut params = GoParams::default();

    for (i, &part) in parts.iter().enumerate() {
        match part {
            "depth" => {
                if let Some(depth_str) = parts.get(i + 1) {
                    params.depth = depth_str.parse().ok();
                }
            }
            "movetime" => {
                if let Some(movetime_str) = parts.get(i + 1) {
                    params.movetime = movetime_str.parse().ok();
                }
            }
            "wtime" => {
                if let Some(wtime_str) = parts.get(i + 1) {
                    params.wtime = wtime_str.parse().ok();
                }
            }
            "btime" => {
                if let Some(btime_str) = parts.get(i + 1) {
                    params.btime = btime_str.parse().ok();
                }
            }
            "winc" => {
                if let Some(winc_str) = parts.get(i + 1) {
                    params.winc = winc_str.parse().ok();
                }
            }
            "binc" => {
                if let Some(binc_str) = parts.get(i + 1) {
                    params.binc = binc_str.parse().ok();
                }
            }
            "nodes" => {
                if let Some(nodes_str) = parts.get(i + 1) {
                    params.nodes = nodes_str.parse().ok();
                }
            }
            "infinite" => {
                params.infinite = true;
            }
            _ => {}
        }
    }

    params
}
