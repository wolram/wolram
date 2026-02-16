# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WOLRAM is an enterprise-grade orchestration layer for AI-assisted development, written in Rust (edition 2024). It applies REFramework-style state machine patterns (inspired by UiPath's Robotic Enterprise Framework) to LLM coding workflows, with intelligent model/skill routing, retry logic, and full audit trails.

**Status**: Early development (v0.1.0). Core state machine is implemented. CLI, HTTP client, and git integration are planned.

## Build Commands

```bash
cargo build                        # Compile
cargo run                          # Run the demo
cargo test                         # Run all tests
cargo test <test_name>             # Run a single test (e.g. cargo test happy_path_walks_all_states)
cargo test --lib state_machine     # Run tests in a specific module
cargo clippy                       # Lint
cargo fmt                          # Format
```

## Architecture

The codebase lives in `src/state_machine/` with two files:

- **`job.rs`** — Core data types: `Job`, `ModelTier`, `AgentConfig`, `RetryConfig`, `AuditRecord`, `FailureKind`, `JobOutcome`, `JobStatus`
- **`state.rs`** — State machine driver: `State` enum (Init → DefineAgent → Process → End), `Transition` enum, and `StateMachine::next()` which computes transitions and mutates the job

### State Machine Flow

Each job flows: **INIT → DEFINE_AGENT → PROCESS → END**. At each state, `StateMachine::next(job, outcome)` either advances to the next state, retries on failure (up to `max_retries` with exponential backoff), or completes terminally. The `Job` struct tracks `state_history` for the audit trail.

### Failure Model

Two failure kinds (`FailureKind::Business` and `FailureKind::System`) — both retryable. After exceeding `max_retries`, the job is marked `Failed`. Retry count is tracked on the `Job`, not reset between states.

### Serialization

All public types derive `Serialize`/`Deserialize`. `AuditRecord::from_job()` produces a JSON-serializable snapshot of a completed job including cost tracking via `ModelTier`.

## Development Conventions

- **Conventional commits**: `feat:`, `fix:`, `docs:`, etc.
- **Tests**: Inline `#[cfg(test)]` modules within source files
- **All public types** must derive `Serialize`/`Deserialize`
- **Every job execution** must produce an `AuditRecord`
- **Default branch**: `main`
