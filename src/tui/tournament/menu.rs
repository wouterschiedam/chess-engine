use super::{TournamentApp, TournamentState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &TournamentApp, area: ratatui::layout::Rect) {
    let stats_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(8)])
        .split(area);

    draw_move_list(f, app, stats_chunks[0]);
    draw_tournament_stats(f, app, stats_chunks[1]);
}

fn draw_move_list(f: &mut Frame, app: &TournamentApp, area: ratatui::layout::Rect) {
    let (moves, title) = match app.state {
        TournamentState::ReplayGame => {
            if let Some(game) = app.game_history.get(app.selected_game) {
                let move_lines: Vec<Line> = game
                    .moves
                    .iter()
                    .enumerate()
                    .map(|(i, mv)| {
                        let is_current = i == app.replay_move_index;
                        let is_past = i < app.replay_move_index;
                        let prefix = if is_current { " " } else { "  " };
                        let line = format!("{}{:3}. {}", prefix, (i / 2) + 1, mv);
                        Line::styled(
                            line,
                            if is_current {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else if is_past {
                                Style::default().fg(Color::DarkGray)
                            } else {
                                Style::default().fg(Color::White)
                            }
                        )
                    })
                    .collect();
                (move_lines, format!(" REPLAY: GAME {} ", app.selected_game + 1))
            } else {
                (vec![Line::from(" No game selected")], " MOVE LIST ".to_string())
            }
        }
        _ => {
            let move_lines: Vec<Line> = app
                .current_game_moves
                .chunks(2)
                .enumerate()
                .map(|(i, pair)| {
                    let move_str = if pair.len() == 2 {
                        format!(" {:3}. {:7} {:7}", i + 1, pair[0], pair[1])
                    } else {
                        format!(" {:3}. {:7}", i + 1, pair[0])
                    };
                    Line::from(move_str)
                })
                .collect();
            (move_lines, " MOVE LIST ".to_string())
        }
    };

    let moves_text = if moves.is_empty() {
        ratatui::text::Text::from("\n  No moves yet...")
    } else {
        ratatui::text::Text::from(moves)
    };

    let moves_paragraph = Paragraph::new(moves_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)))
        .scroll((app.moves_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(moves_paragraph, area);
}

fn draw_tournament_stats(f: &mut Frame, app: &TournamentApp, area: ratatui::layout::Rect) {
    let stats_text = vec![
        Line::from(vec![
            Span::styled(" Games: ", Style::default().fg(Color::White)),
            Span::styled(format!("{} / {}", app.games_played, app.total_games), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(" White: ", Style::default().fg(Color::White)),
            Span::styled(format!("{} wins", app.white_wins), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled(" Black: ", Style::default().fg(Color::White)),
            Span::styled(format!("{} wins", app.black_wins), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled(" Draws: ", Style::default().fg(Color::White)),
            Span::styled(format!("{}", app.draws), Style::default().fg(Color::Gray)),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" TOURNAMENT STATS ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(stats, area);
}
