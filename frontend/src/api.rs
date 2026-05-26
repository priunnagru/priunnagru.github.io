use reqwest::Client;
use gloo_timers::future::TimeoutFuture;
use crate::types::{Anime, CustomGameInput, GameResponse, RecsInput, RecsResponse, VerifyWinInput, VerifyWinResponse};

const API_URL: &str = "http://localhost:3000";
const MAX_RETRIES: u32 = 3;

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// Send a request with automatic retry on 429 (rate limited) responses.
    /// Uses exponential backoff: 1s, 2s, 4s...
    async fn send_with_retry<T: serde::de::DeserializeOwned>(
        request: &reqwest::RequestBuilder,
        retry_count: u32,
    ) -> Result<T, String> {
        for attempt in 0..=retry_count {
            let response = request
                .try_clone()
                .expect("unable to clone request")
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let wait_ms = (2u64.saturating_pow(attempt + 1)) * 1000;
                log::warn!("Rate limited (429). Retrying in {}s... (attempt {}/{})", wait_ms / 1000, attempt + 1, retry_count);
                TimeoutFuture::new(wait_ms as u32).await;
                continue;
            }

            if !status.is_success() {
                return Err(format!("Server returned {}", status));
            }

            return response
                .json::<T>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e));
        }
        Err("Rate limited - too many retries".to_string())
    }

    pub async fn get_daily_game(&self) -> Result<GameResponse, String> {
        let url = format!("{}/game/daily", API_URL);
        let request = self.client.get(&url);
        Self::send_with_retry::<GameResponse>(&request, MAX_RETRIES).await
    }

    pub async fn get_custom_game(&self, start_id: i32, end_id: i32) -> Result<GameResponse, String> {
        let url = format!("{}/game/custom", API_URL);
        let payload = CustomGameInput { start_id, end_id };
        let request = self.client.post(&url).json(&payload);
        Self::send_with_retry::<GameResponse>(&request, MAX_RETRIES).await
    }

    pub async fn get_recs(&self, token: &str, path: &[i32]) -> Result<RecsResponse, String> {
        let url = format!("{}/game/recs", API_URL);
        let payload = RecsInput {
            token: token.to_string(),
            path: path.to_vec(),
        };
        let request = self.client.post(&url).json(&payload);
        Self::send_with_retry::<RecsResponse>(&request, MAX_RETRIES).await
    }

    pub async fn verify_win(&self, token: &str, path: &[i32]) -> Result<VerifyWinResponse, String> {
        let url = format!("{}/game/win", API_URL);
        let payload = VerifyWinInput {
            token: token.to_string(),
            path: path.to_vec(),
        };
        let request = self.client.post(&url).json(&payload);
        Self::send_with_retry::<VerifyWinResponse>(&request, MAX_RETRIES).await
    }

    pub async fn search_anime(&self, query: &str) -> Result<Vec<Anime>, String> {
        // This would need a search endpoint on the server
        // For now, we'll implement a simple search if needed
        Err("Search not implemented yet".to_string())
    }
}
