# State Machine

The state machine is the heart of WOLRAM. Every job follows a deterministic lifecycle with four states, managed by the `StateMachine` struct.

## States

```rust
pub enum State {
    Init,         // Validate the job
    DefineAgent,  // Route skill + select model
    Process,      // Execute the LLM call
    End,          // Produce audit record
}
```

## Flow Diagram

```
    ┌──────┐
    │ Init │
    └──┬───┘
       │ Success
       ▼
┌─────────────┐
│ DefineAgent │
└──────┬──────┘
       │ Success
       ▼
  ┌─────────┐    Failure (retries left)
  │ Process │ ──────────────────────┐
  └────┬────┘                       │
       │ Success               ┌────▼────┐
       ▼                       │  Retry  │
    ┌─────┐                    │ (sleep) │
    │ End │                    └────┬────┘
    └─────┘                        │
                                   └──► back to Process
```

## Transitions

The `Transition` enum describes the result of evaluating a state:

```rust
pub enum Transition {
    Next(State),                               // Advance to next state
    Retry { state: State, reason: FailureKind }, // Retry current state
    Complete(JobOutcome),                       // Terminal state
}
```

`StateMachine::next(job, outcome)` is the core function. It:
1. Records the current state in `job.state_history`
2. Evaluates the outcome against the current state
3. Mutates the job (updates state, status, retry count)
4. Returns the appropriate `Transition`

## Transition Rules

| Current State | Outcome | Result |
|---------------|---------|--------|
| Init | Success | `Next(DefineAgent)` |
| Init | Failure | `Complete(Failure)` — no retries for validation |
| DefineAgent | Success | `Next(Process)` |
| DefineAgent | Failure | `Complete(Failure)` — no retries for routing |
| Process | Success | `Next(End)` |
| Process | Failure (retries left) | `Retry { state: Process, reason }` |
| Process | Failure (retries exhausted) | `Complete(Failure)` |
| End | Success | `Complete(Success)` |
| End | Failure | `Complete(Failure)` |

## Job Structure

```rust
pub struct Job {
    pub id: String,                    // UUID v4
    pub description: String,
    pub status: JobStatus,             // Pending | InProgress | Completed | Failed
    pub state: State,
    pub state_history: Vec<State>,     // Audit trail of visited states
    pub retry_count: u32,
    pub retry_config: RetryConfig,
    pub agent: Option<AgentConfig>,    // Assigned in DEFINE_AGENT
    pub llm_response: Option<String>,  // LLM output from PROCESS
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Failure Kinds

```rust
pub enum FailureKind {
    Business(String),  // Wrong output, validation failure
    System(String),    // API timeout, rate limit, network error
}
```

Both kinds are retryable during the PROCESS state. The retry count is tracked on the `Job` and is **not** reset between states.

## Retry Configuration

```rust
pub struct RetryConfig {
    pub max_retries: u32,    // Default: 3
    pub base_delay_ms: u64,  // Default: 1000
}
```

Delay calculation uses exponential backoff:

```
delay = base_delay_ms × 2^(attempt - 1)
```

| Attempt | Delay (base=1000ms) |
|---------|---------------------|
| 1 | 1,000ms |
| 2 | 2,000ms |
| 3 | 4,000ms |
| 4 | 8,000ms |

## Job Outcomes

```rust
pub enum JobOutcome {
    Success,
    Failure(FailureKind),
}
```

## Model Tiers

```rust
pub enum ModelTier {
    Haiku,   // Fast, low cost (~$0.001/job)
    Sonnet,  // Balanced (~$0.005/job)
    Opus,    // Most capable (~$0.05/job)
}
```

## Audit Record

Every completed job produces an `AuditRecord`:

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

> *[Versão em Português](../pt-br/State-Machine.md)*
