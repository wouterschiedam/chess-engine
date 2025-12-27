use crate::board::Board;
use crate::tui::setup::{SetupMenu, draw_setup, GameMode};
use crate::tui::tournament::draw_results;
use crate::tui::{TournamentApp, TournamentState, ui::draw};
use crate::cli::commands::engine::UciEngineProcess;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub fn run_tournament() {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{Terminal, backend::CrosstermBackend};
    use std::io;

    if enable_raw_mode().is_err() {
        eprintln!("Error: Tournament mode requires an interactive terminal.");
        return;
    }

    let mut stdout = io::stdout();
    if execute!(stdout, EnterAlternateScreen, EnableMouseCapture).is_err() {
        let _ = disable_raw_mode();
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
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal.show_cursor();

    if result.is_err() { return; }

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

    let engine1_path = config.0.clone();
    let engine2_path = config.1.clone();
    let mode = config.7;

    let mut app = TournamentApp::new(config.2, Some(config.0), Some(config.1));
    app.search_depth = config.3;
    app.time_control = config.4;
    app.tournament_format = config.5;
    app.file_path = config.6.clone();
    app.game_mode = mode;

    let app = Arc::new(Mutex::new(app));
    
    if mode == GameMode::EngineVsEngine {
        app.lock().unwrap().start();
    }

    let app_white = Arc::clone(&app);
    let app_black = Arc::clone(&app);

    // White engine thread
    thread::spawn(move || {
        let mut engine = UciEngineProcess::new(engine1_path);
        loop {
            let (is_running, speed, should_quit, board, game_mode, search_depth) = {
                let app = app_white.lock().unwrap();
                (app.is_running(), app.speed, app.should_quit, app.board.clone(), app.game_mode, app.search_depth)
            };

            if should_quit { break; }

            if is_running && board.gamestate.active_side == 0 && game_mode == GameMode::EngineVsEngine {
                let status = board.get_status();
                if status != crate::defs::GameStatus::Ongoing {
                    let mut app = app_white.lock().unwrap();
                    if app.is_running() && app.board.get_status() != crate::defs::GameStatus::Ongoing {
                        end_game_logic(&mut app);
                    }
                } else if let Some(mv_str) = engine.get_move(&board, search_depth) {
                    let mut app = app_white.lock().unwrap();
                    if app.is_running() && app.board.gamestate.active_side == 0 && app.board.get_status() == crate::defs::GameStatus::Ongoing {
                        if let Some(mv) = app.parse_move_str(&app.board, &mv_str) {
                            app.board.make_move(&mv);
                            app.current_game_moves.push(mv_str);
                            app.update_legal_moves();
                            
                            let status = app.board.get_status();
                            if status != crate::defs::GameStatus::Ongoing {
                                end_game_logic(&mut app);
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(speed.max(1) as u64));
        }
        engine.stop();
    });

    // Black engine thread
    thread::spawn(move || {
        let mut engine = UciEngineProcess::new(engine2_path);
        loop {
            let (is_running, speed, should_quit, board, game_mode, search_depth) = {
                let app = app_black.lock().unwrap();
                (app.is_running(), app.speed, app.should_quit, app.board.clone(), app.game_mode, app.search_depth)
            };

            if should_quit { break; }

            if is_running && board.gamestate.active_side == 1 {
                let is_engine_turn = match game_mode {
                    GameMode::EngineVsEngine => true,
                    GameMode::PlayerVsEngine => true,
                    _ => false,
                };

                if is_engine_turn {
                    let status = board.get_status();
                    if status != crate::defs::GameStatus::Ongoing {
                        let mut app = app_black.lock().unwrap();
                        if app.is_running() && app.board.get_status() != crate::defs::GameStatus::Ongoing {
                            end_game_logic(&mut app);
                        }
                    } else if let Some(mv_str) = engine.get_move(&board, search_depth) {
                        let mut app = app_black.lock().unwrap();
                        if app.is_running() && app.board.gamestate.active_side == 1 && app.board.get_status() == crate::defs::GameStatus::Ongoing {
                            if let Some(mv) = app.parse_move_str(&app.board, &mv_str) {
                                app.board.make_move(&mv);
                                app.current_game_moves.push(mv_str);
                                app.update_legal_moves();

                                let status = app.board.get_status();
                                if status != crate::defs::GameStatus::Ongoing {
                                    end_game_logic(&mut app);
                                }
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(speed.max(1) as u64));
        }
        engine.stop();
    });

    let _ = run_tournament_logic(&mut terminal, &app);
}

fn end_game_logic(app: &mut TournamentApp) {
    let status = app.board.get_status();
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
                    KeyCode::Up | KeyCode::Char('k') => setup.prev_field(),
                    KeyCode::Down | KeyCode::Char('j') => setup.next_field(),
                    KeyCode::Left | KeyCode::Char('h') => setup.cycle_release(false),
                    KeyCode::Right | KeyCode::Char('l') => setup.cycle_release(true),
                    KeyCode::Enter => {
                        if setup.current_field == crate::tui::setup::SetupField::Start {
                            if setup.is_ready_to_start() { break; }
                        } else {
                            setup.toggle_edit();
                        }
                    }
                    KeyCode::Esc => return Err("Setup cancelled".into()),
                    KeyCode::Char(c) => setup.handle_char(c),
                    KeyCode::Backspace => setup.handle_backspace(),
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

    let mut last_draw = Instant::now();

    loop {
        let (state, should_quit) = {
            let app = app.lock().unwrap();
            (app.state, app.should_quit)
        };

        if should_quit { break; }

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
                    TournamentState::Finished | TournamentState::ViewingResult => match key.code {
                        KeyCode::Char('q') => { app.quit(); break; }
                        KeyCode::Char('r') => app.restart_tournament(),
                        KeyCode::Up | KeyCode::Char('k') => if app.selected_game > 0 { app.selected_game -= 1; },
                        KeyCode::Down | KeyCode::Char('j') => if app.selected_game < app.game_history.len().saturating_sub(1) { app.selected_game += 1; },
                        KeyCode::Enter => if !app.game_history.is_empty() { let game_index = app.selected_game; app.replay_game(game_index); },
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    },
                    TournamentState::Settings => match key.code {
                        KeyCode::Esc | KeyCode::Char('?') => app.state = TournamentState::Paused,
                        KeyCode::Tab => app.next_board_scale(),
                        KeyCode::Char('1') => app.board_scale = crate::tui::tournament::app::BoardScale::Small,
                        KeyCode::Char('2') => app.board_scale = crate::tui::tournament::app::BoardScale::Compact,
                        KeyCode::Char('3') => app.board_scale = crate::tui::tournament::app::BoardScale::Extended,
                        KeyCode::Char('4') => app.board_scale = crate::tui::tournament::app::BoardScale::Large,
                        KeyCode::Char('q') => { app.quit(); break; }
                        _ => {}
                    },
                    TournamentState::ReplayGame => match key.code {
                        KeyCode::Char('q') => { app.quit(); break; }
                        KeyCode::Esc => app.view_results(),
                        KeyCode::Right | KeyCode::Char('l') => app.replay_next_move(),
                        KeyCode::Left | KeyCode::Char('h') => app.replay_prev_move(),
                        KeyCode::Char(' ') => app.view_results(),
                        _ => {}
                    },
                    _ => match key.code {
                        KeyCode::Char('q') => { app.quit(); break; }
                        KeyCode::Char(' ') => {
                            if app.game_mode == GameMode::EngineVsEngine {
                                app.toggle_pause();
                            } else {
                                app.handle_select();
                            }
                        }
                        KeyCode::Enter => app.handle_select(),
                        KeyCode::Up | KeyCode::Char('k') => app.move_cursor(0, 1),
                        KeyCode::Down | KeyCode::Char('j') => app.move_cursor(0, -1),
                        KeyCode::Left | KeyCode::Char('h') => app.move_cursor(-1, 0),
                        KeyCode::Right | KeyCode::Char('l') => app.move_cursor(1, 0),
                        KeyCode::Char('r') => app.reset(),
                        KeyCode::Char('+') | KeyCode::Char('=') => if app.speed <= 100 { app.speed = app.speed.saturating_sub(10); } else { app.speed = app.speed.saturating_sub(100); },
                        KeyCode::Char('-') | KeyCode::Char('_') => if app.speed < 100 { app.speed = app.speed.saturating_add(10).min(2000); } else { app.speed = app.speed.saturating_add(100).min(2000); },
                        KeyCode::Char('?') => app.state = TournamentState::Settings,
                        KeyCode::Esc => if app.games_played >= app.total_games { app.finish_tournament(); app.view_results(); }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}
