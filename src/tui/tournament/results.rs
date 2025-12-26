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
            Constraint::Length(3), // Title
            Constraint::Fill(1),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    draw_title(f, chunks[0]);
    draw_results_dashboard(f, app, chunks[1]);
    draw_footer(f, chunks[2]);
}

fn draw_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" 󱓟 ", Style::default().fg(Color::Yellow)),
        Span::styled("TOURNAMENT SUMMARY", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" 󱓟 ", Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray)))
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn draw_results_dashboard(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let dashboard_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Quick Stats
            Constraint::Fill(1),    // Bar Graph
        ])
        .split(dashboard_chunks[0]);

    draw_quick_stats(f, app, left_chunks[0]);
    draw_graph(f, app, left_chunks[1]);
    draw_game_list(f, app, dashboard_chunks[1]);
}

fn draw_quick_stats(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let total = app.games_played.max(1);
    let win_rate = app.white_wins as f64 / total as f64 * 100.0;
    let black_win_rate = app.black_wins as f64 / total as f64 * 100.0;
    let draw_rate = app.draws as f64 / total as f64 * 100.0;

    let stats_text = vec![
        Line::from(vec![
            Span::styled(" Total Games: ", Style::default().fg(Color::White)),
            Span::styled(format!("{}", app.games_played), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" White Wins:  ", Style::default().fg(Color::White)),
            Span::styled(format!("{} ", app.white_wins), Style::default().fg(Color::Green)),
            Span::styled(format!("({:.1}%)", win_rate), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" Black Wins:  ", Style::default().fg(Color::White)),
            Span::styled(format!("{} ", app.black_wins), Style::default().fg(Color::Blue)),
            Span::styled(format!("({:.1}%)", black_win_rate), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" Draws:       ", Style::default().fg(Color::White)),
            Span::styled(format!("{} ", app.draws), Style::default().fg(Color::Gray)),
            Span::styled(format!("({:.1}%)", draw_rate), Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(" Quick Stats ").border_style(Style::default().fg(Color::DarkGray)));

    f.render_widget(stats, area);
}

fn draw_graph(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let total_games = app.games_played.max(1);
    let available_width = area.width.saturating_sub(15) as f64;
    
    let white_width = (app.white_wins as f64 / total_games as f64 * available_width) as usize;
    let black_width = (app.black_wins as f64 / total_games as f64 * available_width) as usize;
    let draw_width = (app.draws as f64 / total_games as f64 * available_width) as usize;

    let graph_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" W ", Style::default().fg(Color::Black).bg(Color::Green)),
            Span::raw(" "),
            Span::styled("█".repeat(white_width), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" B ", Style::default().fg(Color::Black).bg(Color::Blue)),
            Span::raw(" "),
            Span::styled("█".repeat(black_width), Style::default().fg(Color::Blue)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" D ", Style::default().fg(Color::Black).bg(Color::Gray)),
            Span::raw(" "),
            Span::styled("█".repeat(draw_width), Style::default().fg(Color::Gray)),
        ]),
    ];

    let graph = Paragraph::new(graph_lines)
        .block(Block::default().borders(Borders::ALL).title(" Win Distribution ").border_style(Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Left);

    f.render_widget(graph, area);
}

fn draw_game_list(f: &mut Frame, app: &TournamentApp, area: Rect) {
    let game_lines: Vec<Line> = app
        .game_history
        .iter()
        .enumerate()
        .map(|(i, game)| {
            let (result_text, result_color) = match game.result.as_str() {
                "1-0" => (" WIN ", Color::Green),
                "0-1" => (" LOSS", Color::Blue),
                "1/2-1/2" => (" DRAW", Color::Gray),
                _ => (" ????", Color::White),
            };

            let is_selected = i == app.selected_game;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(vec![
                Span::styled(format!(" {:02} ", i + 1), style),
                Span::raw(" "),
                Span::styled(result_text, Style::default().fg(result_color).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" moves: {:3}  ", game.moves.len())),
                Span::styled(&game.white_engine, Style::default().fg(Color::DarkGray)),
                Span::raw(" vs "),
                Span::styled(&game.black_engine, Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let list_text = if game_lines.is_empty() {
        vec![Line::from(" No games recorded yet.")]
    } else {
        game_lines
    };

    let list = Paragraph::new(list_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Game History ")
            .border_style(Style::default().fg(Color::DarkGray)))
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
