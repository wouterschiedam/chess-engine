use std::{
    io::{self},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crossbeam_channel::Sender;

use crate::{
    board::Board,
    defs::{About, Sides, FEN_START_POSITION},
    engine::defs::Information,
    evaluation::{evaluate_position, material::count},
    extra::print,
    movegen::defs::Move,
    search::{
        defs::{
            GameTime, PerftSummary, SearchCurrentMove, SearchStats, SearchSummary, CHECKMATE,
            CHECKMATE_THRESHOLD, INF,
        },
        Search,
    },
};

use super::{CommControl, CommReport, IComm};

// Enum for possible uci commands
#[derive(PartialEq, Clone, Debug)]
pub enum UciReport {
    // Uci commands
    Uci,
    UciNewGame,
    IsReady,
    // SetOption(EngineOptionName),
    Position(String, Vec<String>),
    GoInfinite,
    GoDepth(i8),
    GoPerft(i8),
    GoMoveTime(u128),
    GoNodes(usize),
    GoGameTime(GameTime),
    Stop,
    Quit,

    // // Custom commands
    Board,
    // History,
    // Eval,
    // Help,

    // Empty or unknown command.
    Unknown,
}

pub struct Uci {
    control_handle: Option<JoinHandle<()>>, // Control handle is
    report_handle: Option<JoinHandle<()>>,  // Report handle is
    control_tx: Option<Sender<CommControl>>, // CommControl is al possible responses engine can
                                            // give to uci
}

impl Uci {
    pub fn new() -> Self {
        Self {
            control_handle: None,
            report_handle: None,
            control_tx: None,
        }
    }
}
impl IComm for Uci {
    fn init(
        &mut self,
        info_sender: Sender<Information>,
        board: Arc<Mutex<Board>>,
        // options: Arc<Vec<EngineOption>>,
    ) {
        self.report_thread(info_sender);
        self.control_thread(board);
    }

    // Send messages to the control thread
    fn send(&self, msg: CommControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(msg).expect("Failed sending message {msg}");
        }
    }

    // After the engine sends 'quit' to the control thread, it will call
    // wait_for_shutdown() and then wait here until shutdown is completed.
    fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.report_handle.take() {
            h.join().expect("error stopping thread");
        }

        if let Some(h) = self.control_handle.take() {
            h.join().expect("error stopping thread");
        }
        // println!("All threads have stopped");
    }
    // This function just returns the name of the communication protocol.
    fn get_protocol_name(&self) -> &'static str {
        "uci"
    }
}

// Implement report thread
impl Uci {
    pub fn report_thread(&mut self, info_sender: Sender<Information>) {
        // Create thread-local variables
        let mut t_incoming_data = String::from("");
        let t_info_sender = info_sender; // Report sender

        // Not sure why to add move but the compiler told me
        let report_thread = thread::spawn(move || {
            let mut quit = false;

            while !quit {
                // read data from std:in
                io::stdin()
                    .read_line(&mut t_incoming_data)
                    .expect("error reading input");

                let response_report = Self::create_report(&t_incoming_data);

                if response_report.is_valid() {
                    // send to engine receiving thread
                    let _x = t_info_sender
                        .send(Information::Comm(response_report.clone()))
                        .expect("Error sending message");

                    quit = response_report == CommReport::Uci(UciReport::Quit);
                }
                // clear data for next incoming message
                t_incoming_data = String::from("");
            }
        });

        self.report_handle = Some(report_thread);
    }
}

// implement control thread
impl Uci {
    pub fn control_thread(&mut self, board: Arc<Mutex<Board>>) {
        // Create an incoming channel for the control thread.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<CommControl>();

        let t_board = Arc::clone(&board);
        // not sure why to add move
        let control_thread = thread::spawn(move || {
            let mut quit = false;
            while !quit {
                let control = control_rx.recv().expect("error receiving message");

                match control {
                    CommControl::Identify => {
                        println!("id name {} {}", About::ENGINE, About::VERSION);
                        println!("id author {}", About::AUTHOR);
                        println!("uciok");
                    }
                    CommControl::Ready => println!("readyok"),
                    CommControl::Quit => quit = true,
                    CommControl::SearchSummary(summary) => Self::search_summary(&summary, &t_board),
                    CommControl::SearchStats(stats) => Self::search_stats(&stats),
                    CommControl::SearchCurrMove(current) => Self::search_current_move(&current),
                    CommControl::InfoString(info) => Self::info_string(&info),
                    CommControl::BestMove(best_move) => Self::find_best_move(&best_move),
                    CommControl::PerftScore(perftsum) => Self::perft_summary(&perftsum),
                    CommControl::PrintBoard => Self::print_board(&t_board),
                    CommControl::PrintHistory => (),
                    CommControl::PrintHelp => (),

                    CommControl::Update => (),
                }
            }
        });

        self.control_handle = Some(control_thread);
        self.control_tx = Some(control_tx);
    }
}

