# CLI Reference

WOLRAM provides a clap-based command-line interface with three subcommands and global flags.

## Usage

```
wolram [OPTIONS] <COMMAND>
```

## Global Flags

These flags are available on all subcommands:

| Flag | Short | Description |
|------|-------|-------------|
| `--model <TIER>` | | Override model tier selection. Values: `haiku`, `sonnet`, `opus` |
| `--max-retries <N>` | | Override maximum retry count (default: 3) |
| `--verbose` | `-v` | Enable verbose output to stderr |

## Subcommands

### `run`

Execute a job. Provide a description inline or load from a JSON file.

```bash
# Run by description
wolram run "implement a fibonacci function"

# Run from a JSON file
wolram run --file job.json

# With overrides
wolram --model opus --max-retries 5 run "redesign the auth module"
```

**Arguments:**

| Argument | Required | Description |
|----------|----------|-------------|
| `<description>` | No* | Job description text |
| `--file <path>` | No* | Path to a JSON-serialized Job file |

\* One of `description` or `--file` must be provided.

**File loading behavior**: When loading from `--file`, the following fields are reset to ensure a clean initial state:
- `state` → `Init`
- `retry_count` → `0`
- `state_history` → `[]`
- `status` → `Pending`

### `demo`

Run the built-in state machine demo. Walks a sample job through all four states and prints the audit record.

```bash
wolram demo
```

No arguments or options.

### `status`

Display current configuration, API key status, and git repository info.

```bash
wolram status
```

Shows:
- Loaded configuration from `wolram.toml`
- Whether `ANTHROPIC_API_KEY` is set
- Current git branch and repository info

## Examples

```bash
# Basic job execution
cargo run -- run "add unit tests for the router module"

# Verbose mode with model override
cargo run -- --verbose --model haiku run "fix typo in README"

# Demo walkthrough
cargo run -- demo

# Check system status
cargo run -- status

# Load job from file with max retries
cargo run -- --max-retries 10 run --file complex-job.json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Job failed (after retries exhausted or validation error) |

---

> *[Versão em Português](../pt-br/CLI-Reference.md)*
