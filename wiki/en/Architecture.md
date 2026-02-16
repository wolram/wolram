# Architecture

## Overview

WOLRAM follows a modular architecture organized around a central state machine. The system is designed to orchestrate AI-assisted development tasks with reliability guarantees inspired by UiPath's Robotic Enterprise Framework (REFramework).

## System Diagram

```
┌─────────────────────────────────────────────────────┐
│                      CLI (cli.rs)                   │
│         run | demo | status | global flags          │
└─────────────────────┬───────────────────────────────┘
                      │
              ┌───────▼────────┐
              │  Orchestrator  │
              │ (orchestrator) │
              └──┬──────┬──┬──┘
                 │      │  │
     ┌───────────┘      │  └───────────┐
     │                  │              │
┌────▼─────┐   ┌───────▼──────┐  ┌────▼────┐
│  Router  │   │ State Machine│  │Anthropic│
│(router)  │   │(state_machine)│ │ Client  │
└──────────┘   └──────────────┘  └─────────┘
     │                                 │
     │              ┌──────────────────┘
     │              │
┌────▼─────┐  ┌────▼────┐
│  Git     │  │   UI    │
│ (git)    │  │  (ui)   │
└──────────┘  └─────────┘
```

## Module Map

| Module | File(s) | Responsibility |
|--------|---------|----------------|
| `state_machine` | `src/state_machine/mod.rs`, `job.rs`, `state.rs` | Core types: `Job`, `State`, `Transition`, `StateMachine`, `AuditRecord`, `ModelTier`, `RetryConfig`, `FailureKind` |
| `cli` | `src/cli.rs` | clap-based CLI: `run`, `demo`, `status` subcommands with global flags |
| `config` | `src/config.rs` | `WolramConfig` loaded from `wolram.toml` with environment variable overrides |
| `orchestrator` | `src/orchestrator.rs` | `JobOrchestrator::run_job()` — drives a job through all 4 states |
| `router` | `src/router.rs` | `SkillRouter` (keyword/LLM-based skill assignment) and `ModelSelector` (complexity-based tier selection) |
| `anthropic` | `src/anthropic/mod.rs`, `client.rs`, `types.rs`, `error.rs` | HTTP client for the Anthropic Messages API |
| `git` | `src/git.rs` | `GitManager`: commit, branch creation via libgit2 |
| `ui` | `src/ui.rs` | `JobProgress`: spinner + colored output via indicatif/console |

## Data Flow

1. **CLI** parses arguments and loads configuration
2. **Orchestrator** creates a `Job` and begins the state machine loop
3. **Router** assigns a skill and model tier (via keywords or LLM classification)
4. **State Machine** computes transitions based on outcomes
5. **Anthropic Client** sends prompts and receives responses (or stubs in offline mode)
6. **Git Manager** commits results on successful completion
7. **UI** provides real-time feedback throughout the process
8. **Audit Record** is produced at the end of every job

## Dependencies

| Crate | Role |
|-------|------|
| `tokio` | Async runtime |
| `clap` | CLI argument parsing |
| `serde` + `serde_json` | Serialization |
| `toml` | Config file parsing |
| `reqwest` | HTTP client for Anthropic API |
| `git2` | libgit2 bindings |
| `anyhow` | Error propagation |
| `thiserror` | Error derive for `AnthropicError` |
| `chrono` | Timestamps |
| `uuid` | Job ID generation |
| `indicatif` | Terminal progress spinner |
| `console` | Colored terminal output |

### Dev Dependencies

| Crate | Role |
|-------|------|
| `tempfile` | Temporary directories for git tests |
| `wiremock` | HTTP mock server for client tests |

---

> *[Versão em Português](../pt-br/Architecture.md)*
