use crate::tui::tournament::app::{TournamentApp, GameRecord};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    draw_title(f, chunks[0]);
    draw_results(f, app, chunks[1]);
    draw_footer(f, chunks[2]);
}

fn draw_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("TOURNAMENT RESULTS")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_results(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_graph(f, app, main_chunks[0]);
    draw_game_list(f, app, main_chunks[1]);
}

fn draw_graph(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let total_games = app.games_played;
    let max_width = 30;
    let white_width = if total_games > 0 {
        (app.white_wins as f64 / total_games as f64 * max_width as f64) as usize
    } else {
        0
    };
    let black_width = if total_games > 0 {
        (app.black_wins as f64 / total_games as f64 * max_width as f64) as usize
    } else {
        0
    };
    let draw_width = if total_games > 0 {
        (app.draws as f64 / total_games as f64 * max_width as f64) as usize
    } else {
        0
    };

    let white_text = format!(" ({}/{})", app.white_wins, total_games);
    let black_text = format!(" ({}/{})", app.black_wins, total_games);
    let draw_text = format!(" ({}/{})", app.draws, total_games);
    let total_text = format!("{}", total_games);
    let win_rate_text = format!("{:.1}%", (app.white_wins as f64 / total_games as f64 * 100.0));

    let graph_lines = vec![
        Line::from(vec![
            Span::styled("White: ", Style::default().fg(Color::White)),
            Span::styled("█".repeat(white_width), Style::default().fg(Color::Green)),
            Span::raw(white_text),
        ]),
        Line::from(vec![
            Span::styled("Black: ", Style::default().fg(Color::White)),
            Span::styled("█".repeat(black_width), Style::default().fg(Color::Blue)),
            Span::raw(black_text),
        ]),
        Line::from(vec![
            Span::styled("Draws:  ", Style::default().fg(Color::White)),
            Span::styled("█".repeat(draw_width), Style::default().fg(Color::Gray)),
            Span::raw(draw_text),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Games: ", Style::default().fg(Color::White)),
            Span::styled(
                total_text,
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Win Rate (White): ", Style::default().fg(Color::White)),
            Span::styled(
                win_rate_text,
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let graph = Paragraph::new(graph_lines)
        .block(Block::default().borders(Borders::ALL).title("Results Graph"))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);

    f.render_widget(graph, area);
}

fn draw_game_list(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let game_lines: Vec<Line> = app
        .game_history
        .iter()
        .enumerate()
        .map(|(i, game)| {
            let result_color = match game.result.as_str() {
                "1-0" => Color::Green,
                "0-1" => Color::Blue,
                "1/2-1/2" => Color::Gray,
                _ => Color::White,
            };

            let is_selected = i == app.selected_game;
            let marker = if is_selected { "> " } else { "  " };

            Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("Game {}: ", i + 1),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    &game.result,
                    Style::default().fg(result_color),
                ),
                Span::raw(format!(" ({} moves)", game.moves.len())),
            ])
        })
        .collect();

    let list_text = if game_lines.is_empty() {
        vec![Line::from("No games played yet")]
    } else {
        game_lines
    };

    let list = Paragraph::new(list_text)
        .block(Block::default().borders(Borders::ALL).title("Games (Press Enter to replay)"))
        .wrap(Wrap { trim: false });

    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Select Game  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Replay  "),
            Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Restart  "),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Back  "),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
