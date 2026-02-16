pub mod client;
pub mod error;
pub mod types;

pub use client::AnthropicClient;
pub use error::AnthropicError;
pub use types::{Message, MessagesRequest, MessagesResponse, Usage};

/// Trait for sending messages to the Anthropic API (or a mock).
pub trait MessageSender: Send + Sync {
    fn send_message(
        &self,
        req: &MessagesRequest,
    ) -> impl std::future::Future<Output = Result<MessagesResponse, AnthropicError>> + Send;
}

impl MessageSender for AnthropicClient {
    async fn send_message(
        &self,
        req: &MessagesRequest,
    ) -> Result<MessagesResponse, AnthropicError> {
        self.send_message(req).await
    }
}
