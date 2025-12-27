use crate::board::Board;
use crate::tui::setup::{SetupMenu, draw_setup};
use crate::tui::tournament::draw_results;
use crate::tui::{TournamentApp, TournamentState, ui::draw};
use crate::utils::display::print_position;
use crate::utils::perft::{divide, divide_with_known, drill_down, perft, perft_verbose};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

/// Interactive game loop
pub fn play_game(starting_fen: Option<&str>, _engine_plays_black: bool, _depth: u8) {
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

/// Run tournament - engine vs engine matches
///
/// Play your engine against itself or another version to test and compare.
/// Displays a TUI for watching games and viewing statistics.
pub fn run_tournament() {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{Terminal, backend::CrosstermBackend};
    use std::io;

    if enable_raw_mode().is_err() {
        eprintln!("Error: Tournament mode requires an interactive terminal.");
        eprintln!("Please run from a terminal that supports TUI applications.");
        return;
    }

    let mut stdout = io::stdout();
    if execute!(stdout, EnterAlternateScreen, EnableMouseCapture).is_err() {
        let _ = disable_raw_mode();
        eprintln!("Error: Failed to initialize terminal.");
        return;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            let _ = disable_raw_mode();
            eprintln!("Error: Failed to create terminal: {}", e);
            return;
        }
    };

    let mut setup = SetupMenu::new();

    let result = run_setup_menu(&mut terminal, &mut setup);
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();

    if result.is_err() {
        return;
    }

    let config = setup.get_config();
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            let _ = disable_raw_mode();
            eprintln!("Error: Failed to create terminal: {}", e);
            return;
        }
    };

    let mut app = TournamentApp::new(config.2, Some(config.0), Some(config.1));
    app.search_depth = config.3;
    app.time_control = config.4;
    app.tournament_format = config.5;
    app.file_path = config.6.clone();
    app.game_mode = config.7;
    let mode = config.7;

    let app = Arc::new(Mutex::new(app));
    let app_clone = Arc::clone(&app);

    // Engine/Simulation thread
    thread::spawn(move || {
        loop {
            let (is_running, speed, should_quit, board, game_mode) = {
                let app = app_clone.lock().unwrap();
                (app.is_running(), app.speed, app.should_quit, app.board.clone(), app.game_mode)
            };

            if should_quit { break; }

            if is_running {
                let current_side = board.gamestate.active_side;
                let is_engine_turn = match game_mode {
                    crate::tui::setup::GameMode::EngineVsEngine => true,
                    crate::tui::setup::GameMode::PlayerVsEngine => current_side == 1,
                    _ => false,
                };

                if is_engine_turn {
                    // Expensive move calculation outside the lock
                    let status = board.get_status();
                    if status != crate::defs::GameStatus::Ongoing {
                        let mut app = app_clone.lock().unwrap();
                        // Re-check status inside lock to avoid race conditions
                        let status = app.board.get_status();
                        if status != crate::defs::GameStatus::Ongoing {
                            let result = match status {
                                crate::defs::GameStatus::Checkmate => {
                                    if app.board.gamestate.active_side == 0 { "0-1" } else { "1-0" }
                                }
                                _ => "1/2-1/2",
                            };
                            app.end_game(result);
                            if app.games_played < app.total_games {
                                app.start_new_game();
                            } else {
                                app.finish_tournament();
                                app.view_results();
                            }
                        }
                    } else {
                        let legal_moves = crate::movegen::generate_legal_moves(&board);
                        if !legal_moves.is_empty() {
                            let mv = &legal_moves[fastrand::usize(..legal_moves.len())];
                            let move_str = crate::utils::display::format_move(mv);
                            
                            let mut app = app_clone.lock().unwrap();
                            // Apply move if the board hasn't changed (e.g. by reset)
                            if app.board.gamestate.fullmovenumber == board.gamestate.fullmovenumber && 
                               app.board.gamestate.active_side == board.gamestate.active_side {
                                app.board.make_move(mv);
                                app.current_game_moves.push(move_str);
                                app.update_legal_moves();
                            }
                        }
                    }
                }
            }
            
            // Sleep to let UI thread breathe, minimum 1ms to prevent lock starvation
            let sleep_time = if speed == 0 { 1 } else { speed as u64 };
            thread::sleep(Duration::from_millis(sleep_time));
        }
    });

    let result = run_tournament_logic(&mut terminal, &app);
}