// Some private function for Uci
impl Uci {
    // Create a rapport message so the engine understands what to do
    fn create_report(message: &str) -> CommReport {
        let clean_message = message.trim_end().to_string();
        match clean_message {
            cmd if cmd == "uci" => CommReport::Uci(UciReport::Uci),
            cmd if cmd == "ucinewgame" => CommReport::Uci(UciReport::UciNewGame),
            cmd if cmd == "isready" => CommReport::Uci(UciReport::IsReady),
            cmd if cmd == "stop" => CommReport::Uci(UciReport::Stop),
            cmd if cmd == "quit" || cmd == "exit" => CommReport::Uci(UciReport::Quit),
            cmd if cmd.starts_with("position") => Self::parse_position(&cmd),
            // cmd if cmd.starts_with("setoption") => Self::parse_options(&cmd),
            cmd if cmd.starts_with("go") => Self::parse_go(&cmd),
            cmd if cmd == "d" => CommReport::Uci(UciReport::Board),
            _ => CommReport::Uci(UciReport::Unknown),
        }
    }

    fn parse_position(command: &str) -> CommReport {
        enum Tokens {
            Nothing,
            Fen,
            Moves,
        }

        let fen_parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        let mut fen = String::from("");
        let mut moves: Vec<String> = Vec::new();
        let mut skip_fen = false;
        let mut token = Tokens::Nothing;

        for part in fen_parts {
            match part {
                t if t == "position" => (), // Skip. We know we're parsing "position".
                t if t == "startpos" => skip_fen = true, // "fen" is now invalidated.
                t if t == "fen" && !skip_fen => token = Tokens::Fen,
                t if t == "moves" => token = Tokens::Moves,
                _ => match token {
                    Tokens::Nothing => (),
                    Tokens::Fen => {
                        fen.push_str(&part[..]);
                        fen.push(' ');
                    }
                    Tokens::Moves => moves.push(part),
                },
            }
        }
        // if fen part in command use default position
        if fen.is_empty() {
            fen = String::from(FEN_START_POSITION);
        }

        CommReport::Uci(UciReport::Position(fen.trim().to_string(), moves))
    }

