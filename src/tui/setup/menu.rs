use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameMode {
    PlayerVsEngine,
    PlayerVsPlayer,
    EngineVsEngine,
    Replay,
}

impl GameMode {
    pub fn name(&self) -> &str {
        match self {
            GameMode::PlayerVsEngine => "Player vs Engine",
            GameMode::PlayerVsPlayer => "Player vs Player",
            GameMode::EngineVsEngine => "Engine vs Engine",
            GameMode::Replay => "Replay from File",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SetupField {
    Mode,
    Engine1Path,
    Engine2Path,
    NumGames,
    SearchDepth,
    TimeControl,
    TournamentFormat,
    FilePath,
    Start,
}

pub struct SetupMenu {
    pub mode: GameMode,
    pub engine1_path: String,
    pub engine2_path: String,
    pub num_games: String,
    pub search_depth: String,
    pub time_control: String,
    pub tournament_format: String,
    pub file_path: String,
    pub current_field: SetupField,
    pub editing: bool,
    pub available_releases: Vec<String>,
}

impl SetupMenu {
    pub fn new() -> Self {
        let mut available_releases = Vec::new();
        if let Ok(entries) = std::fs::read_dir("./versions") {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(name) = entry.file_name().to_str() {
                                available_releases.push(format!("./versions/{}", name));
                            }
                        }
                    }
                }
            }
        }
        available_releases.sort();

        let engine1_path = available_releases
            .first()
            .cloned()
            .unwrap_or_else(|| "./target/release/chess".to_string());
        let engine2_path = available_releases
            .get(1)
            .or(available_releases.first())
            .cloned()
            .unwrap_or_else(|| "./target/release/chess".to_string());

