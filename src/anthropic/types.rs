use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
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
