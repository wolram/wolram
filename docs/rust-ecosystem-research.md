# WOLRAM — Rust Ecosystem Research

## Summary

No official Anthropic Rust SDK exists. We'll use raw reqwest HTTP calls to the Anthropic Messages API. Below is the full crate evaluation for every component of the stack.

---

## 1. CLI Framework: **clap** (with derive macros)

- **structopt is deprecated** — it was merged into clap v3+
- **argh** follows Fuchsia conventions, not Unix — skip
- **clap** is the industry standard with `#[derive(Parser)]` and `#[derive(Subcommand)]`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wolram")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
```

**Verdict: clap latest with `derive` feature**

---

## 2. Async Runtime: **tokio**

- **async-std was officially discontinued in March 2025**
- **tokio** is used by 60%+ of async Rust projects
- Multi-threaded work-stealing scheduler
- All major async crates (reqwest, etc.) are built on tokio

**Verdict: tokio with `full` feature — the only real option**

---

## 3. HTTP Client: **reqwest**

| Crate | Async | Notes |
|-------|-------|-------|
| **reqwest** | Yes | Built on tokio+hyper, 250M+ downloads, most features |
| ureq | No (sync only) | Minimal, good for scripts, not for us |
| surf | Yes | Cross-runtime, less ecosystem support |

**Verdict: reqwest with `json` feature**

---

## 4. Terminal UI: **indicatif** (+ **console** for colors)

| Crate | Purpose | Stars |
|-------|---------|-------|
| **indicatif** | Progress bars, spinners | Perfect for job execution feedback |
| **ratatui** | Full TUI framework (widgets, layouts) | Overkill for Phase 1 — consider for Phase 2 web-like dashboard |
| **console** | Colors, styles, terminal utilities | Lightweight, pairs well with indicatif |

**Verdict: indicatif + console for Phase 1. Ratatui if we build interactive dashboard later.**

---

## 5. State Machine: **Rust enums + pattern matching**

Options evaluated:
- **rust-fsm** (v0.8+): DSL-based, can generate Mermaid diagrams, async support
- **fsm**: Simpler, minimal
- **finny**: Procedural macro, builder-style

For WOLRAM's 4 states (INIT → DEFINE_AGENT → PROCESS → END), a simple enum with pattern matching is cleaner and more idiomatic than pulling in a crate:

```rust
#[derive(Debug, Clone, Copy)]
pub enum State {
    Init,
    DefineAgent,
    Process,
    End,
}

pub enum Transition {
    Next(State),
    Retry { state: State, reason: FailureKind },
    Complete(JobOutcome),
}
```

**Verdict: Hand-rolled enums. rust-fsm if complexity grows.**

---

## 6. Git Operations: **git2** (libgit2 bindings)

| Crate | Type | Status |
|-------|------|--------|
| **git2** | C bindings (libgit2) | Mature, feature-complete, production-ready |
| **gitoxide** | Pure Rust | Not yet feature-complete (as of 2025) |
| Shell out to `git` | Process::Command | Fragile, injection risk, requires git installed |

**Why git2:**
- No dependency on git CLI being installed
- Programmatic access to commits, branches, diffs
- Bundles libgit2 source — just works with `cargo build`
- Better error handling than shelling out

**Verdict: git2 v0.19**

---

## 7. Serialization: **serde + serde_json + toml**

This is the universal Rust standard. No alternatives worth considering.

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
```

- **serde_json**: for Anthropic API requests/responses and audit records
- **toml**: for `wolram.toml` configuration file

**Verdict: serde ecosystem, standard versions**

---

## 8. Error Handling: **anyhow** + **thiserror**

| Crate | Use Case | Pattern |
|-------|----------|---------|
| **anyhow** | Application-level errors | `fn main() -> anyhow::Result<()>` |
| **thiserror** | Custom error types | `#[derive(Error)]` for WolramError, AnthropicError |
| eyre | Enhanced error reporting | Overkill for CLI |

**Pattern for WOLRAM:**
- `thiserror` to define structured error types (needed for retry logic to distinguish Business vs System failures)
- `anyhow` at the top-level boundary for ergonomic `?` chains

```rust
// Internal: structured
#[derive(thiserror::Error, Debug)]
pub enum AnthropicError {
    #[error("Rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
    #[error("API error {status}: {message}")]
    ApiError { status: u16, message: String },
}

// Boundary: ergonomic
fn main() -> anyhow::Result<()> { ... }
```

**Verdict: anyhow + thiserror together**

---

## 9. Anthropic/Claude Rust SDK: **NONE (official)**

### Critical Finding
**There is no official Anthropic Rust SDK.** Community options exist but are unofficial:

| Crate | Notes |
|-------|-------|
| `anthropic-sdk-rust` | Most feature-complete community SDK |
| `anthropic-rs` | Async support, straightforward API |
| `claude-sdk-rs` | Type-safe wrapper |
| `clust` | Lightweight/minimal |
| `claude-agent-sdk` | Agent-specific |

### Our Approach: Raw reqwest
For WOLRAM, we'll build a thin wrapper around reqwest directly:

```rust
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicClient {
    pub async fn send_message(&self, req: MessagesRequest) -> Result<MessagesResponse> {
        let resp = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&req)
            .send()
            .await?;
        // handle response...
    }
}
```

**Why raw HTTP over community SDK:**
- Full control over API calls
- No third-party dependency risk
- ~80 lines of code total
- Easy to debug
- If Anthropic publishes official SDK, easy to migrate

---

## Final Cargo.toml Dependencies

```toml
[package]
name = "wolram"
version = "0.1.0"
edition = "2021"
description = "Enterprise-grade AI development orchestrator"

[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
anyhow = "1"
thiserror = "2"
git2 = "0.19"
indicatif = "0.17"
console = "0.15"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
```

---

## Stack Summary

| Category | Choice | Why |
|----------|--------|-----|
| CLI | clap (derive) | Industry standard, structopt deprecated |
| Async | tokio (full) | Only viable option, async-std discontinued |
| HTTP | reqwest (json) | Built on tokio, most features, 250M+ downloads |
| Terminal | indicatif + console | Progress bars + colors, ratatui later if needed |
| State Machine | Rust enums | Simpler than crate for 4 states |
| Git | git2 | Mature, no CLI dependency, programmatic |
| Serialization | serde + serde_json + toml | Universal standard |
| Errors | anyhow + thiserror | Ergonomic + structured |
| Anthropic API | Raw reqwest | No official SDK, full control |
