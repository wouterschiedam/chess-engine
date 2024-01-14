#[derive(Debug, Clone)]
pub struct UIConfig {
    pub show_coordinates: bool,
    pub flip_board: bool,
    pub search_depth: u32,
}

impl ::std::default::Default for UIConfig {
    fn default() -> Self {
        Self {
            show_coordinates: true,
            flip_board: false,
            search_depth: 3,
        }
    }
}
