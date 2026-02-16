# Testing

WOLRAM has a comprehensive test suite with ~80 tests covering all modules.

## Running Tests

```bash
# Run all tests
cargo test

# Run a specific test by name
cargo test happy_path_walks_all_states

# Run tests in a specific module
cargo test --lib state_machine

# Run tests with output
cargo test -- --nocapture
```

## Test Organization

All tests are inline `#[cfg(test)]` modules within each source file — there is no separate `tests/` directory.

## Testing Strategies

### Unit Tests

Pure function testing for deterministic logic:

- `SkillRouter::route()` — keyword matching correctness
- `ModelSelector::select()` — complexity classification
- `RetryConfig::delay_for_attempt()` — exponential backoff calculation
- `StateMachine::next()` — state transition rules
- Config deserialization and defaults
- Error display formatting
- Type serialization/deserialization

### Mock Client Tests

Both `router.rs` and `orchestrator.rs` define local `MockClient` structs implementing `MessageSender`:

```rust
struct MockClient {
    response: String,
}

impl MessageSender for MockClient {
    async fn send_message(&self, _req: &MessagesRequest)
        -> Result<MessagesResponse, AnthropicError> {
        // Return hardcoded response
    }
}
```

This allows testing LLM classification and orchestration paths without real API calls.

### HTTP Mock Server Tests

The `client.rs` module uses the `wiremock` crate to test the HTTP client against a real (mock) server:

- Mounts response matchers for specific HTTP methods and headers
- Tests successful 200 responses
- Tests 429 rate limiting (with and without `retry-after` header)
- Tests 500 server error handling

### Async Tests

All orchestrator and client tests use `#[tokio::test]` for async execution.

### Tempfile-Based Git Tests

`git.rs` tests use `tempfile::TempDir` to create disposable Git repositories:

- Tests commit creation
- Tests branch creation
- Tests job result commits
- Automatic cleanup when the test completes

### CLI Validation Tests

`cli.rs` tests use `Cli::parse_from()` to validate argument parsing and `Cli::command().debug_assert()` for clap configuration integrity.

### Serialization Roundtrip Tests

Verify that types can be serialized and deserialized without data loss:

- `Job`
- `MessagesRequest` / `MessagesResponse`
- `AuditRecord`

## Key Test Names

### State Machine Tests

| Test | What It Verifies |
|------|------------------|
| `happy_path_walks_all_states` | Full INIT→DEFINE_AGENT→PROCESS→END flow |
| `business_failure_retries_then_fails` | Business failures exhaust retries |
| `system_failure_retries_then_fails` | System failures exhaust retries |
| `zero_retries_fails_immediately` | Zero retries = immediate failure |
| `retry_then_succeed` | Recovery after retry |
| `state_history_is_recorded` | Audit trail completeness |

## Writing New Tests

1. Add tests in a `#[cfg(test)]` module at the bottom of the relevant source file
2. Use `MockClient` for any test that needs LLM responses
3. Use `tempfile::TempDir` for any test that needs a Git repository
4. Mark async tests with `#[tokio::test]`
5. Follow the naming convention: `snake_case` describing the scenario

## Linting and Formatting

```bash
# Lint
cargo clippy

# Format
cargo fmt
```

---

> *[Versão em Português](../pt-br/Testing.md)*
