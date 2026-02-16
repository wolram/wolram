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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limited_display() {
        let err = AnthropicError::RateLimited {
            retry_after_ms: 5000,
        };
        assert_eq!(err.to_string(), "rate limited, retry after 5000ms");
    }

    #[test]
    fn api_error_display() {
        let err = AnthropicError::ApiError {
            status: 401,
            message: "Invalid API key".into(),
        };
        assert_eq!(
            err.to_string(),
            "API error (status 401): Invalid API key"
        );
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AnthropicError>();
    }
}