        Self {
            mode: GameMode::EngineVsEngine,
            engine1_path,
            engine2_path,
            num_games: "10".to_string(),
            search_depth: "6".to_string(),
            time_control: "blitz".to_string(),
            tournament_format: "gauntlet".to_string(),
            file_path: "game.pgn".to_string(),
            current_field: SetupField::Mode,
            editing: false,
            available_releases,
        }
    }

    pub fn cycle_release(&mut self, forward: bool) {
        if self.available_releases.is_empty() {
            return;
        }

        let current_path = match self.current_field {
            SetupField::Engine1Path => &self.engine1_path,
            SetupField::Engine2Path => &self.engine2_path,
            _ => return,
        };

        let current_idx = self
            .available_releases
            .iter()
            .position(|r| r == current_path);

        let next_idx = match current_idx {
            Some(idx) => {
                if forward {
                    (idx + 1) % self.available_releases.len()
                } else {
                    (idx + self.available_releases.len() - 1) % self.available_releases.len()
                }
            }
            None => 0,
        };

        match self.current_field {
            SetupField::Engine1Path => {
                self.engine1_path = self.available_releases[next_idx].clone()
            }
            SetupField::Engine2Path => {
                self.engine2_path = self.available_releases[next_idx].clone()
            }
            _ => {}
        }
    }

    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            SetupField::Mode => match self.mode {
                GameMode::Replay => SetupField::FilePath,
                GameMode::PlayerVsPlayer => SetupField::Start,
                GameMode::PlayerVsEngine => SetupField::Engine1Path,
                GameMode::EngineVsEngine => SetupField::Engine1Path,
            },
            SetupField::Engine1Path => match self.mode {
                GameMode::EngineVsEngine => SetupField::Engine2Path,
                _ => SetupField::SearchDepth,
            },
            SetupField::Engine2Path => SetupField::NumGames,
            SetupField::NumGames => SetupField::SearchDepth,
            SetupField::SearchDepth => SetupField::TimeControl,
            SetupField::TimeControl => SetupField::TournamentFormat,
            SetupField::TournamentFormat => SetupField::Start,
            SetupField::FilePath => SetupField::Start,
            SetupField::Start => SetupField::Start,
        };
        self.editing = false;
    }

    pub fn prev_field(&mut self) {
        self.current_field = match self.current_field {
            SetupField::Mode => SetupField::Mode,
            SetupField::Engine1Path => SetupField::Mode,
            SetupField::Engine2Path => SetupField::Engine1Path,
            SetupField::NumGames => SetupField::Engine2Path,
            SetupField::SearchDepth => match self.mode {
                GameMode::PlayerVsEngine => SetupField::Engine1Path,
                GameMode::EngineVsEngine => SetupField::NumGames,
                _ => SetupField::Mode,
            },
            SetupField::TimeControl => SetupField::SearchDepth,
            SetupField::TournamentFormat => SetupField::TimeControl,
            SetupField::FilePath => SetupField::Mode,
            SetupField::Start => match self.mode {
                GameMode::Replay => SetupField::FilePath,
                GameMode::PlayerVsPlayer => SetupField::Mode,
                _ => SetupField::TournamentFormat,
            },
        };
        self.editing = false;
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.editing {
            match self.current_field {
                SetupField::Mode => {
                    if c == ' ' || c == '\n' || c == '\r' {
                        // handled by toggle_edit/enter
                    } else {
                        // cycle mode
                        self.mode = match self.mode {
                            GameMode::PlayerVsEngine => GameMode::PlayerVsPlayer,
                            GameMode::PlayerVsPlayer => GameMode::EngineVsEngine,
                            GameMode::EngineVsEngine => GameMode::Replay,
                            GameMode::Replay => GameMode::PlayerVsEngine,
                        };
                    }
                }
                _ => {}
            }
            return;
        }

        match self.current_field {
            SetupField::Engine1Path => self.engine1_path.push(c),
            SetupField::Engine2Path => self.engine2_path.push(c),
            SetupField::NumGames => {
                if c.is_ascii_digit() && self.num_games.len() < 4 {
                    self.num_games.push(c);
                }
            }
            SetupField::SearchDepth => {
                if c.is_ascii_digit() && self.search_depth.len() < 2 {
                    self.search_depth.push(c);
                }
            }
            SetupField::TimeControl => self.time_control.push(c),
            SetupField::TournamentFormat => self.tournament_format.push(c),
            SetupField::FilePath => self.file_path.push(c),
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        if !self.editing {
            return;
        }

        match self.current_field {
            SetupField::Engine1Path => {
                self.engine1_path.pop();
            }
            SetupField::Engine2Path => {
                self.engine2_path.pop();
            }
            SetupField::NumGames => {
                self.num_games.pop();
            }
            SetupField::SearchDepth => {
                self.search_depth.pop();
            }
            SetupField::TimeControl => {
                self.time_control.pop();
            }
            SetupField::TournamentFormat => {
                self.tournament_format.pop();
            }
            SetupField::FilePath => {
                self.file_path.pop();
            }
            _ => {}
        }
    }

    pub fn toggle_edit(&mut self) {
        match self.current_field {
            SetupField::Start => {}
            SetupField::Mode => {
                self.mode = match self.mode {
                    GameMode::PlayerVsEngine => GameMode::PlayerVsPlayer,
                    GameMode::PlayerVsPlayer => GameMode::EngineVsEngine,
                    GameMode::EngineVsEngine => GameMode::Replay,
                    GameMode::Replay => GameMode::PlayerVsEngine,
                };
            }
            _ => {
                self.editing = !self.editing;
            }
        }
    }

    pub fn is_ready_to_start(&self) -> bool {
        match self.mode {
            GameMode::PlayerVsPlayer => true,
            GameMode::PlayerVsEngine => {
                !self.engine1_path.is_empty() && self.search_depth.parse::<u8>().is_ok()
            }
            GameMode::EngineVsEngine => {
                !self.engine1_path.is_empty()
                    && !self.engine2_path.is_empty()
                    && self.num_games.parse::<usize>().is_ok()
                    && self.search_depth.parse::<u8>().is_ok()
            }
            GameMode::Replay => !self.file_path.is_empty(),
        }
    }

    pub fn get_config(&self) -> (String, String, usize, u8, String, String, String, GameMode) {
        (
            self.engine1_path.clone(),
            self.engine2_path.clone(),
            self.num_games.parse().unwrap_or(10),
            self.search_depth.parse().unwrap_or(6),
            self.time_control.clone(),
            self.tournament_format.clone(),
            self.file_path.clone(),
            self.mode,
        )
    }
}

