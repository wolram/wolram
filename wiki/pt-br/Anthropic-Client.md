# Cliente Anthropic

O módulo `anthropic` fornece um cliente HTTP para a API de Mensagens da Anthropic.

## Arquitetura

```
anthropic/
├── mod.rs      # Trait MessageSender + re-exportações
├── client.rs   # AnthropicClient (HTTP via reqwest)
├── types.rs    # Tipos de Request/Response
└── error.rs    # AnthropicError (thiserror)
```

## Trait MessageSender

A abstração central para envio de mensagens, habilitando tanto chamadas reais à API quanto mocking em testes:

```rust
pub trait MessageSender: Send + Sync {
    fn send_message(&self, req: &MessagesRequest)
        -> impl Future<Output = Result<MessagesResponse, AnthropicError>> + Send;
}
```

## AnthropicClient

A implementação real do cliente HTTP usando `reqwest`.

### Construção

```rust
// Cliente padrão
let client = AnthropicClient::new("sk-ant-...".to_string());

// Com URL base personalizada (para testes)
let client = AnthropicClient::with_base_url(
    "sk-ant-...".to_string(),
    "http://localhost:8080".to_string()
);
```

### Detalhes HTTP

- **Endpoint**: `POST {base_url}/v1/messages`
- **URL base padrão**: `https://api.anthropic.com`
- **Headers**:
  - `x-api-key: {api_key}`
  - `anthropic-version: 2023-06-01`
  - `content-type: application/json`
- **Timeouts**:
  - Conexão: 10 segundos
  - Requisição: 120 segundos

### Mapeamento de Modelos

O orquestrador mapeia `ModelTier` para strings de modelo da API:

| ModelTier | String do Modelo na API |
|-----------|-------------------------|
| `Haiku` | `claude-haiku-4-5-20251001` |
| `Sonnet` | `claude-sonnet-4-5-20250929` |
| `Opus` | `claude-opus-4-6` |

## Tipos de Request/Response

### MessagesRequest

```rust
pub struct MessagesRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
}

pub struct Message {
    pub role: String,     // "user" ou "assistant"
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
    pub content_type: String,  // serializado como "type"
    pub text: String,
}

pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

## Tratamento de Erros

```rust
pub enum AnthropicError {
    RateLimited { retry_after_ms: u64 },
    ApiError { status: u16, message: String },
    NetworkError(#[from] reqwest::Error),
}
```

### Comportamento de Erros

| Status HTTP | Tipo de Erro | Comportamento |
|-------------|-------------|---------------|
| 429 | `RateLimited` | Extrai header `retry-after`; padrão de 1000ms se ausente |
| 4xx/5xx | `ApiError` | Captura código de status e corpo da resposta |
| Falha de rede | `NetworkError` | `reqwest::Error` encapsulado |

No orquestrador, todos os erros são mapeados para `FailureKind::System`, que aciona o loop de retry com backoff exponencial.

## Uso no Orquestrador

O orquestrador constrói requisições com um prompt de sistema adaptado à habilidade atribuída:

```
You are an expert software developer. Complete the following task: {description}
```

O texto da resposta é armazenado em `job.llm_response`.

---

> *[English Version](../en/Anthropic-Client.md)*
