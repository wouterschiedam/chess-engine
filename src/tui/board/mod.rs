use crate::board::Board;
use crate::board::types::Pieces;
use crate::defs::{NrOf, Side, Sides};
use crate::tui::pieces::{Bishop, King, Knight, Pawn, PieceArt, PieceSize, Queen, Rook};
use crate::tui::tournament::app::BoardScale;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

pub const LIGHT_SQUARE: Color = Color::Rgb(240, 217, 181);
pub const DARK_SQUARE: Color = Color::Rgb(181, 136, 99);
pub const CURSOR_COLOR: Color = Color::Rgb(100, 150, 255);
pub const SELECTED_COLOR: Color = Color::Rgb(255, 255, 100);
pub const MOVE_TARGET_COLOR: Color = Color::Rgb(100, 255, 100);

pub struct BoardWidget<'a> {
    board: &'a Board,
    scale: BoardScale,
    cursor_pos: usize,
    selected_square: Option<usize>,
    show_cursor: bool,
}

impl<'a> BoardWidget<'a> {
    pub fn new(app: &'a crate::tui::tournament::app::TournamentApp) -> Self {
        let board = if matches!(app.state, crate::tui::tournament::app::TournamentState::ReplayGame) {
            &app.replay_board
        } else {
            &app.board
        };

        // Only show cursor in interactive modes or during replay
        let show_cursor = match app.game_mode {
            crate::tui::setup::GameMode::EngineVsEngine => {
                matches!(app.state, crate::tui::tournament::app::TournamentState::Paused | crate::tui::tournament::app::TournamentState::Stopped)
            },
            _ => true,
        };

        Self {
            board,
            scale: app.board_scale,
            cursor_pos: app.cursor_pos,
            selected_square: app.selected_square,
            show_cursor,
        }
    }
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Chess Board ");
        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.width < 8 || inner_area.height < 8 {
            return;
        }

        let (cell_width, cell_height) = match self.scale {
            BoardScale::Small => (1, 1),
            BoardScale::Compact => (3, 2),
            BoardScale::Extended => (5, 4),
            BoardScale::Large => (
                (inner_area.width / 8).max(1),
                (inner_area.height / 8).max(1),
            ),
        };

        let board_width = cell_width * 8;
        let board_height = cell_height * 8;

        let start_x = inner_area.x + (inner_area.width - board_width) / 2;
        let start_y = inner_area.y + (inner_area.height - board_height) / 2;

        for rank in (0..8).rev() {
            for file in 0..8 {
                let square_index = rank * 8 + file;
                let is_light = (rank + file) % 2 != 0;
                
                let mut bg_color = if is_light { LIGHT_SQUARE } else { DARK_SQUARE };
                
                if self.show_cursor && square_index == self.cursor_pos {
                    bg_color = CURSOR_COLOR;
                } else if Some(square_index) == self.selected_square {
                    bg_color = SELECTED_COLOR;
                }

                let x = start_x + (file as u16 * cell_width);
                let y = start_y + ((7 - rank) as u16 * cell_height);

                for i in 0..cell_height {
                    for j in 0..cell_width {
                        if x + j < buf.area.width && y + i < buf.area.height {
                            buf[(x + j, y + i)].set_bg(bg_color);
                        }
                    }
                }

                let piece = self.board.get_piece_on_square(square_index);
                if piece != Pieces::NONE {
                    let side = if (self.board.bb_side[Sides::WHITE] >> square_index) & 1 == 1 {
                        Sides::WHITE
                    } else {
                        Sides::BLACK
                    };
                    let piece_size = match self.scale {
                        BoardScale::Small => PieceSize::Small,
                        BoardScale::Compact => PieceSize::Compact,
                        BoardScale::Extended => PieceSize::Extended,
                        BoardScale::Large => PieceSize::Large,
                    };

                    let piece_art = get_piece_art(piece, piece_size, side);
                    let piece_color = if side == Sides::WHITE {
                        Color::White
                    } else {
                        Color::Black
                    };

                    let art_lines: Vec<&str> = piece_art.lines().collect();
                    let art_height = art_lines.len() as u16;
                    let offset_y = (cell_height.saturating_sub(art_height)) / 2;

                    for (i, line) in art_lines.iter().enumerate() {
                        let line_y = y + offset_y + i as u16;
                        if line_y < y + cell_height && line_y < buf.area.height {
                            let line_width = line.chars().count() as u16;
                            let offset_x = (cell_width.saturating_sub(line_width)) / 2;
                            for (j, c) in line.chars().enumerate() {
                                let char_x = x + offset_x + j as u16;
                                if char_x < x + cell_width && char_x < buf.area.width {
                                    if c != ' ' {
                                        buf[(char_x, line_y)]
                                            .set_char(c)
                                            .set_fg(piece_color)
                                            .set_style(
                                                Style::default().add_modifier(Modifier::BOLD),
                                            );
                                    }
                                }
                            }
                        }
                    }
                }

                // Rank Labels (1-8)
                if file == 0 {
                    let label_x = start_x.saturating_sub(2);
                    let label_y = y + cell_height / 2;
                    if label_x < buf.area.width && label_y < buf.area.height && label_x > 0 {
                        buf[(label_x, label_y)].set_char((rank as u8 + b'1') as char);
                    }
                }

                // File Labels (a-h)
                if rank == 0 {
                    let label_x = x + cell_width / 2;
                    let label_y = start_y + board_height;
                    if label_x < buf.area.width && label_y < buf.area.height {
                        buf[(label_x, label_y)].set_char((file as u8 + b'a') as char);
                    }
                }
            }
        }
    }
}

fn get_piece_art(piece: usize, size: PieceSize, side: Side) -> String {
    match piece {
        Pieces::KING => King::get_art(size, side),
        Pieces::QUEEN => Queen::get_art(size, side),
        Pieces::ROOK => Rook::get_art(size, side),
        Pieces::BISHOP => Bishop::get_art(size, side),
        Pieces::KNIGHT => Knight::get_art(size, side),
        Pieces::PAWN => Pawn::get_art(size, side),
        _ => " ".to_string(),
    }
}
