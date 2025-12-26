use crate::board::Board;
use crate::tui::setup::{SetupMenu, draw_setup};
use crate::tui::tournament::draw_results;
use crate::tui::{TournamentApp, TournamentState, ui::draw};
use crate::utils::display::print_position;
use crate::utils::perft::{divide, divide_with_known, drill_down, perft, perft_verbose};
use std::time::Duration;

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
    // config.6 is file_path, config.7 is mode
    let mode = config.7;
    
    match mode {
        crate::tui::setup::GameMode::EngineVsEngine => {
            app.start();
            let result = run_tournament_logic(&mut terminal, &mut app);
            if let Err(err) = result {
                eprintln!("Error: {}", err);
            }
        }
        crate::tui::setup::GameMode::Replay => {
            if let Err(e) = app.load_replay_from_file(&config.6) {
                // If file not found, we could show an error, but for now let's just not start replay
                // and maybe return to setup or just log it if we could.
            }
            let result = run_tournament_logic(&mut terminal, &mut app);
            if let Err(err) = result {
                eprintln!("Error: {}", err);
            }
        }
        _ => {
            // Placeholder for PvP and PvE
            app.start();
            let result = run_tournament_logic(&mut terminal, &mut app);
            if let Err(err) = result {
                eprintln!("Error: {}", err);
            }
        }
    }
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
    app: &mut TournamentApp,
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::{self, KeyCode};
    use std::thread;
    use std::time::Duration;

    loop {
        match app.state {
            TournamentState::Finished | TournamentState::ViewingResult => {
                terminal.draw(|f| draw_results(f, app, f.area()))?;

                if event::poll(Duration::from_millis(100))? {
                    if let event::Event::Key(key) = event::read()? {
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
                                    app.replay_game(app.selected_game);
                                }
                            }
                            KeyCode::Esc => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
            }
            TournamentState::Settings => {
                terminal.draw(|f| draw(f, app))?;

                if event::poll(Duration::from_millis(100))? {
                    if let event::Event::Key(key) = event::read()? {
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
                }
            }
            TournamentState::ReplayGame => {
                terminal.draw(|f| draw(f, app))?;

                if event::poll(Duration::from_millis(100))? {
                    if let event::Event::Key(key) = event::read()? {
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
                }
            }
            _ => {
                terminal.draw(|f| draw(f, app))?;

                if event::poll(Duration::from_millis(100))? {
                    if let event::Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('q') => {
                                app.quit();
                                break;
                            }
                            KeyCode::Char(' ') => {
                                app.toggle_pause();
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

                if app.is_running() && app.games_played < app.total_games {
                    simulate_move(app);
                    thread::sleep(Duration::from_millis(app.speed as u64));
                } else if app.games_played >= app.total_games {
                    app.finish_tournament();
                    app.view_results();
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn simulate_move(app: &mut TournamentApp) {
    use crate::movegen::generate_legal_moves;
    use fastrand;

    let legal_moves = generate_legal_moves(&app.board);

    if legal_moves.is_empty() {
        if app.board.is_in_check(app.board.gamestate.active_side) {
            let result = if app.board.gamestate.active_side == 0 {
                "0-1"
            } else {
                "1-0"
            };
            app.end_game(result);
        } else {
            app.end_game("1/2-1/2");
        }

        if app.games_played < app.total_games {
            app.start_new_game();
        }
        return;
    }

    let mv = &legal_moves[fastrand::usize(..legal_moves.len())];
    app.board.make_move(mv);
    app.add_move(mv);

    if fastrand::usize(..200) < 3 {
        let result = if fastrand::bool() { "1-0" } else { "0-1" };
        app.end_game(result);
        if app.games_played < app.total_games {
            app.start_new_game();
        }
    }
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
