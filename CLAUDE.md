# WOLRAM - Enterprise AI Development Orchestrator

## Project Overview

WOLRAM is an enterprise-grade orchestration layer for AI-assisted development, written in Rust. It applies REFramework-style state machine patterns (inspired by UiPath's Robotic Enterprise Framework) to LLM coding workflows, with intelligent model/skill routing, retry logic, git integration, and full audit trails.

**Status**: Early development (v0.1.0). Core state machine is implemented with retry logic, model tier cost tracking, and JSON audit records. CLI, HTTP client, and git integration are planned.

## Tech Stack

- **Language**: Rust (edition 2024)
- **Build system**: Cargo
- **Dependencies**: serde/serde_json (serialization), anyhow/thiserror (errors), chrono (timestamps), uuid (job IDs)
- **Planned additions**: clap (CLI), tokio (async), reqwest (HTTP), git2 (git), indicatif/console (terminal UI)

## Build Commands

```bash
cargo build          # Compile the project
cargo run            # Build and run the demo
cargo check          # Fast compile check (no binary output)
cargo test           # Run all tests (16 unit tests)
cargo clippy         # Run linter
cargo fmt            # Auto-format code
cargo doc --open     # Generate and view documentation
```

## Project Structure

```
wolram/
├── src/
│   ├── main.rs                    # Entry point: demo of the job state machine
│   └── state_machine/
│       ├── mod.rs                 # Module re-exports
│       ├── job.rs                 # Job, ModelTier, AgentConfig, RetryConfig, AuditRecord
│       └── state.rs               # State enum, Transition enum, StateMachine driver
├── docs/
│   ├── architecture.md            # Detailed state machine + git integration spec
│   ├── rust-class.md              # Developer onboarding: Rust patterns + cargo commands
│   └── rust-ecosystem-research.md # Crate evaluation for dependencies
├── .github/workflows/
│   ├── claude.yml                 # Claude PR assistant (@claude mentions)
│   └── claude-code-review.yml     # Automated Claude code review on PRs
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## Architecture

### State Machine

Each job flows through four states: **INIT → DEFINE_AGENT → PROCESS → END**

- **INIT**: Parse job from TODO queue, validate inputs, set up execution context
- **DEFINE_AGENT**: Skill Router assigns agent type; Model Selector picks cost-optimal model tier
- **PROCESS**: Execute the job, handle success/failure, retry if needed
- **END**: Generate audit record, create git commit (on success), log results

### Key Types (in `src/state_machine/`)

| Type | File | Purpose |
|------|------|---------|
| `State` | state.rs | Enum: Init, DefineAgent, Process, End |
| `Transition` | state.rs | Enum: Next(State), Retry, Complete |
| `StateMachine` | state.rs | Drives jobs through state transitions |
| `Job` | job.rs | A single task with lifecycle tracking |
| `ModelTier` | job.rs | Haiku ($0.001), Sonnet ($0.005), Opus ($0.05) |
| `AgentConfig` | job.rs | Skill name + model tier assignment |
| `FailureKind` | job.rs | Business (logic) vs System (infrastructure) failures |
| `JobOutcome` | job.rs | Success or Failure(FailureKind) |
| `RetryConfig` | job.rs | Max retries (default 3) + exponential backoff |
| `AuditRecord` | job.rs | Serializable snapshot of a completed job |

### Failure Handling

- **Business Failure**: Task logic failed (wrong output, validation error, tests fail) — retryable
- **System Failure**: Infrastructure failed (API timeout, rate limit, network error) — retryable
- Both types retry up to `max_retries` (default 3) with exponential backoff
- After max retries exceeded, the job is marked as Failed

### Git Integration Pattern

- **Job completed** → git commit (conventional commit format)
- **Pool of related jobs** → git branch
- **Full build complete** → Pull Request with audit trail summary

## Development Conventions

- **Conventional commits**: `feat:`, `fix:`, `docs:`, etc. with descriptive body
- **Tests**: Inline `#[cfg(test)]` modules within source files (idiomatic Rust)
- **State machine transitions** must be explicit and logged
- **Every job execution** produces a serializable audit record
- **Architecture decisions** documented in `/docs/`
- **All public types** derive Serialize/Deserialize for JSON audit records
- **Default branch**: `master`

## Testing

All tests are standard Rust unit tests using `assert_eq!`, `assert!`, and `matches!` macros. No external test framework.

Run tests with:

```bash
cargo test
```

Test files and coverage:
- `src/state_machine/job.rs` — 9 tests: job creation, retry config, audit records, serialization roundtrip, model tiers, agent assignment
- `src/state_machine/state.rs` — 8 tests: happy path transitions, retry/failure behavior, edge cases, state history tracking

## CI/CD

Two GitHub Actions workflows:
- **Claude PR Assistant** (`claude.yml`): Responds to `@claude` mentions in issues/PRs
- **Claude Code Review** (`claude-code-review.yml`): Automated review on every PR

No build/test CI pipeline yet — this is a planned addition.
