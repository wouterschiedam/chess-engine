use std::path::Path;

use super::config::UIConfig;
use super::engine::{EngineStatus, UIengine};
use super::settings::{SettingsMessage, SettingsTab};
use super::styling::button::{self, CustomButtonStyle};
use crate::board::defs::Pieces;
use crate::board::Board;
use crate::defs::{Sides, Square};
use crate::movegen::defs::{Move, MoveList, MoveType};
use crate::movegen::MoveGenerator;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    column, image, responsive, row, Button, Column, Container, Image, Radio, Row, Svg, Text,
};
use iced::{
    executor, Alignment, Application, Color, Command, Element, Length, Sandbox, Settings, Size,
    Subscription, Theme,
};
use tokio::sync::mpsc::Sender;

pub struct Editor {
    board: Board,
    engine: UIengine,
    engine_status: EngineStatus,
    movegen: MoveGenerator,
    legal_moves: MoveList,
    settings: SettingsTab,
    from_square: Option<Square>,
    engine_sender: Option<Sender<String>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Settings(SettingsMessage),
    ChangeSettings(Option<UIConfig>),
    SelectSquare(Option<Square>),
    EventOccurred(iced::Event),
    StartEngine,
    EngineReady(Sender<String>),
    EngineStopped(bool),
    UndoMoveVirtual,
    NextMoveVirtual,
    ResetBoardEngine,
    SelectSideToMove(usize),
}

