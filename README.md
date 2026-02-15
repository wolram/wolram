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
| **Skill Router** | Assigns the right agent/skill to each job during INIT |
| **Model Selector** | Picks the most cost-effective model for each job (haiku for simple, opus for complex) |
| **Business Failure** | Task logic failed (wrong output, validation error) — retryable |
| **System Failure** | Infrastructure failed (API timeout, rate limit) — retryable |
| **Audit Trail** | Every job logs: timestamp, model, skill, status, retry count, cost |

## Planned Features

- [ ] CLI interface (terminal-first)
- [ ] TODO generation from natural language prompts
- [x] 4-stage state machine with configurable retry logic
- [ ] Intelligent skill/model routing per job
- [ ] Git integration (auto-commit, branching, PR generation)
- [ ] Audit trail with timestamp, model, cost tracking
- [ ] CI/CD pipeline via GitHub Actions
- [ ] Web interface (wolram.com.br)

## Tech Stack

TBD — evaluating Rust, Go, and TypeScript (Bun).

## Status

**Pre-development** — Architecture defined, implementation starting soon.

## Author

**Marlow Sousa** — [@wolram](https://github.com/wolram) | [wolram.com.br](https://wolram.com.br)
