//! Módulo do cliente Anthropic — tipos, erros e trait de envio de mensagens.
//!
//! Fornece [`AnthropicClient`] para chamadas reais à API e o trait
//! [`MessageSender`] para permitir mocking em testes.

pub mod client;
pub mod error;
pub mod types;

pub use client::AnthropicClient;
pub use error::AnthropicError;
pub use types::{Message, MessagesRequest, MessagesResponse};

/// Trait para envio de mensagens à API Anthropic (ou a um mock).
///
/// Permite que o orquestrador e o roteador trabalhem com qualquer implementação
/// que saiba enviar uma [`MessagesRequest`] e devolver uma [`MessagesResponse`].
pub trait MessageSender: Send + Sync {
    fn send_message(
        &self,
        req: &MessagesRequest,
    ) -> impl std::future::Future<Output = Result<MessagesResponse, AnthropicError>> + Send;
}

/// Implementação de [`MessageSender`] que delega para o método real do [`AnthropicClient`].
impl MessageSender for AnthropicClient {
    async fn send_message(
        &self,
        req: &MessagesRequest,
    ) -> Result<MessagesResponse, AnthropicError> {
        self.send_message(req).await
    }
}
