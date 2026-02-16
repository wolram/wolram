# Máquina de Estados

A máquina de estados é o coração do WOLRAM. Todo job segue um ciclo de vida determinístico com quatro estados, gerenciado pela struct `StateMachine`.

## Estados

```rust
pub enum State {
    Init,         // Validar o job
    DefineAgent,  // Rotear habilidade + selecionar modelo
    Process,      // Executar a chamada ao LLM
    End,          // Produzir registro de auditoria
}
```

## Diagrama de Fluxo

```
    ┌──────┐
    │ Init │
    └──┬───┘
       │ Sucesso
       ▼
┌─────────────┐
│ DefineAgent │
└──────┬──────┘
       │ Sucesso
       ▼
  ┌─────────┐    Falha (retries restantes)
  │ Process │ ──────────────────────┐
  └────┬────┘                       │
       │ Sucesso               ┌────▼────┐
       ▼                       │  Retry  │
    ┌─────┐                    │ (sleep) │
    │ End │                    └────┬────┘
    └─────┘                        │
                                   └──► volta ao Process
```

## Transições

O enum `Transition` descreve o resultado da avaliação de um estado:

```rust
pub enum Transition {
    Next(State),                               // Avançar para próximo estado
    Retry { state: State, reason: FailureKind }, // Retentar estado atual
    Complete(JobOutcome),                       // Estado terminal
}
```

`StateMachine::next(job, outcome)` é a função principal. Ela:
1. Registra o estado atual em `job.state_history`
2. Avalia o resultado contra o estado atual
3. Muta o job (atualiza estado, status, contador de retries)
4. Retorna a `Transition` apropriada

## Regras de Transição

| Estado Atual | Resultado | Ação |
|--------------|-----------|------|
| Init | Sucesso | `Next(DefineAgent)` |
| Init | Falha | `Complete(Failure)` — sem retries para validação |
| DefineAgent | Sucesso | `Next(Process)` |
| DefineAgent | Falha | `Complete(Failure)` — sem retries para roteamento |
| Process | Sucesso | `Next(End)` |
| Process | Falha (retries restantes) | `Retry { state: Process, reason }` |
| Process | Falha (retries esgotados) | `Complete(Failure)` |
| End | Sucesso | `Complete(Success)` |
| End | Falha | `Complete(Failure)` |

## Estrutura do Job

```rust
pub struct Job {
    pub id: String,                    // UUID v4
    pub description: String,
    pub status: JobStatus,             // Pending | InProgress | Completed | Failed
    pub state: State,
    pub state_history: Vec<State>,     // Trilha de auditoria dos estados visitados
    pub retry_count: u32,
    pub retry_config: RetryConfig,
    pub agent: Option<AgentConfig>,    // Atribuído em DEFINE_AGENT
    pub llm_response: Option<String>,  // Saída do LLM em PROCESS
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Tipos de Falha

```rust
pub enum FailureKind {
    Business(String),  // Saída incorreta, falha de validação
    System(String),    // Timeout de API, rate limit, erro de rede
}
```

Ambos os tipos são retentáveis durante o estado PROCESS. O contador de retries é rastreado no `Job` e **não** é resetado entre estados.

## Configuração de Retry

```rust
pub struct RetryConfig {
    pub max_retries: u32,    // Padrão: 3
    pub base_delay_ms: u64,  // Padrão: 1000
}
```

O cálculo do delay usa backoff exponencial:

```
delay = base_delay_ms × 2^(tentativa - 1)
```

| Tentativa | Delay (base=1000ms) |
|-----------|---------------------|
| 1 | 1.000ms |
| 2 | 2.000ms |
| 3 | 4.000ms |
| 4 | 8.000ms |

## Resultados de Job

```rust
pub enum JobOutcome {
    Success,
    Failure(FailureKind),
}
```

## Níveis de Modelo

```rust
pub enum ModelTier {
    Haiku,   // Rápido, baixo custo (~$0,001/job)
    Sonnet,  // Equilibrado (~$0,005/job)
    Opus,    // Mais capaz (~$0,05/job)
}
```

## Registro de Auditoria

Todo job completado produz um `AuditRecord`:

```rust
pub struct AuditRecord {
    pub job_id: String,
    pub description: String,
    pub status: JobStatus,
    pub state_transitions: Vec<State>,
    pub agent: Option<AgentConfig>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub cost_usd: f64,
    pub llm_response: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: i64,
}
```

---

> *[English Version](../en/State-Machine.md)*
