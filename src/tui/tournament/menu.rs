use super::{TournamentApp, TournamentState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &TournamentApp, area: ratatui::layout::Rect) {
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
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
                        let line = if is_current {
                            format!("> {}. {}", (i / 2) + 1, mv)
                        } else {
                            format!("  {}. {}", (i / 2) + 1, mv)
                        };
                        Line::styled(
                            line,
                            if is_current {
                                Style::default().fg(Color::Yellow)
                            } else if is_past {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default().fg(Color::White)
                            }
                        )
                    })
                    .collect();
                (move_lines, format!("Replay: Game {}", app.selected_game + 1))
            } else {
                (vec![Line::from("No game selected")], "Move List".to_string())
            }
        }
        _ => {
            let move_lines: Vec<Line> = app
                .current_game_moves
                .chunks(2)
                .enumerate()
                .map(|(i, pair)| {
                    let move_str = if pair.len() == 2 {
                        format!("{}. {}  {}", i + 1, pair[0], pair[1])
                    } else {
                        format!("{}. {}", i + 1, pair[0])
                    };
                    Line::from(move_str)
                })
                .collect();
            (move_lines, "Move List".to_string())
        }
    };

    let moves_text = if moves.is_empty() {
        ratatui::text::Text::from("No moves yet")
    } else {
        ratatui::text::Text::from(moves)
    };

    let moves = Paragraph::new(moves_text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    f.render_widget(moves, area);
}

fn draw_tournament_stats(f: &mut Frame, app: &TournamentApp, area: ratatui::layout::Rect) {
    let stats_text = vec![
        Line::from(format!("Games Played: {}", app.games_played)),
        Line::from(format!("Games Total:  {}", app.total_games)),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("White Wins: {} ", app.white_wins),
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            format!("Black Wins: {} ", app.black_wins),
            Style::default().fg(Color::Blue),
        )]),
        Line::from(format!("Draws:       {}", app.draws)),
        Line::from(""),
    ];

    let stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tournament Stats"),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(stats, area);
}
