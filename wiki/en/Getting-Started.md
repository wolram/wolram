# Getting Started

## Prerequisites

- **Rust** (edition 2024) — Install via [rustup](https://rustup.rs/)
- **libgit2** — Required by the `git2` crate (usually bundled automatically)
- **Anthropic API Key** (optional) — Required for real LLM calls; without it, jobs run in stub mode

## Installation

Clone the repository and build:

```bash
git clone https://github.com/wolram/wolram.git
cd wolram
cargo build
```

## Configuration

### API Key

Set your Anthropic API key as an environment variable:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

Alternatively, create a `wolram.toml` file in the project root (see [Configuration](Configuration.md) for details).

### Config File (Optional)

Copy the example config:

```bash
cp wolram.toml.example wolram.toml
```

Edit `wolram.toml` to set defaults for model tier, max retries, and backoff delay.

## Running Your First Job

### Demo Mode

Run the built-in state machine demo to see the full lifecycle:

```bash
cargo run -- demo
```

This walks a sample job through all four states (INIT → DEFINE_AGENT → PROCESS → END) and prints the audit record.

### Running a Real Job

Execute a job by description:

```bash
cargo run -- run "implement a function that calculates fibonacci numbers"
```

If `ANTHROPIC_API_KEY` is set, this will call the Anthropic API. Otherwise, it runs in stub mode with simulated success.

### Loading a Job from File

You can also load a pre-configured job from a JSON file:

```bash
cargo run -- run --file job.json
```

### Checking Status

View current configuration, API key status, and git repository info:

```bash
cargo run -- status
```

## CLI Flags

All subcommands accept these global flags:

| Flag | Description |
|------|-------------|
| `--model haiku\|sonnet\|opus` | Override automatic model tier selection |
| `--max-retries <n>` | Override the maximum retry count |
| `--verbose` / `-v` | Enable verbose output to stderr |

Example with overrides:

```bash
cargo run -- --model opus --max-retries 5 --verbose run "redesign the auth module"
```

## Next Steps

- Read the [Architecture](Architecture.md) guide to understand the system design
- Explore the [State Machine](State-Machine.md) to learn about job lifecycles
- Check the [CLI Reference](CLI-Reference.md) for all available commands

---

> *[Versão em Português](../pt-br/Getting-Started.md)*
