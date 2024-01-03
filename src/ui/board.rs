use iced::widget::{
    image::{self, Handle},
    Image,
};

use super::styling::container::{DarkSquare, LightSquare, Squares};

pub fn create_board(index: &i32, row: &i32) -> (Squares, Image<Handle>) {
    let button_style: Squares;
    let index = index.clone();
    let row = row.clone();

    let mut piece: Image<Handle> = Image::new("resources");
    if row == 0 {
        if index == 1 || index == 8 {
            piece = Image::<image::Handle>::new("resources/rook_black.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 2 || index == 7 {
            piece = Image::<image::Handle>::new("resources/knight_black.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 3 || index == 6 {
            piece = Image::<image::Handle>::new("resources/bishop_black.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 4 {
            piece = Image::<image::Handle>::new("resources/queen_black.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 5 {
            piece = Image::<image::Handle>::new("resources/king_black.png")
                .content_fit(iced::ContentFit::Cover);
        }
    }
    if row == 7 {
        if index == 57 || index == 64 {
            piece = Image::<image::Handle>::new("resources/rook_white.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 58 || index == 63 {
            piece = Image::<image::Handle>::new("resources/knight_white.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 59 || index == 62 {
            piece = Image::<image::Handle>::new("resources/bishop_white.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 60 {
            piece = Image::<image::Handle>::new("resources/queen_white.png")
                .content_fit(iced::ContentFit::Cover);
        } else if index == 61 {
            piece = Image::<image::Handle>::new("resources/king_white.png")
                .content_fit(iced::ContentFit::Cover);
        }
    }
    if row == 1 {
        piece = Image::<image::Handle>::new("resources/pawn_black.png")
            .content_fit(iced::ContentFit::Cover);
    }
    if row == 6 {
        piece = Image::<image::Handle>::new("resources/pawn_white.png")
            .content_fit(iced::ContentFit::Cover);
    }
    if row % 2 == 0 {
        if index % 2 == 0 {
            button_style = Squares::Even(LightSquare)
        } else {
            button_style = Squares::Odd(DarkSquare)
        }
    } else {
        if !index % 2 == 0 {
            button_style = Squares::Even(LightSquare)
        } else {
            button_style = Squares::Odd(DarkSquare)
        }
    }

    (button_style, piece)
}
