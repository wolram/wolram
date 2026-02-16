//! Tipos de dados para requisições e respostas da API Anthropic Messages.
//!
//! Todas as structs derivam `Serialize` e `Deserialize` para conversão JSON
//! conforme o formato esperado pelo endpoint `v1/messages` da Anthropic.

use serde::{Deserialize, Serialize};

/// Corpo da requisição para o endpoint `/v1/messages` da API Anthropic.
///
/// Contém o modelo desejado, o limite de tokens e a lista de mensagens
/// que compõem a conversa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesRequest {
    /// Identificador do modelo a ser usado (ex.: "claude-sonnet-4-5-20250929").
    pub model: String,
    /// Número máximo de tokens na resposta gerada pelo modelo.
    pub max_tokens: u32,
    /// Lista de mensagens compondo a conversa (usuário e assistente).
    pub messages: Vec<Message>,
}

/// Uma única mensagem em uma conversa com a API Anthropic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Papel do remetente: "user" ou "assistant".
    pub role: String,
    /// Conteúdo textual da mensagem.
    pub content: String,
}

/// Resposta retornada pelo endpoint `/v1/messages` da API Anthropic.
///
/// Contém o identificador único, os blocos de conteúdo gerados, informações
/// sobre o modelo utilizado e estatísticas de uso de tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesResponse {
    /// Identificador único da resposta (gerado pela API).
    pub id: String,
    /// Blocos de conteúdo na resposta (normalmente texto).
    pub content: Vec<ContentBlock>,
    /// Modelo que gerou a resposta.
    pub model: String,
    /// Motivo da parada da geração (ex.: "end_turn", "max_tokens").
    /// `None` se ainda em progresso.
    pub stop_reason: Option<String>,
    /// Estatísticas de uso de tokens (entrada e saída).
    pub usage: Usage,
}

/// Um bloco de conteúdo dentro da resposta — atualmente apenas texto.
///
/// O campo `content_type` é serializado como `"type"` no JSON via `serde(rename)`,
/// seguindo o formato da API da Anthropic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    /// Tipo do bloco ("text"). Serializado como "type" no JSON.
    #[serde(rename = "type")]
    pub content_type: String,
    /// Conteúdo textual deste bloco.
    pub text: String,
}

/// Estatísticas de consumo de tokens para uma chamada à API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Número de tokens consumidos na entrada (prompt).
    pub input_tokens: u32,
    /// Número de tokens gerados na saída (resposta).
    pub output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_request_roundtrip() {
        let req = MessagesRequest {
            model: "claude-sonnet-4-5-20250929".into(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".into(),
                content: "Hello".into(),
            }],
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: MessagesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model, "claude-sonnet-4-5-20250929");
        assert_eq!(parsed.max_tokens, 4096);
        assert_eq!(parsed.messages.len(), 1);
        assert_eq!(parsed.messages[0].role, "user");
        assert_eq!(parsed.messages[0].content, "Hello");
    }

    #[test]
    fn messages_response_roundtrip() {
        let resp = MessagesResponse {
            id: "msg_abc".into(),
            content: vec![ContentBlock {
                content_type: "text".into(),
                text: "World".into(),
            }],
            model: "claude-sonnet-4-5-20250929".into(),
            stop_reason: Some("end_turn".into()),
            usage: Usage {
                input_tokens: 10,
                output_tokens: 20,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: MessagesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "msg_abc");
        assert_eq!(parsed.content[0].text, "World");
        assert_eq!(parsed.stop_reason, Some("end_turn".into()));
        assert_eq!(parsed.usage.input_tokens, 10);
        assert_eq!(parsed.usage.output_tokens, 20);
    }

    #[test]
    fn content_block_type_field_renames_correctly() {
        let block = ContentBlock {
            content_type: "text".into(),
            text: "hello".into(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type""#));
        assert!(!json.contains("content_type"));
    }

    #[test]
    fn messages_response_deserialize_from_api_format() {
        let api_json = r#"{
            "id": "msg_123",
            "content": [{"type": "text", "text": "Response here"}],
            "model": "claude-sonnet-4-5-20250929",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 5, "output_tokens": 15}
        }"#;
        let resp: MessagesResponse = serde_json::from_str(api_json).unwrap();
        assert_eq!(resp.id, "msg_123");
        assert_eq!(resp.content[0].text, "Response here");
        assert_eq!(resp.content[0].content_type, "text");
    }

    #[test]
    fn messages_response_null_stop_reason() {
        let json = r#"{
            "id": "msg_456",
            "content": [],
            "model": "test",
            "stop_reason": null,
            "usage": {"input_tokens": 0, "output_tokens": 0}
        }"#;
        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.stop_reason, None);
    }
}
