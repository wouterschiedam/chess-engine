use super::{TournamentApp, TournamentState};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &TournamentApp, area: ratatui::layout::Rect) {
    let controls_text = match app.state {
        TournamentState::ReplayGame => {
            let game = app.game_history.get(app.selected_game);
            let total_moves = game.map(|g| g.moves.len()).unwrap_or(0);
            let current_move = app.replay_move_index;

            vec![
                Line::from("Replay Controls:"),
                Line::from(vec![
                    Span::styled("[←/→]", Style::default().fg(Color::Cyan)),
                    Span::raw(format!(" Prev/Next ({}/{})", current_move, total_moves)),
                ]),
                Line::from(vec![
                    Span::styled("[ESC]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Back to Results  "),
                ]),
                Line::from(vec![
                    Span::styled("[SPACE]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Back to Results  "),
                ]),
            ]
        }
        TournamentState::Settings => {
            vec![
                Line::from("Settings Controls:"),
                Line::from(vec![
                    Span::styled("[1-4]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Select Scale "),
                ]),
                Line::from(vec![
                    Span::styled("[TAB]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Cycle Scale "),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[ESC/?]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Close Settings "),
                ]),
            ]
        }
        _ => {
            vec![
                Line::from(vec![
                    Span::styled("[SPACE]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Start/Pause  "),
                ]),
                Line::from(vec![
                    Span::styled("[R]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Reset  "),
                ]),
                Line::from(vec![
                    Span::styled("[Q]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Quit  "),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[+/-]", Style::default().fg(Color::Cyan)),
                    Span::raw(format!(" Speed: {}ms", app.speed)),
                ]),
                Line::from(vec![
                    Span::styled("[?]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Settings "),
                ]),
            ]
        }
    };

    let stats = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true });

    f.render_widget(stats, area);
}
