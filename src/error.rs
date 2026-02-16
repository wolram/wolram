use thiserror::Error;

#[derive(Debug, Error)]
pub enum WolramError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("No jobs found. Run `wolram init` first.")]
    NoJobs,

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Anthropic API error: {0}")]
    Anthropic(#[from] AnthropicError),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum AnthropicError {
    #[error("API returned status {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("Request timed out")]
    Timeout,

    #[error("Failed to parse API response: {0}")]
    ParseError(String),
}

/// Classifies a job failure for retry logic decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FailureKind {
    /// Logic/validation failure (wrong output, tests fail)
    Business,
    /// Infrastructure failure (API timeout, rate limit, network error)
    System,
}

impl std::fmt::Display for FailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureKind::Business => write!(f, "Business"),
            FailureKind::System => write!(f, "System"),
        }
    }
}
