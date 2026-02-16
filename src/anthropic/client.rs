use std::time::Duration;

use reqwest::Client;

use super::error::AnthropicError;
use super::types::{MessagesRequest, MessagesResponse};

const API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct AnthropicClient {
    api_key: String,
    client: Client,
    base_url: String,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, API_URL.to_string())
    }

    /// Create a client pointing at a custom base URL (useful for testing).
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build HTTP client");
        Self {
            api_key,
            client,
            base_url,
        }
    }

    pub async fn send_message(
        &self,
        req: &MessagesRequest,
    ) -> Result<MessagesResponse, AnthropicError> {
        let response = self
            .client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(req)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(|secs| secs * 1000)
                .unwrap_or(1000);
            return Err(AnthropicError::RateLimited {
                retry_after_ms: retry_after,
            });
        }

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(AnthropicError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.json::<MessagesResponse>().await?;
        Ok(body)
    }
}
