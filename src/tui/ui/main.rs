use crate::tui::tournament::{TournamentApp, TournamentState};
use crate::tui::tournament::{draw_board, draw_menu, draw_shortcuts};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(f: &mut Frame, app: &TournamentApp) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(f.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(main_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Min(6),
        ])
        .split(main_chunks[1]);

    draw_board(f, app, left_chunks[0]);
    draw_header(f, app, right_chunks[0]);
    draw_status(f, app, right_chunks[1]);
    draw_menu(f, app, right_chunks[2]);
    draw_shortcuts(f, app, right_chunks[3]);

    if matches!(app.state, TournamentState::Settings) {
        draw_settings_popup(f, app);
    }
}

fn draw_settings_popup(f: &mut Frame, app: &TournamentApp) {
    let area = centered_rect(60, 40, f.area());
    let block = Block::default()
        .title(" Board Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let inner_area = block.inner(area);
    f.render_widget(ratatui::widgets::Clear, area); // Clear the background
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner_area);

    let scales = ["Small", "Compact", "Extended", "Large"];
    let mut scale_spans = Vec::new();
    for (i, scale) in scales.iter().enumerate() {
        let is_selected = app.board_scale == match i {
            0 => crate::tui::tournament::app::BoardScale::Small,
            1 => crate::tui::tournament::app::BoardScale::Compact,
            2 => crate::tui::tournament::app::BoardScale::Extended,
            3 => crate::tui::tournament::app::BoardScale::Large,
            _ => crate::tui::tournament::app::BoardScale::Small,
        };
        
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        scale_spans.push(Span::styled(format!(" [{}] ", scale), style));
    }

    let scale_para = Paragraph::new(Line::from(scale_spans))
        .block(Block::default().title(" Board Scale (Use 1-4 or TAB) ").borders(Borders::NONE))
        .alignment(Alignment::Center);

    f.render_widget(scale_para, chunks[0]);

    let footer = Paragraph::new("Press [ESC] or [?] to close")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_header(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let state_color = match app.state {
        TournamentState::Running => Color::Green,
        TournamentState::Paused => Color::Yellow,
        TournamentState::Stopped => Color::Red,
        TournamentState::Finished => Color::Cyan,
        TournamentState::ViewingResult => Color::Magenta,
        TournamentState::ReplayGame => Color::LightCyan,
        TournamentState::Settings => Color::Yellow,
    };

    let state_text = match app.state {
        TournamentState::Running => "[RUNNING]",
        TournamentState::Paused => "[PAUSED]",
        TournamentState::Stopped => "[STOPPED]",
        TournamentState::Finished => "[FINISHED]",
        TournamentState::ViewingResult => "[RESULTS]",
        TournamentState::ReplayGame => "[REPLAY]",
        TournamentState::Settings => "[SETTINGS]",
    };

    let game_text = match app.state {
        TournamentState::ReplayGame => format!("Replaying Game {}", app.selected_game + 1),
        _ => format!("Game {}/{}", app.games_played + 1, app.total_games),
    };

    let header_text = vec![Line::from(vec![
        Span::styled(
            "CHESS ENGINE TOURNAMENT",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            state_text,
            Style::default()
                .fg(state_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(game_text, Style::default().fg(Color::Yellow)),
    ])];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

fn draw_status(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let status_text = vec![
        Line::from(vec![
            Span::styled("White: ", Style::default().fg(Color::White)),
            Span::styled(&app.white_engine, Style::default().fg(Color::Green)),
            Span::raw("  |  "),
            Span::styled("Black: ", Style::default().fg(Color::White)),
            Span::styled(&app.black_engine, Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("Result: ", Style::default().fg(Color::White)),
            Span::styled(
                &app.game_result,
                if app.game_result == "In Progress" {
                    Style::default().fg(Color::Yellow)
                } else if app.game_result == "1-0" {
                    Style::default().fg(Color::Green)
                } else if app.game_result == "0-1" {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
        ]),
    ];

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(status, area);
}
