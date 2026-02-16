//! Tipos de erro para o cliente da API Anthropic.
//!
//! Define [`AnthropicError`] com variantes para rate limiting, erros da API
//! e erros de rede. Usa `thiserror` para derivar `Display` e `Error`
//! automaticamente a partir dos atributos `#[error(...)]`.

use thiserror::Error;

/// Erros que podem ocorrer ao interagir com a API da Anthropic.
///
/// As variantes cobrem os três cenários mais comuns de falha:
/// - [`RateLimited`](AnthropicError::RateLimited) — o servidor retornou HTTP 429
/// - [`ApiError`](AnthropicError::ApiError) — qualquer outro erro HTTP (4xx/5xx)
/// - [`NetworkError`](AnthropicError::NetworkError) — falha na camada de rede
#[derive(Debug, Error)]
pub enum AnthropicError {
    /// O servidor retornou HTTP 429 (rate limit).
    /// O campo `retry_after_ms` indica quantos milissegundos esperar antes de retentar.
    #[error("rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    /// Erro retornado pela API (ex.: 401 chave inválida, 500 erro interno).
    /// Contém o código de status HTTP e a mensagem de erro do corpo da resposta.
    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    /// Falha de rede subjacente (DNS, conexão recusada, timeout).
    /// Encapsula o erro original do `reqwest` via `#[from]`.
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
        assert_eq!(err.to_string(), "API error (status 401): Invalid API key");
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AnthropicError>();
    }
}
