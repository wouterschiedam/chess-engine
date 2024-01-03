use crate::board::Board;
use crate::defs::Sides;
use iced::widget::text_input::Value;
use iced::widget::{text_input, Column, Container, Row, Text, TextInput};
use iced::{Alignment, Element, Length, Sandbox, Settings, Theme};

use super::board;
use super::squares::{Position, SQUARE_SIZE};
use super::styling::container::{DarkSquare, LightSquare, Squares};

pub struct Editor {
    board: Board,
    input: TextInput<'static, Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextInputChanged(String),
}

pub fn run() -> iced::Result {
    Editor::run(Settings {
        window: iced::window::Settings {
            size: (1130, 1080),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self {
            board: Board::new(),
            input: TextInput::new("send uci command", ""),
        }
    }

    fn title(&self) -> String {
        String::from("Chess app")
    }

    fn update(&mut self, _: Message) {}

    fn view(&self) -> Element<Message> {
        let mut result = Column::new().spacing(0).align_items(Alignment::Center);
        let mut row = Row::new().spacing(0).align_items(Alignment::Center);
        let mut i = 0;
        let mut count = 0;

        let is_white = self.board.side_to_move() == Sides::WHITE;
        for button in 1..=64 {
            let r = if is_white { 7 - i / 8 } else { i / 8 };
            let c = if is_white { i % 8 } else { 7 - (i % 8) };
            let pos = Position::new(r, c);

            // let (text, color) = if let Some(piece) = self.board.get_pieces(piece, Sides::WHITE) {
            //     (get_symbol(&piece).to_string(), piece.get_color())
            // } else {
            //     (String::from(" "), Sides::BOTH)
            // };
            //

            let (button_style, piece) = board::create_board(&button, &count);

            match button_style {
                Squares::Even(LightSquare) => {
                    row = row.push(
                        Container::new(piece)
                            .width(SQUARE_SIZE)
                            .height(SQUARE_SIZE)
                            .style(iced::theme::Container::Custom(Box::new(LightSquare))),
                    );
                }
                Squares::Odd(DarkSquare) => {
                    row = row.push(
                        Container::new(piece)
                            .width(SQUARE_SIZE)
                            .height(SQUARE_SIZE)
                            .style(iced::theme::Container::Custom(Box::new(DarkSquare))),
                    );
                }
            };

            i += 1;

            if i % 8 == 0 {
                count += 1;
                result = result.push(row);
                row = Row::new().spacing(0).align_items(Alignment::Center);
            }
        }

        // let test = Column::new(TextInput::new("x", "x1"));

        // row.push(test);

        Container::new(result)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(1)
            // .style()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