pub fn run() -> iced::Result {
    // let _ = thread::spawn(|| {
    //     let mut engine = Engine::new();
    //     let _ = engine.run();
    // });
    Editor::run(Settings {
        window: iced::window::Settings {
            size: (1130, 1080),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                board: Board::build(),
                engine: UIengine::new(),
                engine_status: EngineStatus::TurnedOff,
                movegen: MoveGenerator::new(),
                legal_moves: MoveList::new(),
                settings: SettingsTab::new(),
                from_square: None,
                engine_sender: None,
            },
            Command::none(),
        )
    }
    fn title(&self) -> String {
        String::from("Chess app")
    }

    fn update(&mut self, message: self::Message) -> Command<Message> {
        match (self.from_square, message) {
            (None, Message::SelectSquare(pos)) => {
                //
                let side = self.board.side_to_move();
                let color = self.board.color_on(pos);

                // if user clicked on another square with a piece his own side reset
                if color == side {
                    self.from_square = pos;
                }
                Command::none()
            }
            (Some(from), Message::SelectSquare(to)) if from != to.unwrap() => {
                let side = self.board.side_to_move();
                let color = self.board.color_on(to);
                if color == side {
                    self.from_square = Some(from);
                    return Command::none();
                }

                self.from_square = None;

                // Get all psuedo legal moves for the position.
                self.movegen
                    .generate_moves(&self.board, &mut self.legal_moves, MoveType::All);

                // get data needed for converting algebraic move to Move data
                let is_white = self.board.side_to_move() == Sides::WHITE;
                let move_data = self.board.generate_move_data(&from, &to, is_white);
                // Check if move is legal
                if self.legal_moves.moves.iter().any(|x| x.data == move_data) {
                    self.board.make_move(Move::new(move_data), &self.movegen);
                } else {
                    println!("illegal move");
                }

                Command::none()
            }
            (_, Message::StartEngine) => {
                match self.engine_status {
                    EngineStatus::TurnedOff => {
                        // Check if engine path is correct
                        if Path::new(&self.engine.engine_path).exists() {
                            self.engine.position = self.board.create_fen();
                            self.engine_status = EngineStatus::TurnedOn;
                        } else {
                            println!("Invalid engine path");
                        }
                    }
                    _ => {
                        if let Some(sender) = &self.engine_sender {
                            sender
                                .blocking_send(String::from("STOP"))
                                .expect("Error quiting engine");
                            self.engine_sender = None;
                        }
                    }
                }
                Command::none()
            }
            (_, Message::EngineReady(message)) => {
                println!("Engine is ready");
                self.engine_sender = Some(message);
                Command::none()
            }
            (_, Message::EventOccurred(event)) => {
                //
                Command::none()
            }
            (_, Message::Settings(message)) => self.settings.update(message),
            (_, Message::ChangeSettings(message)) => {
                if let Some(settings) = message {
                    self.settings.flip_board = settings.flip_board;
                    self.settings.show_coords = settings.show_coordinates;
                    self.settings.search_depth = settings.search_depth;
                }
                Command::none()
            }
            (_, Message::SelectSideToMove(message)) => {
                self.board.swap_side();
                Command::none()
            }
            (_, Message::UndoMoveVirtual) => {
                if self.board.history.len() > 0 {
                    self.board.unmake();
                }
                Command::none()
            }
            (_, Message::NextMoveVirtual) => {
                //
                Command::none()
            }
            (_, _) => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.engine_status {
            EngineStatus::TurnedOff => iced::subscription::events().map(Message::EventOccurred),
            _ => Subscription::batch(vec![
                UIengine::run_engine(self.engine.clone()),
                iced::subscription::events().map(Message::EventOccurred),
            ]),
        }
    }

    fn view(&self) -> Element<Message, iced::Renderer<Theme>> {
        let resp = responsive(move |size| {
            main_view(
                &self.board,
                self.settings.flip_board,
                self.settings.show_coords,
                self.settings.search_depth,
                self.settings.view(),
                self.engine_status != EngineStatus::TurnedOff,
                size,
            )
        });

        Container::new(resp).padding(1).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn main_view<'a>(
    board: &Board,
    flip_board: bool,
    show_coordinates: bool,
    search: u32,
    settings_tab: Element<'a, Message, iced::Renderer<Theme>>,
    engine_started: bool,
    size: Size,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    let mut board_col = Column::new().spacing(0).align_items(Alignment::Center);
    let mut board_row = Row::new().spacing(0).align_items(Alignment::Center);

    let ranks;
    let files;
    ranks = (1..=8).collect::<Vec<i32>>();
    files = (1..=8).rev().collect::<Vec<i32>>();
    let board_height = 100;
    let row = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

    for rank in ranks {
        for file in &files {
            let pos = board.get_square((rank as usize, *file as usize));

            let (piece, color) = (board.piece_on(pos), board.color_on(pos));

            let mut text = "";
            let light_square = (rank + file) % 2 != 0;

            // let selected = from_square == Some(pos);
            let square_style = if light_square {
                CustomButtonStyle::new()
                    .background_color(Color::from_rgb(0.91, 0.741, 0.529))
                    .hovered()
                    .background_color(Color::from_rgb(0.91, 0.741, 0.529))
                    .pressed()
                    .background_color(Color::from_rgb(0.803, 0.82, 0.415))
            } else {
                CustomButtonStyle::new()
                    .background_color(Color::from_rgb(0.639, 0.502, 0.329))
                    .hovered()
                    .background_color(Color::from_rgb(0.639, 0.502, 0.329))
                    .pressed()
                    .background_color(Color::from_rgb(0.666, 0.635, 0.22))
            };

            // Set pieces on boad
            if let Some(piece) = piece {
                if color == Sides::WHITE {
                    text = match piece {
                        Pieces::PAWN => "/wP.svg",
                        Pieces::ROOK => "/wR.svg",
                        Pieces::KNIGHT => "/wN.svg",
                        Pieces::BISHOP => "/wB.svg",
                        Pieces::QUEEN => "/wQ.svg",
                        Pieces::KING => "/wK.svg",
                        _ => "",
                    };
                } else {
                    text = match piece {
                        Pieces::PAWN => "/bP.svg",
                        Pieces::ROOK => "/bR.svg",
                        Pieces::KNIGHT => "/bN.svg",
                        Pieces::BISHOP => "/bB.svg",
                        Pieces::QUEEN => "/bQ.svg",
                        Pieces::KING => "/bK.svg",
                        _ => "",
                    };
                }

                board_col = board_col.push(
                    Button::new(Svg::from_path(format!(
                        "{}/pieces{}",
                        env!("CARGO_MANIFEST_DIR"),
                        text
                    )))
                    .width(board_height)
                    .height(board_height)
                    .on_press(Message::SelectSquare(pos))
                    .style(square_style.as_custom()),
                );
            } else {
                board_col = board_col.push(
                    Button::new(Text::new(""))
                        .width(board_height)
                        .height(board_height)
                        .on_press(Message::SelectSquare(pos))
                        .style(square_style.as_custom()),
                );
            }
        }

        if show_coordinates {
            board_col = board_col.push(
                Container::new(Text::new((row[rank as usize - 1]).to_string()).size(15))
                    .align_y(iced::alignment::Vertical::Top)
                    .align_x(iced::alignment::Horizontal::Left)
                    .padding(5)
                    .width(board_height),
            );
        }

        board_row = board_row.push(board_col);
        board_col = Column::new().spacing(0).align_items(Alignment::Center);
    }

    if show_coordinates {
        if !flip_board {
            board_row = board_row.push(
                column![
                    Text::new("8").size(15).height(board_height),
                    Text::new("7").size(15).height(board_height),
                    Text::new("6").size(15).height(board_height),
                    Text::new("5").size(15).height(board_height),
                    Text::new("4").size(15).height(board_height),
                    Text::new("3").size(15).height(board_height),
                    Text::new("2").size(15).height(board_height),
                    Text::new("1").size(15).height(board_height),
                ]
                .padding(5),
            );
        } else {
            board_row = board_row.push(column![
                Text::new("1").size(15).height(board_height),
                Text::new("2").size(15).height(board_height),
                Text::new("3").size(15).height(board_height),
                Text::new("4").size(15).height(board_height),
                Text::new("5").size(15).height(board_height),
                Text::new("6").size(15).height(board_height),
                Text::new("7").size(15).height(board_height),
                Text::new("8").size(15).height(board_height),
            ]);
        }
    }

    let mut side_to_play = row![];

    if board.side_to_move() == Sides::WHITE {
        side_to_play = side_to_play.push(Text::new("White to move"));
    } else {
        side_to_play = side_to_play.push(Text::new("Black to move"));
    }

    let game_mode_row = row![
        Text::new("Play as"),
        Radio::new(
            "White",
            Sides::WHITE,
            Some(board.side_to_move()),
            Message::SelectSideToMove
        ),
        Radio::new(
            "Black",
            Sides::BLACK,
            Some(board.side_to_move()),
            Message::SelectSideToMove
        )
    ]
    .spacing(10)
    .padding(10)
    .align_items(Alignment::Center);

    let mut navigation_row = Row::new().padding(3).spacing(10);

    if engine_started {
        navigation_row = navigation_row
            .push(Button::new(Text::new("Stop engine")).on_press(Message::StartEngine));
    } else {
        navigation_row = navigation_row
            .push(Button::new(Text::new("Start engine")).on_press(Message::StartEngine));
    }

    navigation_row = navigation_row
        .push(Button::new(Text::new("< Previous")).on_press(Message::UndoMoveVirtual));

    navigation_row =
        navigation_row.push(Button::new(Text::new("Next >")).on_press(Message::NextMoveVirtual));

    navigation_row = navigation_row
        .push(Button::new(Text::new("Reset board")).on_press(Message::ResetBoardEngine));

    row![
        column![
            board_row,
            column![side_to_play, game_mode_row, navigation_row]
                .width(board_height * 8)
                .height(Length::Fill)
                .align_items(Alignment::Center)
        ],
        settings_tab
    ]
    .into()
}

pub trait Tab {
    type Message;

    fn title(&self) -> String;

    // fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<Message, iced::Renderer<Theme>> {
        let column = Column::new()
            .spacing(20)
            .push(Text::new(self.title()).size(20))
            .push(self.content());

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(20)
            .into()
    }

    fn content(&self) -> Element<Message, iced::Renderer<Theme>>;
}
