# Router & Skills

The router module handles two decisions for every job: **which skill** to assign and **which model tier** to use.

## Skill Router

`SkillRouter::route(description) -> String` assigns a skill label based on the job description.

### Available Skills

| Skill | Description |
|-------|-------------|
| `testing` | Writing tests, specs, test suites |
| `refactoring` | Code restructuring and cleanup |
| `documentation` | Docs, READMEs, comments |
| `bug_fix` | Debugging, error fixing |
| `code_generation` | New features, implementations (default) |

### Keyword Scoring

The router uses weighted keyword matching:

| Skill | Keywords (weight) |
|-------|-------------------|
| `testing` | "test" (10), "spec" (5) |
| `refactoring` | "refactor" (10), "clean up" (5) |
| `documentation` | "doc" (10), "readme" (5) |
| `bug_fix` | "fix" (10), "bug" (10), "debug" (7), "error" (5) |
| `code_generation` | "implement" (5), "add" (3), "create" (5), "build" (5) |

The description is lowercased and scanned for each keyword. The skill with the highest cumulative score wins. If no keywords match, `code_generation` is used as the default.

### LLM-Based Classification

When an API key is available, `classify_with_llm()` attempts LLM-based classification first:

1. Sends the job description to `claude-haiku-4-5-20251001` with a structured prompt
2. Expects JSON: `{"skill": "...", "complexity": "..."}`
3. On success, returns the LLM's skill and complexity assessment
4. On any failure (network, bad JSON, unknown skill), falls back to keyword scoring

## Model Selector

`ModelSelector::select(description) -> ModelTier` chooses a model tier based on task complexity.

### Complexity Scoring

Two categories of keywords are scored:

**Simple keywords** (push toward Haiku):

| Keyword | Weight |
|---------|--------|
| "rename" | 10 |
| "format" | 10 |
| "typo" | 10 |
| "delete" | 7 |
| "remove" | 5 |
| "update" | 3 |

**Complex keywords** (push toward Opus):

| Keyword | Weight |
|---------|--------|
| "architect" | 10 |
| "redesign" | 10 |
| "overhaul" | 10 |
| "refactor" | 8 |
| "migrate" | 8 |
| "multi-file" | 10 |
| "system" | 5 |

### Heuristics

In addition to keywords, the selector applies:

- **Length heuristic**: Descriptions under 20 characters get +5 simple score; over 100 characters get +5 complex score
- **Word count**: Over 15 words adds +3 complex score

### Selection Logic

| Condition | Result |
|-----------|--------|
| simple_score > complex_score + 5 | `Haiku` |
| complex_score > simple_score + 5 | `Opus` |
| Otherwise | `Sonnet` (default) |

## Integration Flow

In the orchestrator's DEFINE_AGENT phase:

1. If an API client is available:
   - Try `classify_with_llm()` first
   - On failure, fall back to keyword routing
2. If no API client:
   - Use `SkillRouter::route()` and `ModelSelector::select()` directly
3. If a CLI `--model` override was provided, it replaces the selected tier
4. The skill and model are stored in `job.agent` as an `AgentConfig`

---

> *[Versão em Português](../pt-br/Router-and-Skills.md)*
