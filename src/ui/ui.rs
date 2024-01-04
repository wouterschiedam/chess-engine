use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use super::board;
use super::squares::{Position, SQUARE_SIZE};
use super::styling::container::{DarkSquare, LightSquare, Squares, Terminal};
use crate::board::Board;
use crate::engine::Engine;
use iced::advanced::Widget;
use iced::widget::container::Id;
use iced::widget::{column, row, Button, Column, Container, Row, Text, TextInput};
use iced::{Alignment, Element, Length, Sandbox, Settings, Theme};
pub struct Editor {
    input: String,
    moves_log: Vec<String>,
    terminal_log: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextInputSend(String),
    Start,
    Reset,
    Received,
    SendMessage,
}

pub fn run() -> iced::Result {
    let _ = thread::spawn(|| {
        let mut engine = Engine::new();
        let _ = engine.run();
    });
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
            input: "".to_string(),
            moves_log: vec![],
            terminal_log: ">> Waiting for uci command".to_string(),
        }
    }

    fn title(&self) -> String {
        String::from("Chess app")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::TextInputSend(content) => self.input = content,
            Message::Start => {
                println!("uci");
            }
            Message::Reset => {
                self.terminal_log = "".to_string();
                self.input = "".to_string();
            }
            Message::Received => {}
            Message::SendMessage => {
                self.moves_log.push(self.input.clone());
                self.terminal_log += &("\n>> ".to_owned() + &self.input);
                // Send moves to engine
                println!(
                    "position moves {}",
                    self.moves_log
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                );
                // Reset input field
                self.input = "".to_string();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut result = Column::new().spacing(0).width(SQUARE_SIZE * 8);
        let mut row = Row::new().spacing(0).align_items(Alignment::Center);
        let mut i = 0;
        let mut j = 0;
        let mut count = 0;

        // init board
        for button in 1..=64 {
            let (button_style, piece) = board::create_board(&button, &count);

            match button_style {
                Squares::Even(LightSquare) => {
                    row = row.push(
                        Container::new(Row::new().push(piece))
                            .width(SQUARE_SIZE)
                            .height(SQUARE_SIZE)
                            .style(iced::theme::Container::Custom(Box::new(LightSquare)))
                            .id(Id::new(format!("{} {}", j + 1, count + 1))),
                    )
                }
                Squares::Odd(DarkSquare) => {
                    row = row.push(
                        Container::new(Row::new().push(piece))
                            .width(SQUARE_SIZE)
                            .height(SQUARE_SIZE)
                            .style(iced::theme::Container::Custom(Box::new(DarkSquare)))
                            .id(Id::new(format!("{} {}", j + 1, count + 1))),
                    )
                }
            };

            i += 1;
            j += 1;

            if i % 8 == 0 {
                count += 1;
                j = 0;
                result = result.push(row);
                row = Row::new().spacing(0).align_items(Alignment::Center);
            }
        }

        // Init terminal
        let logs = Row::new().push(
            Container::new(Text::new(&self.terminal_log).width(SQUARE_SIZE * 8))
                .height(281)
                .style(iced::theme::Container::Custom(Box::new(Terminal))),
        );

        let input = row![
            TextInput::new(">> ", &self.input)
                .width(SQUARE_SIZE * 7)
                .on_input(Message::TextInputSend),
            Button::new("Submit")
                .on_press(Message::SendMessage)
                .width(SQUARE_SIZE)
        ]
        .height(Length::Fill);
        let buttons = column![
            Button::new("Start Game").on_press(Message::Start),
            Button::new("Reset Game").on_press(Message::Reset),
        ]
        .spacing(10);

        let side = Container::new(buttons)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .center_x();

        // Side options
        let sidebar = Column::new().push(side);

        Container::new(row![column![result, logs, input], sidebar])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
