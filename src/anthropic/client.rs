//! Cliente HTTP para a API Anthropic Messages.
//!
//! Encapsula a lógica de autenticação, envio de requisições e tratamento
//! de respostas (incluindo rate limiting e erros HTTP). Usa `reqwest` internamente.

use std::time::Duration;

use reqwest::Client;

use super::error::AnthropicError;
use super::types::{MessagesRequest, MessagesResponse};

// URL padrão da API Anthropic Messages v1.
const API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Cliente HTTP para enviar requisições à API Anthropic Messages.
///
/// Gerencia a chave de API, timeouts de conexão e a URL base.
/// Em testes, a URL base pode ser apontada para um servidor mock
/// via [`with_base_url`](Self::with_base_url).
pub struct AnthropicClient {
    // Chave de autenticação para a API Anthropic.
    api_key: String,
    // Cliente HTTP reqwest reutilizável (com pool de conexões).
    client: Client,
    // URL base para requisições (padrão: API de produção da Anthropic).
    base_url: String,
}

impl AnthropicClient {
    /// Cria um novo cliente apontando para a API de produção da Anthropic.
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, API_URL.to_string())
    }

    /// Cria um cliente apontando para uma URL base customizada (útil para testes com mock server).
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

    /// Envia uma requisição de mensagem para a API Anthropic.
    ///
    /// Retorna [`MessagesResponse`] em caso de sucesso, ou [`AnthropicError`]
    /// para rate limiting (429), erros HTTP ou falhas de rede.
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

        // Verifica se o servidor retornou HTTP 429 (rate limit).
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Extrai o header retry-after e converte de segundos para milissegundos.
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

        // Qualquer outro erro HTTP: extrai o corpo da resposta como mensagem de erro.
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

        // Sucesso: deserializa o corpo JSON como MessagesResponse.
        let body = response.json::<MessagesResponse>().await?;
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_request() -> MessagesRequest {
        MessagesRequest {
            model: "claude-sonnet-4-5-20250929".into(),
            max_tokens: 1024,
            messages: vec![super::super::types::Message {
                role: "user".into(),
                content: "Hi".into(),
            }],
        }
    }

    #[tokio::test]
    async fn successful_response_parsing() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg_123",
                "content": [{"type": "text", "text": "Hello!"}],
                "model": "claude-sonnet-4-5-20250929",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 10, "output_tokens": 5}
            })))
            .mount(&server)
            .await;

        let client = AnthropicClient::with_base_url("test-key".into(), server.uri());
        let resp = client.send_message(&test_request()).await.unwrap();
        assert_eq!(resp.id, "msg_123");
        assert_eq!(resp.content[0].text, "Hello!");
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 5);
        assert_eq!(resp.stop_reason, Some("end_turn".into()));
    }

    #[tokio::test]
    async fn rate_limit_429_with_retry_after() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "5"))
            .mount(&server)
            .await;

        let client = AnthropicClient::with_base_url("key".into(), server.uri());
        let err = client.send_message(&test_request()).await.unwrap_err();
        match err {
            AnthropicError::RateLimited { retry_after_ms } => {
                assert_eq!(retry_after_ms, 5000);
            }
            other => panic!("Expected RateLimited, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn rate_limit_429_without_header() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let client = AnthropicClient::with_base_url("key".into(), server.uri());
        let err = client.send_message(&test_request()).await.unwrap_err();
        match err {
            AnthropicError::RateLimited { retry_after_ms } => {
                assert_eq!(retry_after_ms, 1000); // fallback padrão
            }
            other => panic!("Expected RateLimited, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn api_error_500() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let client = AnthropicClient::with_base_url("key".into(), server.uri());
        let err = client.send_message(&test_request()).await.unwrap_err();
        match err {
            AnthropicError::ApiError { status, message } => {
                assert_eq!(status, 500);
                assert!(message.contains("Internal Server Error"));
            }
            other => panic!("Expected ApiError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn client_sends_correct_headers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("x-api-key", "my-secret-key"))
            .and(header("anthropic-version", "2023-06-01"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg_456",
                "content": [{"type": "text", "text": "ok"}],
                "model": "test",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 1, "output_tokens": 1}
            })))
            .mount(&server)
            .await;

        let client = AnthropicClient::with_base_url("my-secret-key".into(), server.uri());
        let resp = client.send_message(&test_request()).await.unwrap();
        assert_eq!(resp.id, "msg_456");
    }
}
