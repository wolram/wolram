# Testes

WOLRAM possui uma suíte de testes abrangente com ~80 testes cobrindo todos os módulos.

## Executando os Testes

```bash
# Executar todos os testes
cargo test

# Executar um teste específico por nome
cargo test happy_path_walks_all_states

# Executar testes em um módulo específico
cargo test --lib state_machine

# Executar testes com saída
cargo test -- --nocapture
```

## Organização dos Testes

Todos os testes são módulos `#[cfg(test)]` inline dentro de cada arquivo fonte — não há um diretório `tests/` separado.

## Estratégias de Teste

### Testes Unitários

Testes de funções puras para lógica determinística:

- `SkillRouter::route()` — correção da correspondência de palavras-chave
- `ModelSelector::select()` — classificação de complexidade
- `RetryConfig::delay_for_attempt()` — cálculo de backoff exponencial
- `StateMachine::next()` — regras de transição de estado
- Desserialização de configuração e valores padrão
- Formatação de exibição de erros
- Serialização/desserialização de tipos

### Testes com Cliente Mock

Tanto `router.rs` quanto `orchestrator.rs` definem structs locais `MockClient` implementando `MessageSender`:

```rust
struct MockClient {
    response: String,
}

impl MessageSender for MockClient {
    async fn send_message(&self, _req: &MessagesRequest)
        -> Result<MessagesResponse, AnthropicError> {
        // Retorna resposta hardcoded
    }
}
```

Isso permite testar caminhos de classificação LLM e orquestração sem chamadas reais à API.

### Testes com Servidor HTTP Mock

O módulo `client.rs` usa o crate `wiremock` para testar o cliente HTTP contra um servidor (mock) real:

- Monta matchers de resposta para métodos HTTP e headers específicos
- Testa respostas 200 bem-sucedidas
- Testa rate limiting 429 (com e sem header `retry-after`)
- Testa tratamento de erro 500 do servidor

### Testes Assíncronos

Todos os testes de orquestrador e cliente usam `#[tokio::test]` para execução assíncrona.

### Testes Git com Tempfile

Testes em `git.rs` usam `tempfile::TempDir` para criar repositórios Git descartáveis:

- Testa criação de commits
- Testa criação de branches
- Testa commits de resultados de jobs
- Limpeza automática quando o teste completa

### Testes de Validação CLI

Testes em `cli.rs` usam `Cli::parse_from()` para validar parsing de argumentos e `Cli::command().debug_assert()` para integridade da configuração clap.

### Testes de Roundtrip de Serialização

Verificam que tipos podem ser serializados e desserializados sem perda de dados:

- `Job`
- `MessagesRequest` / `MessagesResponse`
- `AuditRecord`

## Nomes de Testes Importantes

### Testes da Máquina de Estados

| Teste | O que Verifica |
|-------|----------------|
| `happy_path_walks_all_states` | Fluxo completo INIT→DEFINE_AGENT→PROCESS→END |
| `business_failure_retries_then_fails` | Falhas de negócio esgotam retries |
| `system_failure_retries_then_fails` | Falhas de sistema esgotam retries |
| `zero_retries_fails_immediately` | Zero retries = falha imediata |
| `retry_then_succeed` | Recuperação após retry |
| `state_history_is_recorded` | Completude da trilha de auditoria |

## Escrevendo Novos Testes

1. Adicione testes em um módulo `#[cfg(test)]` no final do arquivo fonte relevante
2. Use `MockClient` para qualquer teste que precise de respostas LLM
3. Use `tempfile::TempDir` para qualquer teste que precise de um repositório Git
4. Marque testes assíncronos com `#[tokio::test]`
5. Siga a convenção de nomenclatura: `snake_case` descrevendo o cenário

## Linting e Formatação

```bash
# Lint
cargo clippy

# Formatar
cargo fmt
```

---

> *[English Version](../en/Testing.md)*