    fn parse_go(command: &str) -> CommReport {
        // Possible params that are added to go command
        enum Tokens {
            Nothing,
            Depth,
            Nodes,
            MoveTime,
            WTime,
            BTime,
            WInc,
            BInc,
            MovesToGo,
            Perft,
        }

        let go_parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        let mut report = CommReport::Uci(UciReport::Unknown);
        let mut token = Tokens::Nothing;
        let mut game_time = GameTime::new(0, 0, 0, 0, None);
        for part in go_parts {
            match part {
                t if t == "go" => report = CommReport::Uci(UciReport::GoInfinite),
                t if t == "infinite" => break, // Already Infinite; nothing more to do.
                t if t == "depth" => token = Tokens::Depth,
                t if t == "perft" => token = Tokens::Perft,
                t if t == "movetime" => token = Tokens::MoveTime,
                t if t == "nodes" => token = Tokens::Nodes,
                t if t == "wtime" => token = Tokens::WTime,
                t if t == "btime" => token = Tokens::BTime,
                t if t == "winc" => token = Tokens::WInc,
                t if t == "binc" => token = Tokens::BInc,
                t if t == "movestogo" => token = Tokens::MovesToGo,
                _ => match token {
                    Tokens::Nothing => (),
                    Tokens::Depth => {
                        let depth = part.parse::<i8>().unwrap_or(1);
                        report = CommReport::Uci(UciReport::GoDepth(depth));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::Perft => {
                        let perft = part.parse::<i8>().unwrap_or(1);
                        report = CommReport::Uci(UciReport::GoPerft(perft));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::MoveTime => {
                        let milliseconds = part.parse::<u128>().unwrap_or(1000);
                        report = CommReport::Uci(UciReport::GoMoveTime(milliseconds));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::Nodes => {
                        let nodes = part.parse::<usize>().unwrap_or(1);
                        report = CommReport::Uci(UciReport::GoNodes(nodes));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::WTime => game_time.white_time = part.parse::<u128>().unwrap_or(0),
                    Tokens::BTime => game_time.black_time = part.parse::<u128>().unwrap_or(0),
                    Tokens::WInc => game_time.white_time_incr = part.parse::<u128>().unwrap_or(0),
                    Tokens::BInc => game_time.black_time_incr = part.parse::<u128>().unwrap_or(0),
                    Tokens::MovesToGo => {
                        game_time.moves_to_go = if let Ok(x) = part.parse::<usize>() {
                            Some(x)
                        } else {
                            None
                        }
                    }
                },
            }
        }

        // if mode is still GoInfinite we must switch to gametime mode
        let is_default_mode = report == CommReport::Uci(UciReport::GoInfinite);
        let has_time = game_time.white_time > 0 || game_time.black_time > 0;
        let has_inc = game_time.white_time_incr > 0 || game_time.black_time_incr > 0;
        let is_game_time = has_time || has_inc;
        if is_default_mode && is_game_time {
            report = CommReport::Uci(UciReport::GoGameTime(game_time));
        }

        report
    }

    // fn parse_options(command: &str) -> CommReport {
    //     CommReport::Uci(UciReport::SetOption())
    // }

    fn search_summary(summary: &SearchSummary, board: &Arc<Mutex<Board>>) {
        // Check for checkmate
        let score = if summary.cp == -INF {
            format!("draw")
        } else if (summary.cp.abs() >= CHECKMATE_THRESHOLD) && (summary.cp.abs() <= CHECKMATE) {
            // number of plays left to mate
            let plays = CHECKMATE - summary.cp.abs();

            // Check if the number of ply's is odd
            let is_odd = plays % 2 == 1;

            // Calculate number of moves to mate
            let moves = if is_odd { (plays + 1) / 2 } else { plays / 2 };

            // If the engine is being mated itself, flip the score.
            let flip = if summary.cp < 0 { -1 } else { 1 };

            // Report the mate
            format!("mate {}", moves * flip)
        } else {
            // Report the normal score if there's no mate detected.
            format!("cp {}", summary.cp)
        };

        // Report depth and seldepth (if available).
        let depth = if summary.seldepth > 0 {
            format!("depth {} seldepth {}", summary.depth, summary.seldepth)
        } else {
            format!("depth {}", summary.depth)
        };

        // // Only display hash full if not 0
        // let hash_full = if summary.hash_full > 0 {
        //     format!(" hashfull {} ", summary.hash_full)
        // } else {
        //     String::from(" ")
        // };

        let pv = summary.pv_as_string();

        let board_lock = board.lock().expect("Error locking board");
        let eval = evaluate_position(&board_lock);
        let w_psqt = &board_lock.gamestate.psqt[Sides::WHITE];
        let b_psqt = &board_lock.gamestate.psqt[Sides::BLACK];

        let (w_material, b_material) = count(&board_lock);

        let info = format!(
            "info score {} {} time {} nodes {} nps {} pv {} eval {} w_psqt {} b_psqt {} w_material {} b_material {}",
            score, depth, summary.time, summary.nodes, summary.nps, pv, eval, w_psqt, b_psqt, w_material, b_material
        );

        println!("{info}");
    }

    fn search_stats(stats: &SearchStats) {
        let hash_full = if stats.hash_full > 0 {
            format!(" hashfull {}", stats.hash_full)
        } else {
            String::from("")
        };

        println!(
            "info time {} nodes {} nps {}{}",
            stats.time, stats.nodes, stats.nps, hash_full
        );
    }

    fn search_current_move(current: &SearchCurrentMove) {
        format!(
            "info currmove {} currmovenumber {}",
            current.curr_move.as_string(),
            current.curr_move_number
        );
    }

    fn info_string(msg: &str) {
        println!("info string {msg}");
    }

    fn find_best_move(bestmove: &Move) {
        println!("bestmove {}", bestmove.as_string());
    }

    fn perft_summary(summary: &PerftSummary) {
        let mut sorted_vec: Vec<_> = summary.moves.clone().into_iter().collect();

        // Sort the vector based on the keys
        sorted_vec.sort_by(|a, b| a.0.cmp(&b.0));

        // Iterate over the sorted vector
        for (key, value) in sorted_vec {
            println!("{}: {}", key, value);
        }

        println!(
            "\nDepth: {}\nnodes: {}\nNodes per second: {}\nTime: {:?} milliseconds\n",
            summary.depth,
            summary.nodes,
            Search::nodes_per_sec(summary.nodes as usize, summary.time.as_millis()),
            summary.time.as_millis()
        );

        summary.move_stats.log();
    }

    fn print_board(board: &Arc<Mutex<Board>>) {
        print::print_position(&board.lock().expect("Error locking board"));
    }
}
