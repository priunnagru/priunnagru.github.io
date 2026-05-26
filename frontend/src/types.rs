use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Anime {
    pub id: i32,
    pub image_url: String,
    pub title_native: String,
    pub title_english: Option<String>,
    pub title_romaji: String,
    pub popularity: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GameResponse {
    pub token: String,
    pub start: Anime,
    pub end: Anime,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RecsResponse {
    pub recs: Vec<Anime>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RecsInput {
    pub token: String,
    pub path: Vec<i32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct VerifyWinInput {
    pub token: String,
    pub path: Vec<i32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VerifyWinResponse {
    pub is_valid: bool,
    pub shortest_paths: Vec<Vec<Anime>>,
    pub min_steps: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct CustomGameInput {
    pub start_id: i32,
    pub end_id: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameMode {
    Daily,
    Custom,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub mode: GameMode,
    pub token: String,
    pub start: Anime,
    pub end: Anime,
    pub path: Vec<Anime>,
    pub recs: Vec<Anime>,
    pub loading: bool,
    pub won: bool,
    pub min_steps: usize,
    pub error: Option<String>,
}
