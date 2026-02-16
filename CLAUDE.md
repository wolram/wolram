# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WOLRAM is an enterprise-grade orchestration layer for AI-assisted development, written in Rust (edition 2024). It applies REFramework-style state machine patterns (inspired by UiPath's Robotic Enterprise Framework) to LLM coding workflows, with intelligent model/skill routing, retry logic, and full audit trails.

**Status**: v0.1.0. Core state machine, CLI, Anthropic HTTP client, skill router, git integration, and terminal UI are implemented. The Anthropic client works when `ANTHROPIC_API_KEY` is set; otherwise jobs run in stub mode (simulated success).

## Build Commands

```bash
cargo build                        # Compile
cargo run -- demo                  # Run the built-in state machine demo
cargo run -- run "implement X"     # Run a single job (stub mode without API key)
cargo test                         # Run all tests
cargo test <test_name>             # Run a single test (e.g. cargo test happy_path_walks_all_states)
cargo test --lib state_machine     # Run tests in a specific module
cargo clippy                       # Lint
cargo fmt                          # Format
```

## Architecture

### Modules

- **`state_machine/`** — Core types: `Job`, `State`, `Transition`, `StateMachine`, `AuditRecord`, `ModelTier`, `RetryConfig`, `FailureKind`, `JobOutcome`
- **`cli.rs`** — clap-based CLI: `run`, `demo`, `status` subcommands with global `--model`, `--max-retries`, `--verbose` flags
- **`config.rs`** — `WolramConfig` loaded from `wolram.toml` (falls back to defaults). `ANTHROPIC_API_KEY` env var overrides config file.
- **`anthropic/`** — HTTP client for the Anthropic Messages API: `AnthropicClient`, `MessagesRequest/Response`, `AnthropicError` (rate limiting, API errors)
- **`router.rs`** — `SkillRouter` (keyword-based skill assignment) and `ModelSelector` (complexity-based model tier selection)
- **`orchestrator.rs`** — `JobOrchestrator::run_job()` drives a job through all 4 states using the state machine, router, and optionally the Anthropic client
- **`git.rs`** — `GitManager`: commit, create branch, get current branch via git2
- **`ui.rs`** — `JobProgress`: spinner + colored output via indicatif/console

### State Machine Flow

Each job flows: **INIT → DEFINE_AGENT → PROCESS → END**. `StateMachine::next(job, outcome)` computes transitions and mutates the job. Failures retry up to `max_retries` with exponential backoff (`RetryConfig::delay_for_attempt`). The `Job` struct tracks `state_history` for the audit trail.

### Orchestrator

`JobOrchestrator::run_job()` is the main entry point. It:
1. Validates the job (INIT)
2. Uses `SkillRouter::route()` + `ModelSelector::select()` to assign agent config (DEFINE_AGENT)
3. Calls Anthropic API or simulates success (PROCESS), with retry loop
4. Produces `AuditRecord` (END)

### Failure Model

Two failure kinds (`FailureKind::Business` and `FailureKind::System`) — both retryable. Retry count is tracked on the `Job`, not reset between states.

## Development Conventions

- **Conventional commits**: `feat:`, `fix:`, `docs:`, etc.
- **Tests**: Inline `#[cfg(test)]` modules within source files
- **All public types** must derive `Serialize`/`Deserialize`
- **Every job execution** must produce an `AuditRecord`
- **Default branch**: `main`
