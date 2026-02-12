# Rust Class Notes — WOLRAM Development

Notes from the planning session. Key concepts and Rust Book chapters to review before coding.

---

## Why tokio?

**async-std was officially discontinued in March 2025.** The Rust async ecosystem collapsed into a single winner:

- **tokio** = the async runtime that everything else is built on
- **reqwest** (our HTTP client) requires tokio under the hood
- If you use reqwest, you're already using tokio whether you declare it or not

Think of it like this: tokio is to async Rust what Node.js's event loop is to JavaScript. You don't choose it — it's the floor you stand on.

For WOLRAM in Phase 1, the only async thing we do is HTTP calls to the Anthropic API. The rest is synchronous. Tokio is just there to run those `await` calls.

---

## Why raw reqwest instead of an Anthropic SDK?

**There is no official Anthropic Rust SDK.** Community options exist but are all unofficial:

| Crate | Notes |
|-------|-------|
| `anthropic-sdk-rust` | Most feature-complete, few hundred stars |
| `anthropic-rs` | Async, straightforward |
| `claude-sdk-rs` | Type-safe wrapper |
| `clust` | Lightweight |

### Why we chose raw HTTP:

1. **They're all unofficial and small.** If the maintainer stops updating, you're stuck when Anthropic changes their API.
2. **WOLRAM needs fine-grained control.** Token counts for cost calculation, custom retry logic, streaming — with a wrapper, you control it all.
3. **It's only ~80 lines of code.** The Anthropic Messages API is one endpoint: `POST /v1/messages` with a JSON body. Two headers (`x-api-key`, `anthropic-version`).
4. **Portfolio signal.** Showing you understand the API at HTTP level is more impressive than importing a crate.

The API client is isolated in `src/anthropic/client.rs` — swap internals anytime if an official SDK appears.

---

## Architecture Decisions — Why Each One Matters

| Decision | Choice | Why |
|----------|--------|-----|
| API Client | Raw reqwest | WOLRAM is an orchestrator — your most critical path can't depend on someone else's abstraction |
| Git | git2 (libgit2) | Every shell `git` call is untyped, unstructured. Enterprise tools need governed git operations |
| State Machine | Rust enums | The state machine IS the product. You should own every line of your core IP |
| State Storage | JSON files | Audit trail should be git-committable. SQLite is opaque. JSON is inspectable, diffable, portable |
| Async | tokio | Ecosystem decided for us. No point fighting gravity |

---

## Rust Book — Essential Chapters for WOLRAM

### Must-Read Before Coding:
- **Chapter 3**: Common Programming Concepts — variables, types, functions, control flow
- **Chapter 4**: Understanding Ownership — THE Rust concept, borrow checker lives here
- **Chapter 5**: Using Structs — you'll define Job, Config, AuditRecord as structs
- **Chapter 6**: Enums and Pattern Matching — your entire state machine is enums + match
- **Chapter 9**: Error Handling — Result<T, E>, the ? operator, anyhow/thiserror patterns

### Read When You Hit Them:
- **Chapter 7**: Packages, Crates, and Modules — how src/anthropic/mod.rs works
- **Chapter 8**: Common Collections — Vec, HashMap, String (used everywhere)
- **Chapter 10**: Generic Types, Traits, and Lifetimes — serde derives use these
- **Chapter 11**: Writing Automated Tests — for cargo test
- **Chapter 13**: Closures and Iterators — .map(), .filter(), .collect() patterns

### Reference When Needed:
- **Chapter 17**: Async Programming — tokio, async/await (Rust Book added this recently)
- **Chapter 15**: Smart Pointers — Box, Rc, Arc (if concurrent execution in Phase 2)

### Book URL: https://doc.rust-lang.org/book/

---

## Key Rust Patterns You'll Use in WOLRAM

### Pattern 1: Enum + Match (State Machine)
```rust
enum State {
    Init,
    DefineAgent,
    Process,
    End,
}

fn next_state(current: State) -> State {
    match current {
        State::Init => State::DefineAgent,
        State::DefineAgent => State::Process,
        State::Process => State::End,
        State::End => State::End, // terminal
    }
}
```

### Pattern 2: Result + ? operator (Error Handling)
```rust
fn load_config() -> anyhow::Result<Config> {
    let content = std::fs::read_to_string("wolram.toml")?; // ? propagates error
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

### Pattern 3: Serde Derive (Serialization)
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Job {
    id: String,
    description: String,
    status: JobStatus,
}
// Now you can: serde_json::to_string(&job) and serde_json::from_str(&json)
```

### Pattern 4: Async/Await (API Calls)
```rust
async fn call_claude(client: &reqwest::Client) -> anyhow::Result<String> {
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", "sk-ant-...")
        .json(&request_body)
        .send()
        .await?;  // await the future, ? propagate errors

    let body: ApiResponse = resp.json().await?;
    Ok(body.content[0].text.clone())
}
```

### Pattern 5: Clap Derive (CLI Parsing)
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wolram")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { prompt: String },
    Run,
    Status,
}
```

---

## Install Rust (run on MacBook Pro)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
cargo --version
```

Then clone and start:
```bash
git clone https://github.com/wolram/wolram.git
cd wolram
cargo init  # will scaffold Cargo.toml + src/main.rs
cargo build
cargo run
```

---

## Useful Cargo Commands

| Command | What it does |
|---------|-------------|
| `cargo build` | Compile the project |
| `cargo run` | Build + run |
| `cargo run -- init "prompt"` | Run with CLI arguments |
| `cargo check` | Fast compile check (no binary output) |
| `cargo clippy` | Linter — catches common mistakes |
| `cargo fmt` | Auto-format code |
| `cargo test` | Run all tests |
| `cargo add clap --features derive` | Add a dependency |
| `cargo doc --open` | Generate + open docs for your crate |
