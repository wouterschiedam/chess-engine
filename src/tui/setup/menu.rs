use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Clone, Copy, PartialEq)]
pub enum SetupField {
    Engine1Path,
    Engine2Path,
    NumGames,
    SearchDepth,
    TimeControl,
    TournamentFormat,
    Start,
}

pub struct SetupMenu {
    pub engine1_path: String,
    pub engine2_path: String,
    pub num_games: String,
    pub search_depth: String,
    pub time_control: String,
    pub tournament_format: String,
    pub current_field: SetupField,
    pub editing: bool,
}

impl SetupMenu {
    pub fn new() -> Self {
        Self {
            engine1_path: "./target/release/chess".to_string(),
            engine2_path: "./target/release/chess".to_string(),
            num_games: "10".to_string(),
            search_depth: "6".to_string(),
            time_control: "blitz".to_string(),
            tournament_format: "gauntlet".to_string(),
            current_field: SetupField::Engine1Path,
            editing: false,
        }
    }

    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            SetupField::Engine1Path => SetupField::Engine2Path,
            SetupField::Engine2Path => SetupField::NumGames,
            SetupField::NumGames => SetupField::SearchDepth,
            SetupField::SearchDepth => SetupField::TimeControl,
            SetupField::TimeControl => SetupField::TournamentFormat,
            SetupField::TournamentFormat => SetupField::Start,
            SetupField::Start => SetupField::Start,
        };
        self.editing = false;
    }

    pub fn prev_field(&mut self) {
        self.current_field = match self.current_field {
            SetupField::Engine1Path => SetupField::Engine1Path,
            SetupField::Engine2Path => SetupField::Engine1Path,
            SetupField::NumGames => SetupField::Engine2Path,
            SetupField::SearchDepth => SetupField::NumGames,
            SetupField::TimeControl => SetupField::SearchDepth,
            SetupField::TournamentFormat => SetupField::TimeControl,
            SetupField::Start => SetupField::TournamentFormat,
        };
        self.editing = false;
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.editing {
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
            SetupField::Start => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        if !self.editing {
            return;
        }

        match self.current_field {
            SetupField::Engine1Path => { self.engine1_path.pop(); }
            SetupField::Engine2Path => { self.engine2_path.pop(); }
            SetupField::NumGames => { self.num_games.pop(); }
            SetupField::SearchDepth => { self.search_depth.pop(); }
            SetupField::TimeControl => { self.time_control.pop(); }
            SetupField::TournamentFormat => { self.tournament_format.pop(); }
            SetupField::Start => {}
        }
    }

    pub fn toggle_edit(&mut self) {
        if self.current_field != SetupField::Start {
            self.editing = !self.editing;
        }
    }

    pub fn is_ready_to_start(&self) -> bool {
        !self.engine1_path.is_empty()
            && !self.engine2_path.is_empty()
            && self.num_games.parse::<usize>().is_ok()
            && self.search_depth.parse::<u8>().is_ok()
    }

    pub fn get_config(&self) -> (String, String, usize, u8, String, String) {
        (
            self.engine1_path.clone(),
            self.engine2_path.clone(),
            self.num_games.parse().unwrap_or(10),
            self.search_depth.parse().unwrap_or(6),
            self.time_control.clone(),
            self.tournament_format.clone(),
        )
    }
}

pub fn draw(f: &mut Frame, setup: &SetupMenu) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .margin(2)
        .split(f.area());

    draw_title(f, chunks[0]);
    draw_form(f, setup, chunks[1]);
    draw_footer(f, chunks[2]);
}

fn draw_title(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new("CHESS TOURNAMENT SETUP")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_form(f: &mut Frame, setup: &SetupMenu, area: ratatui::layout::Rect) {
    let fields = vec![
        draw_field(
            "Engine 1 Path (White in odd games):",
            &setup.engine1_path,
            SetupField::Engine1Path,
            setup,
        ),
        draw_field(
            "Engine 2 Path (Black in odd games):",
            &setup.engine2_path,
            SetupField::Engine2Path,
            setup,
        ),
        draw_field(
            "Number of Games:",
            &setup.num_games,
            SetupField::NumGames,
            setup,
        ),
        draw_field(
            "Search Depth:",
            &setup.search_depth,
            SetupField::SearchDepth,
            setup,
        ),
        draw_field(
            "Time Control (bullet/blitz/rapid/custom):",
            &setup.time_control,
            SetupField::TimeControl,
            setup,
        ),
        draw_field(
            "Format (gauntlet/round-robin):",
            &setup.tournament_format,
            SetupField::TournamentFormat,
            setup,
        ),
        draw_start_button(setup),
    ];

    let form = Paragraph::new(fields)
        .block(Block::default().borders(Borders::ALL))
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

    let marker = if is_selected { "> " } else { "  " };
    let cursor = if is_editing { "█" } else { "" };
    let value_display = if is_editing {
        format!("{}{}", value, cursor)
    } else {
        value.to_string()
    };

    Line::from(vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::styled(label.to_string(), Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled(
            value_display,
            Style::default()
                .fg(if is_editing { Color::Green } else { Color::Cyan }),
        ),
    ])
}

fn draw_start_button(setup: &SetupMenu) -> Line<'static> {
    let is_selected = setup.current_field == SetupField::Start;
    let can_start = setup.is_ready_to_start();

    let marker = if is_selected { "> " } else { "  " };
    let button_text = if can_start {
        "[START TOURNAMENT]"
    } else {
        "[Fill in all fields]"
    };

    Line::from(vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::styled(
            button_text,
            Style::default()
                .fg(if can_start { Color::Green } else { Color::Gray })
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn draw_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Edit/Start  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Exit  "),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
