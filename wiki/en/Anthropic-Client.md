# Anthropic Client

The `anthropic` module provides an HTTP client for the Anthropic Messages API.

## Architecture

```
anthropic/
├── mod.rs      # MessageSender trait + re-exports
├── client.rs   # AnthropicClient (reqwest HTTP)
├── types.rs    # Request/Response types
└── error.rs    # AnthropicError (thiserror)
```

## MessageSender Trait

The core abstraction for sending messages, enabling both real API calls and test mocking:

```rust
pub trait MessageSender: Send + Sync {
    fn send_message(&self, req: &MessagesRequest)
        -> impl Future<Output = Result<MessagesResponse, AnthropicError>> + Send;
}
```

## AnthropicClient

The real HTTP client implementation using `reqwest`.

### Construction

```rust
// Standard client
let client = AnthropicClient::new("sk-ant-...".to_string());

// With custom base URL (for testing)
let client = AnthropicClient::with_base_url(
    "sk-ant-...".to_string(),
    "http://localhost:8080".to_string()
);
```

### HTTP Details

- **Endpoint**: `POST {base_url}/v1/messages`
- **Default base URL**: `https://api.anthropic.com`
- **Headers**:
  - `x-api-key: {api_key}`
  - `anthropic-version: 2023-06-01`
  - `content-type: application/json`
- **Timeouts**:
  - Connect: 10 seconds
  - Request: 120 seconds

### Model Mapping

The orchestrator maps `ModelTier` to API model strings:

| ModelTier | API Model String |
|-----------|------------------|
| `Haiku` | `claude-haiku-4-5-20251001` |
| `Sonnet` | `claude-sonnet-4-5-20250929` |
| `Opus` | `claude-opus-4-6` |

## Request/Response Types

### MessagesRequest

```rust
pub struct MessagesRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
}

pub struct Message {
    pub role: String,     // "user" or "assistant"
    pub content: String,
}
```

### MessagesResponse

```rust
pub struct MessagesResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

pub struct ContentBlock {
    pub content_type: String,  // serialized as "type"
    pub text: String,
}

pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

## Error Handling

```rust
pub enum AnthropicError {
    RateLimited { retry_after_ms: u64 },
    ApiError { status: u16, message: String },
    NetworkError(#[from] reqwest::Error),
}
```

### Error Behavior

| HTTP Status | Error Type | Behavior |
|-------------|-----------|----------|
| 429 | `RateLimited` | Extracts `retry-after` header; defaults to 1000ms if absent |
| 4xx/5xx | `ApiError` | Captures status code and response body |
| Network failure | `NetworkError` | Wrapped `reqwest::Error` |

In the orchestrator, all errors are mapped to `FailureKind::System`, which triggers the retry loop with exponential backoff.

## Usage in the Orchestrator

The orchestrator constructs requests with a system prompt tailored to the assigned skill:

```
You are an expert software developer. Complete the following task: {description}
```

The response text is stored in `job.llm_response`.

---

> *[Versão em Português](../pt-br/Anthropic-Client.md)*
