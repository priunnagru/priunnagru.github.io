use reqwest::Client;
use serde::Deserialize;
use gloo_timers::future::TimeoutFuture;
use crate::types::{ApiError, Anime, CustomGameInput, GameResponse, RecsInput, RecsResponse,
    SolutionData, SolutionInput, VerifyWinInput, VerifyWinResponse};

const RAW_ENV_URL: &str = "https://gist.githubusercontent.com/priunnagru/e5ec2ec0506d857526bdc49a7ece64ec/raw/anirecdle_env.json";
const BACKUP_FALLBACK_URL: &str = "http://localhost:3000";
const MAX_RETRIES: u32 = 3;
const GIST_RETRY_DELAY_MS: u32 = 1000;

#[cfg(debug_mode)]
const LOCALHOST_URL: &str = "http://localhost:3000";

#[derive(Deserialize)]
struct EnvConfig {
    backend_url: String,
}

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

    async fn resolve_backend_url(&self) -> String {
        #[cfg(debug_mode)]
        {
            log::info!("Debug mode enabled — using localhost backend");
            return LOCALHOST_URL.to_string();
        }

        for attempt in 1..=MAX_RETRIES {
            log::info!("Attempt {}/{} to fetch raw config...", attempt, MAX_RETRIES);

            // No special GitHub headers or API parsing required anymore
            let response = self.client.get(RAW_ENV_URL).send().await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(config) = resp.json::<EnvConfig>().await {
                            log::info!("Using backend url: {}", config.backend_url);
                            return config.backend_url.trim_end_matches('/').to_string();
                        }
                    }
                    log::warn!("Raw config parsing failed on attempt {}.", attempt);
                }
                Err(err) => {
                    log::error!("Network error fetching raw config on attempt {}: {:?}", attempt, err);
                }
            }

            if attempt < MAX_RETRIES {
                TimeoutFuture::new(GIST_RETRY_DELAY_MS).await;
            }
        }

        log::error!("All config resolution retries failed. Falling back to default.");
        BACKUP_FALLBACK_URL.to_string()
    }

    /// Send a request with automatic retry on 429 (rate limited) responses.
    /// Returns ApiError::Conflict on 409, ApiError::Other for all other failures.
    async fn send_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        make_request: impl Fn(&Client, &str) -> reqwest::RequestBuilder,
        retry_count: u32,
    ) -> Result<T, ApiError> {
        let base_url = self.resolve_backend_url().await;

        for attempt in 0..=retry_count {
            let request = make_request(&self.client, &base_url);

            let response = request
                .send()
                .await
                .map_err(|e| ApiError::Other(format!("Request failed: {}", e)))?;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let wait_ms = (2u64.saturating_pow(attempt + 1)) * 1000;
                log::warn!("Rate limited (429). Retrying in {}s... (attempt {}/{})", wait_ms / 1000, attempt + 1, retry_count);
                TimeoutFuture::new(wait_ms as u32).await;
                continue;
            }

            if status == reqwest::StatusCode::CONFLICT {
                return Err(ApiError::Conflict("This request conflicts with today's daily challenge.".to_string()));
            }

            if !status.is_success() {
                return Err(ApiError::Other(format!("Server returned {}", status)));
            }

            return response
                .json::<T>()
                .await
                .map_err(|e| ApiError::Other(format!("Failed to parse response: {}", e)));
        }
        Err(ApiError::Other("Rate limited - too many retries".to_string()))
    }

    pub async fn get_daily_game(&self) -> Result<GameResponse, ApiError> {
        Self::send_with_retry::<GameResponse>(
            self,
            |client, base_url| client.get(format!("{}/game/daily", base_url)),
            MAX_RETRIES
        ).await
    }

    pub async fn get_custom_game(&self, start_id: i32, end_id: i32) -> Result<GameResponse, ApiError> {
        let payload = CustomGameInput { start_id, end_id };
        Self::send_with_retry::<GameResponse>(
            self,
            |client, base_url| client.post(format!("{}/game/custom", base_url)).json(&payload),
            MAX_RETRIES
        ).await
    }

    pub async fn get_recs(&self, token: &str, path: &[i32]) -> Result<RecsResponse, ApiError> {
        let payload = RecsInput {
            token: token.to_string(),
            path: path.to_vec(),
        };
        Self::send_with_retry::<RecsResponse>(
            self,
            |client, base_url| client.post(format!("{}/game/recs", base_url)).json(&payload),
            MAX_RETRIES
        ).await
    }

    pub async fn verify_win(&self, token: &str, path: &[i32]) -> Result<VerifyWinResponse, ApiError> {
        let payload = VerifyWinInput {
            token: token.to_string(),
            path: path.to_vec(),
        };
        Self::send_with_retry::<VerifyWinResponse>(
            self,
            |client, base_url| client.post(format!("{}/game/win", base_url)).json(&payload),
            MAX_RETRIES
        ).await
    }

    pub async fn get_solution(&self, token: &str, path: &[i32]) -> Result<SolutionData, ApiError> {
        let payload = SolutionInput {
            token: token.to_string(),
            path: path.to_vec(),
        };
        Self::send_with_retry::<SolutionData>(
            self,
            |client, base_url| client.post(format!("{}/game/solution", base_url)).json(&payload),
            MAX_RETRIES
        ).await
    }

    pub async fn search_anime(&self, _query: &str) -> Result<Vec<Anime>, ApiError> {
        Err(ApiError::Other("Search not implemented yet".to_string()))
    }
}
