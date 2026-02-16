# Contributing

## Development Setup

1. Clone the repository
2. Install Rust (edition 2024) via [rustup](https://rustup.rs/)
3. Run `cargo build` to verify the setup
4. Run `cargo test` to confirm all tests pass

## Conventions

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new skill for database migrations
fix: handle rate limit without retry-after header
docs: update architecture diagram
test: add mock tests for router LLM path
refactor: simplify state machine transition logic
```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- All public types must derive `Serialize` and `Deserialize`
- Every job execution must produce an `AuditRecord`

### Testing

- Write tests as inline `#[cfg(test)]` modules within source files
- Use `MockClient` for tests needing LLM responses
- Use `tempfile::TempDir` for tests needing a Git repository
- Mark async tests with `#[tokio::test]`
- Aim for comprehensive coverage of both happy paths and error cases

### Branch Naming

- Feature branches: `feat/<description>`
- Bug fixes: `fix/<description>`
- Default branch: `main`

## Architecture Guidelines

- **State machine purity**: `StateMachine::next()` should remain a pure function that takes a job and outcome, returning a transition. Side effects belong in the orchestrator.
- **Trait-based abstraction**: Use traits (like `MessageSender`) to enable testing without external dependencies.
- **Error propagation**: Use `anyhow::Result` for application errors and `thiserror` for library-level error types.
- **Serialization**: All data types that cross boundaries should derive `Serialize`/`Deserialize`.

## Adding a New Skill

1. Add the skill name and keywords to `SkillRouter` in `router.rs`
2. Update `classify_with_llm()` prompt to include the new skill
3. Add tests for the new keyword routing
4. Update this wiki's [Router & Skills](Router-and-Skills.md) page

## Adding a New State

1. Add the variant to `State` enum in `state_machine/state.rs`
2. Update `StateMachine::next()` transition logic
3. Add the handling code in `orchestrator.rs`
4. Add tests covering all transitions involving the new state
5. Update this wiki's [State Machine](State-Machine.md) page

---

> *[Versão em Português](../pt-br/Contributing.md)*