pub fn draw(f: &mut Frame, setup: &SetupMenu) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Fill(1),   // Form
            Constraint::Length(3), // Footer
        ])
        .margin(1)
        .split(f.area());

    draw_title(f, chunks[0]);
    draw_form(f, setup, chunks[1]);
    draw_footer(f, chunks[2]);
}

fn draw_title(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" 󱓟 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            "CHESS ENGINE SETUP",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 󱓟 ", Style::default().fg(Color::Yellow)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_form(f: &mut Frame, setup: &SetupMenu, area: ratatui::layout::Rect) {
    let mut fields = vec![draw_field(
        "Game Mode:",
        setup.mode.name(),
        SetupField::Mode,
        setup,
    )];

    match setup.mode {
        GameMode::PlayerVsPlayer => {}
        GameMode::PlayerVsEngine => {
            fields.push(draw_field(
                "Engine Path:",
                &setup.engine1_path,
                SetupField::Engine1Path,
                setup,
            ));
            fields.push(draw_field(
                "Search Depth:",
                &setup.search_depth,
                SetupField::SearchDepth,
                setup,
            ));
        }
        GameMode::EngineVsEngine => {
            fields.push(draw_field(
                "Engine 1 (White):",
                &setup.engine1_path,
                SetupField::Engine1Path,
                setup,
            ));
            fields.push(draw_field(
                "Engine 2 (Black):",
                &setup.engine2_path,
                SetupField::Engine2Path,
                setup,
            ));
            fields.push(draw_field(
                "Number of Games:",
                &setup.num_games,
                SetupField::NumGames,
                setup,
            ));
            fields.push(draw_field(
                "Search Depth:",
                &setup.search_depth,
                SetupField::SearchDepth,
                setup,
            ));
            fields.push(draw_field(
                "Time Control:",
                &setup.time_control,
                SetupField::TimeControl,
                setup,
            ));
            fields.push(draw_field(
                "Format:",
                &setup.tournament_format,
                SetupField::TournamentFormat,
                setup,
            ));
        }
        GameMode::Replay => {
            fields.push(draw_field(
                "File Path:",
                &setup.file_path,
                SetupField::FilePath,
                setup,
            ));
        }
    }

    fields.push(Line::from(""));
    fields.push(draw_start_button(setup));

    let form = Paragraph::new(fields)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Configuration ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(form, area);
}

fn draw_field(
    label: &str,
    value: &str,
    field_type: SetupField,
    setup: &SetupMenu,
) -> Line<'static> {
    let is_selected = setup.current_field == field_type;
    let is_editing = setup.editing && is_selected;

    let marker = if is_selected { "  " } else { "   " };
    let cursor = if is_editing { "█" } else { "" };

    let style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let value_style = if is_editing {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut spans = vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::styled(format!("{:20}", label), style),
        Span::styled(format!(" {} ", value), value_style),
        Span::styled(cursor, Style::default().fg(Color::Green)),
    ];

    if is_selected
        && !setup.editing
        && (field_type == SetupField::Engine1Path || field_type == SetupField::Engine2Path)
    {
        if !setup.available_releases.is_empty() {
            spans.push(Span::styled(
                "  (Use ←/→ to cycle releases)",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ));
        }
    }

    Line::from(spans)
}

fn draw_start_button(setup: &SetupMenu) -> Line<'static> {
    let is_selected = setup.current_field == SetupField::Start;
    let can_start = setup.is_ready_to_start();

    let marker = if is_selected { "  " } else { "   " };
    let (button_text, style) = if can_start {
        if is_selected {
            (
                " START SESSION ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                " START SESSION ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        }
    } else {
        (" [Missing Info] ", Style::default().fg(Color::DarkGray))
    };

    Line::from(vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::styled(button_text, style),
    ])
}

fn draw_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let footer_text = vec![Line::from(vec![
        Span::styled(
            " ↑/↓ ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Navigate  "),
        Span::styled(
            " ←/→ ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Cycle Releases  "),
        Span::styled(
            " Enter ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Toggle Edit / Cycle Mode  "),
        Span::styled(
            " Esc ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Exit "),
    ])];

    let footer = Paragraph::new(footer_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
