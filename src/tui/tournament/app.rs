use crate::movegen::Move;
use crate::{board::Board, utils::display::format_move};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BoardScale {
    Small,
    Compact,
    Extended,
    Large,
}

pub enum TournamentState {
    Running,
    Paused,
    Stopped,
    Finished,
    ViewingResult,
    ReplayGame,
    Settings,
}

#[derive(Clone)]
pub struct GameRecord {
    pub moves: Vec<String>,
    pub result: String,
    pub white_engine: String,
    pub black_engine: String,
}

pub struct TournamentApp {
    pub games_played: usize,
    pub total_games: usize,
    pub current_game_moves: Vec<String>,
    pub board: Board,
    pub game_result: String,
    pub white_engine: String,
    pub black_engine: String,
    pub white_wins: usize,
    pub black_wins: usize,
    pub draws: usize,
    pub should_quit: bool,
    pub state: TournamentState,
    pub speed: usize,
    pub search_depth: u8,
    pub time_control: String,
    pub tournament_format: String,
    pub game_history: Vec<GameRecord>,
    pub selected_game: usize,
    pub replay_move_index: usize,
    pub replay_board: Board,
    pub board_scale: BoardScale,
    pub selected_setting: usize,
}

impl TournamentApp {
    pub fn new(total_games: usize, engine1: Option<String>, engine2: Option<String>) -> Self {
        let engine1_path = engine1.unwrap_or_else(|| "./target/release/chess".to_string());
        let engine2_path = engine2.unwrap_or_else(|| engine1_path.clone());

        Self {
            games_played: 0,
            total_games,
            current_game_moves: Vec::new(),
            board: Board::build(None),
            game_result: "Ready".to_string(),
            white_engine: engine1_path,
            black_engine: engine2_path,
            white_wins: 0,
            black_wins: 0,
            draws: 0,
            should_quit: false,
            state: TournamentState::Stopped,
            speed: 500,
            search_depth: 6,
            time_control: "blitz".to_string(),
            tournament_format: "gauntlet".to_string(),
            game_history: Vec::new(),
            selected_game: 0,
            replay_move_index: 0,
            replay_board: Board::build(None),
            board_scale: BoardScale::Large,
            selected_setting: 0,
        }
    }

    pub fn next_board_scale(&mut self) {
        self.board_scale = match self.board_scale {
            BoardScale::Small => BoardScale::Compact,
            BoardScale::Compact => BoardScale::Extended,
            BoardScale::Extended => BoardScale::Large,
            BoardScale::Large => BoardScale::Small,
        };
    }

    pub fn add_move(&mut self, mv: &Move) {
        self.current_game_moves.push(format_move(mv));
    }

    pub fn start_new_game(&mut self) {
        self.current_game_moves.clear();
        self.board = Board::build(None);
        self.game_result = "In Progress".to_string();
    }

    pub fn end_game(&mut self, result: &str) {
        self.game_result = result.to_string();
        match result {
            "1-0" => self.white_wins += 1,
            "0-1" => self.black_wins += 1,
            "1/2-1/2" => self.draws += 1,
            _ => {}
        }
        self.games_played += 1;

        self.game_history.push(GameRecord {
            moves: self.current_game_moves.clone(),
            result: result.to_string(),
            white_engine: self.white_engine.clone(),
            black_engine: self.black_engine.clone(),
        });
    }

    pub fn start(&mut self) {
        self.state = TournamentState::Running;
        if self.current_game_moves.is_empty() && self.game_result == "Ready" {
            self.start_new_game();
        }
    }

    pub fn pause(&mut self) {
        self.state = TournamentState::Paused;
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            TournamentState::Running => self.pause(),
            TournamentState::Paused => self.start(),
            TournamentState::Stopped => self.start(),
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.games_played = 0;
        self.white_wins = 0;
        self.black_wins = 0;
        self.draws = 0;
        self.state = TournamentState::Stopped;
        self.start_new_game();
        self.game_result = "Ready".to_string();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, TournamentState::Running)
    }

    pub fn finish_tournament(&mut self) {
        self.state = TournamentState::Finished;
    }

    pub fn view_results(&mut self) {
        self.state = TournamentState::ViewingResult;
        if !self.game_history.is_empty() && self.selected_game >= self.game_history.len() {
            self.selected_game = 0;
        }
    }

    pub fn replay_game(&mut self, game_index: usize) {
        if game_index < self.game_history.len() {
            self.selected_game = game_index;
            self.replay_move_index = 0;
            self.replay_board = Board::build(None);
            self.state = TournamentState::ReplayGame;
        }
    }

    pub fn replay_next_move(&mut self) {
        if let Some(game) = self.game_history.get(self.selected_game) {
            if self.replay_move_index < game.moves.len() {
                self.replay_move_index += 1;
            }
        }
    }

    pub fn replay_prev_move(&mut self) {
        if self.replay_move_index > 0 {
            self.replay_move_index -= 1;
        }
    }

    pub fn restart_tournament(&mut self) {
        self.game_history.clear();
        self.games_played = 0;
        self.white_wins = 0;
        self.black_wins = 0;
        self.draws = 0;
        self.state = TournamentState::Stopped;
        self.start_new_game();
        self.game_result = "Ready".to_string();
    }
}
