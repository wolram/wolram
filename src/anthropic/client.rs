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
                assert_eq!(retry_after_ms, 1000); // default fallback
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
