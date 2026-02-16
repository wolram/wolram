# Configuration

WOLRAM is configured through a combination of a TOML config file and environment variables.

## Config File: `wolram.toml`

Place a `wolram.toml` file in the project root. If absent, all defaults are used.

### Example

```toml
api_key = ""
default_model_tier = "sonnet"
max_retries = 3
base_delay_ms = 1000
```

A template is available at `wolram.toml.example`.

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `api_key` | String | `""` | Anthropic API key. Overridden by `ANTHROPIC_API_KEY` env var |
| `default_model_tier` | String | `"sonnet"` | Default model tier (`haiku`, `sonnet`, `opus`) |
| `max_retries` | u32 | `3` | Maximum retry attempts for the PROCESS state |
| `base_delay_ms` | u64 | `1000` | Base delay in milliseconds for exponential backoff |

## Environment Variables

| Variable | Description | Priority |
|----------|-------------|----------|
| `ANTHROPIC_API_KEY` | Anthropic API key | Overrides `api_key` in `wolram.toml` |

### Setting the API Key

```bash
# Linux/macOS
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Or inline
ANTHROPIC_API_KEY="sk-ant-..." cargo run -- run "your task"
```

## CLI Overrides

CLI flags take the highest priority, overriding both the config file and environment variables:

```bash
# Override model tier
cargo run -- --model opus run "complex task"

# Override max retries
cargo run -- --max-retries 10 run "flaky task"
```

## Priority Order

For each setting, the highest-priority source wins:

```
CLI flags  >  Environment variables  >  wolram.toml  >  Defaults
```

## Stub Mode

When no API key is configured (empty string and no `ANTHROPIC_API_KEY` env var), WOLRAM runs in **stub mode**:

- The orchestrator skips real API calls
- Jobs complete with simulated success
- All other functionality (state machine, routing, git, audit) works normally
- Useful for testing and development without API costs

---

> *[Versão em Português](../pt-br/Configuration.md)*
