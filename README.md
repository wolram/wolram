# WOLRAM

Enterprise-grade orchestration for AI-assisted development.

WOLRAM applies battle-tested enterprise automation patterns (inspired by UiPath's REFramework) to AI coding workflows — bringing state machine governance, retry logic, model routing, and full audit trails to LLM-powered development.

## Architecture

```
Brainstorm / Reasoning
        |
        v
  TODO Generation (10-100 items)
        |
        v
┌───────────────────────────────────────────┐
│         JOB STATE MACHINE (per task)      │
│                                           │
│  INIT ──> DEFINE AGENT ──> PROCESS ──> END│
│   │          │               │          │ │
│   │     Skill Router    Execute Task  Log │
│   │     Model Selector  (w/ retries)  Git │
│   │                         │             │
│   │                   ┌─────┴─────┐       │
│   │                Success    Failure     │
│   │                   │    (Biz/System)   │
│   │                 Commit  Retry <= Max? │
│   │                   │     Y: Re-queue   │
│   │                   │     N: Log fail   │
└───┴───────────────────┴─────────┴─────────┘
        |
        v
  GIT INTEGRATION
  - Job completed --> commit (with summary)
  - Pool of jobs (e.g. hero page, login) --> branch
  - Full build --> PR with audit trail
```

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Job** | A single task extracted from the TODO list |
| **State Machine** | Each job passes through INIT > DEFINE AGENT > PROCESS > END |
| **Skill Router** | Assigns the right agent/skill using weighted keyword scoring or LLM-based classification |
| **Model Selector** | Picks the most cost-effective model using keyword + heuristic scoring (Haiku for simple, Sonnet for medium, Opus for complex) |
| **LLM Classification** | Optional Haiku pre-classification call that routes jobs by skill and complexity, with automatic fallback to keyword scoring |
| **Model Override** | CLI `--model` flag to force a specific model tier, overriding both LLM and keyword-based selection |
| **Business Failure** | Task logic failed (wrong output, validation error) — retryable |
| **System Failure** | Infrastructure failed (API timeout, rate limit) — retryable |
| **Audit Trail** | Every job logs: timestamp, model, skill, status, retry count, cost |

## Quick Start

```bash
cargo build                        # Compile
cargo run -- demo                  # Run the built-in state machine demo
cargo run -- run "implement X"     # Run a single job (stub mode without API key)
cargo run -- run --model opus "fix the typo"  # Force Opus model
cargo test                         # Run all tests (80 tests)
```

Set `ANTHROPIC_API_KEY` to enable real API calls and LLM-based classification; otherwise jobs run in stub mode.

## Routing

WOLRAM uses a layered routing strategy during the DEFINE_AGENT phase:

1. **LLM classification** (if API key is set) — sends a Haiku call to classify the job into a skill and complexity level
2. **Weighted keyword scoring** (fallback) — sums weights of matched keywords to pick the best skill and model tier
3. **CLI override** (`--model`) — always takes precedence for model selection

### Skill routing keywords

| Keyword | Skill | Weight |
|---------|-------|--------|
| test, spec | testing | 10, 5 |
| refactor, clean up | refactoring | 10, 5 |
| doc, readme | documentation | 10, 5 |
| fix, bug, debug, error | bug_fix | 10, 10, 7, 5 |
| implement, add, create, build | code_generation | 5, 3, 5, 5 |

### Model selection heuristics

- **Simple keywords** (rename, format, typo, delete, remove, update) push toward Haiku
- **Complex keywords** (architect, refactor, redesign, migrate, multi-file, system, overhaul) push toward Opus
- Short descriptions (<20 chars) boost simple score; long descriptions (>100 chars) and high word count (>15 words) boost complex score
- Default is Sonnet when scores are inconclusive

## Planned Features

- [x] CLI interface (terminal-first)
- [ ] TODO generation from natural language prompts
- [x] 4-stage state machine with configurable retry logic
- [x] Intelligent skill/model routing per job
- [x] Git integration (auto-commit, branching)
- [x] Audit trail with timestamp, model, cost tracking
- [x] LLM-based job classification with keyword fallback
- [x] CLI model override (`--model` flag)
- [x] CI/CD pipeline via GitHub Actions
- [ ] Web interface (wolram.com.br)

## Tech Stack

- **Rust** (edition 2024) — core runtime
- **Anthropic API** — Claude models for task execution and job classification
- **git2** — programmatic git operations
- **clap** — CLI argument parsing
- **tokio** — async runtime
- **indicatif/console** — terminal UI with spinners and colored output

## Status

**v0.1.0** — Core state machine, CLI, Anthropic HTTP client, intelligent skill/model routing, git integration, and terminal UI are implemented.

## Author

**Marlow Sousa** — [@wolram](https://github.com/wolram) | [wolram.com.br](https://wolram.com.br)
