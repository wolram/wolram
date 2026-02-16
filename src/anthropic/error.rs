use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnthropicError {
    #[error("rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
