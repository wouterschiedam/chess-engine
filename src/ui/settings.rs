use super::{config::UIConfig, ui::Message, ui::Tab};
use iced::widget::{
    column, row, Button, Checkbox, Column, Container, PickList, Scrollable, Text, TextInput,
};
use iced::{alignment, Alignment, Command, Element, Length, Theme};

#[derive(Clone, Debug)]
pub enum SettingsMessage {
    CheckFlipBoard(bool),
    CheckShowCoords(bool),
    CheckSetDepth(u32),
}

pub struct SettingsTab {
    pub flip_board: bool,
    pub show_coords: bool,
    pub search_depth: u32,
}

impl SettingsTab {
    pub fn new() -> Self {
        SettingsTab {
            flip_board: false,
            show_coords: true,
            search_depth: 3,
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Command<Message> {
        match message {
            SettingsMessage::CheckFlipBoard(value) => {
                self.flip_board = value;
                Command::perform(
                    SettingsTab::send_changes(self.flip_board, self.show_coords, self.search_depth),
                    Message::ChangeSettings,
                )
            }
            SettingsMessage::CheckShowCoords(value) => {
                self.show_coords = value;
                Command::perform(
                    SettingsTab::send_changes(self.flip_board, self.show_coords, self.search_depth),
                    Message::ChangeSettings,
                )
            }
            SettingsMessage::CheckSetDepth(value) => {
                self.search_depth = value;
                Command::perform(
                    SettingsTab::send_changes(self.flip_board, self.show_coords, self.search_depth),
                    Message::ChangeSettings,
                )
            }
        }
    }

    pub async fn send_changes(flip: bool, coords: bool, depth: u32) -> Option<UIConfig> {
        let mut config = UIConfig::default();
        config.flip_board = flip;
        config.show_coordinates = coords;
        config.search_depth = depth;
        Some(config)
    }
}

impl Tab for SettingsTab {
    type Message = Message;

    fn title(&self) -> String {
        "Chess engine".to_string()
    }

    // fn tab_label(&self) -> TabLabel {
    //     TabLabel::Text(self.title())
    // }

    fn content(&self) -> iced::Element<Message, iced::Renderer<iced::Theme>> {
        let col_settings = column![
            row![
                Text::new("show coordinates"),
                Checkbox::new("", self.show_coords, SettingsMessage::CheckShowCoords,).size(20),
            ]
            .spacing(5)
            .align_items(Alignment::Center),
            row![
                Text::new("flip board"),
                Checkbox::new("", self.flip_board, SettingsMessage::CheckFlipBoard,).size(20),
            ]
            .spacing(5)
            .align_items(Alignment::Center),
        ]
        .spacing(5)
        .align_items(Alignment::Center);

        let content: Element<SettingsMessage, iced::Renderer<Theme>> = Container::new(
            Scrollable::new(Column::new().spacing(10).push(col_settings)),
        )
        .align_x(alignment::Horizontal::Center)
        .height(Length::Fill)
        .width(Length::Fill)
        .into();

        content.map(Message::Settings)
    }
}