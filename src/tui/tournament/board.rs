use crate::board::Board;
use crate::tui::board::BoardWidget;
use ratatui::{
    Frame,
    layout::Rect,
};

pub fn draw(f: &mut Frame, app: &crate::tui::TournamentApp, area: Rect) {
    let board = if matches!(app.state, crate::tui::tournament::app::TournamentState::ReplayGame) {
        &app.replay_board
    } else {
        &app.board
    };
    let board_widget = BoardWidget::new(board, app.board_scale);
    f.render_widget(board_widget, area);
}