fn run_setup_menu<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    setup: &mut SetupMenu,
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::{self, KeyCode};

    loop {
        terminal.draw(|f| draw_setup(f, setup))?;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        setup.prev_field();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        setup.next_field();
                    }
                    KeyCode::Enter => {
                        if setup.current_field == crate::tui::setup::SetupField::Start {
                            if setup.is_ready_to_start() {
                                break;
                            }
                        } else {
                            setup.toggle_edit();
                        }
                    }
                    KeyCode::Esc => {
                        return Err("Setup cancelled".into());
                    }
                    KeyCode::Char(c) => {
                        setup.handle_char(c);
                    }
                    KeyCode::Backspace => {
                        setup.handle_backspace();
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn run_tournament_logic<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &Arc<Mutex<TournamentApp>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::{self, KeyCode};
    use std::time::Duration;

    let mut last_draw = Instant::now();

    loop {
        let (state, should_quit) = {
            let app = app.lock().unwrap();
            (app.state, app.should_quit)
        };

        if should_quit {
            break;
        }

        // Draw at most 60 times per second
        if last_draw.elapsed() >= Duration::from_millis(16) {
            let app = app.lock().unwrap();
            match state {
                TournamentState::Finished | TournamentState::ViewingResult => {
                    terminal.draw(|f| draw_results(f, &app, f.area()))?;
                }
                _ => {
                    terminal.draw(|f| draw(f, &app))?;
                }
            }
            last_draw = Instant::now();
        }

        if event::poll(Duration::from_millis(10))? {
            if let event::Event::Key(key) = event::read()? {
                let mut app = app.lock().unwrap();
                match app.state {
                    TournamentState::Finished | TournamentState::ViewingResult => {
                        match key.code {
                            KeyCode::Char('q') => {
                                app.quit();
                                break;
                            }
                            KeyCode::Char('r') => {
                                app.restart_tournament();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.selected_game > 0 {
                                    app.selected_game -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.selected_game < app.game_history.len().saturating_sub(1) {
                                    app.selected_game += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if !app.game_history.is_empty() {
                                    let game_index = app.selected_game;
                                    app.replay_game(game_index);
                                }
                            }
                            KeyCode::Esc => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    TournamentState::Settings => {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('?') => {
                                app.state = TournamentState::Paused;
                            }
                            KeyCode::Tab => {
                                app.next_board_scale();
                            }
                            KeyCode::Char('1') => {
                                app.board_scale = crate::tui::tournament::app::BoardScale::Small;
                            }
                            KeyCode::Char('2') => {
                                app.board_scale = crate::tui::tournament::app::BoardScale::Compact;
                            }
                            KeyCode::Char('3') => {
                                app.board_scale = crate::tui::tournament::app::BoardScale::Extended;
                            }
                            KeyCode::Char('4') => {
                                app.board_scale = crate::tui::tournament::app::BoardScale::Large;
                            }
                            KeyCode::Char('q') => {
                                app.quit();
                                break;
                            }
                            _ => {}
                        }
                    }
                    TournamentState::ReplayGame => {
                        match key.code {
                            KeyCode::Char('q') => {
                                app.quit();
                                break;
                            }
                            KeyCode::Esc => {
                                app.view_results();
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                app.replay_next_move();
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                app.replay_prev_move();
                            }
                            KeyCode::Char(' ') => {
                                app.view_results();
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        match key.code {
                            KeyCode::Char('q') => {
                                app.quit();
                                break;
                            }
                            KeyCode::Char(' ') => {
                                if app.game_mode == crate::tui::setup::GameMode::EngineVsEngine {
                                    app.toggle_pause();
                                } else {
                                    app.handle_select();
                                }
                            }
                            KeyCode::Enter => {
                                app.handle_select();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.move_cursor(0, 1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.move_cursor(0, -1);
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                app.move_cursor(-1, 0);
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                app.move_cursor(1, 0);
                            }
                            KeyCode::Char('r') => {
                                app.reset();
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                if app.speed <= 100 {
                                    app.speed = app.speed.saturating_sub(10);
                                } else {
                                    app.speed = app.speed.saturating_sub(100);
                                }
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                if app.speed < 100 {
                                    app.speed = app.speed.saturating_add(10).min(2000);
                                } else {
                                    app.speed = app.speed.saturating_add(100).min(2000);
                                }
                            }
                            KeyCode::Char('?') => {
                                app.state = TournamentState::Settings;
                            }
                            KeyCode::Esc => {
                                if app.games_played >= app.total_games {
                                    app.finish_tournament();
                                    app.view_results();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(())
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
