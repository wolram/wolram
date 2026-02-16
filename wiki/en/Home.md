# WOLRAM Wiki

**WOLRAM** is an enterprise-grade orchestration layer for AI-assisted development, written in Rust (edition 2024). It applies REFramework-style state machine patterns — inspired by UiPath's Robotic Enterprise Framework — to LLM coding workflows, with intelligent model/skill routing, retry logic, and full audit trails.

## Current Status

**v0.1.0** — Core state machine, CLI, Anthropic HTTP client, skill router, git integration, and terminal UI are implemented. The Anthropic client works when `ANTHROPIC_API_KEY` is set; otherwise jobs run in stub mode (simulated success).

## Key Features

- **State Machine Orchestration** — Every job flows through a well-defined lifecycle: INIT → DEFINE_AGENT → PROCESS → END
- **Intelligent Routing** — Automatic skill assignment and model tier selection based on task complexity
- **Retry with Exponential Backoff** — Configurable retry logic for both business and system failures
- **Full Audit Trail** — Every job execution produces a structured `AuditRecord` with cost estimation, timing, and state transitions
- **Git Integration** — Automatic commit of job results via libgit2
- **Terminal UI** — Progress spinners and colored output for real-time feedback

## Wiki Pages

| Page | Description |
|------|-------------|
| [Getting Started](Getting-Started.md) | Installation, setup, and first run |
| [Architecture](Architecture.md) | High-level system design and module overview |
| [State Machine](State-Machine.md) | Core state machine types, transitions, and flow |
| [CLI Reference](CLI-Reference.md) | Commands, subcommands, and flags |
| [Configuration](Configuration.md) | `wolram.toml` and environment variables |
| [Anthropic Client](Anthropic-Client.md) | HTTP client for the Anthropic Messages API |
| [Router & Skills](Router-and-Skills.md) | Skill assignment and model tier selection |
| [Git Integration](Git-Integration.md) | Automatic commits and branch management |
| [Testing](Testing.md) | Test strategy, running tests, and writing new tests |
| [Contributing](Contributing.md) | Development conventions and contribution guidelines |

## Quick Links

- [GitHub Repository](https://github.com/wolram/wolram)
- [License: MIT](https://github.com/wolram/wolram/blob/main/LICENSE)

---

> *[Versão em Português](../pt-br/Home.md)*
