use crate::board::Board;
use crate::tui::board::BoardWidget;
use ratatui::{
    Frame,
    layout::Rect,
};

pub fn draw(f: &mut Frame, app: &crate::tui::TournamentApp, area: Rect) {
    let board_widget = BoardWidget::new(app);
    f.render_widget(board_widget, area);
}
