# Git Integration

WOLRAM integrates with Git via the `git2` crate (libgit2 bindings) to automatically commit job results and manage branches.

## GitManager

The `GitManager` struct wraps a `git2::Repository`:

```rust
let git = GitManager::open(".")?;
```

## Methods

### `open(path) -> Result<GitManager>`

Opens an existing Git repository at the given path.

### `commit(message) -> Result<String>`

Stages and commits all tracked and new files, returning the 7-character short hash.

**Excluded files** (not staged):
- `wolram.toml`
- `.env`
- `.env.local`
- `*.key`

**Commit author**: Uses the configured Git user, or falls back to `WOLRAM <wolram@localhost>`.

### `commit_job_result(job) -> Result<String>`

Convenience method that formats a commit message and calls `commit()`:

```
wolram: [skill] description (status)
```

Example:
```
wolram: [code_generation] implement fibonacci function (Completed)
```

### `create_branch(name) -> Result<()>`

Creates a new branch from HEAD and checks it out.

### `create_job_branch(job) -> Result<()>`

Creates a branch named after the job:

```
wolram/<first-8-chars-of-job-id>
```

Example: `wolram/a1b2c3d4`

### `current_branch() -> Result<String>`

Returns the current branch name (shorthand).

## Orchestrator Integration

In the orchestrator flow, after the PROCESS state completes successfully:

1. If `has_git` is `true`, calls `GitManager::commit_job_result(job)`
2. On commit failure, a warning is printed but the job continues
3. The commit does not affect the job's success/failure status

This means Git integration is best-effort — a Git error will not cause the job to fail.

## Sensitive File Protection

The commit method automatically excludes sensitive files from staging:

| Pattern | Reason |
|---------|--------|
| `wolram.toml` | May contain API keys |
| `.env` | Environment secrets |
| `.env.local` | Local environment secrets |
| `*.key` | Private keys |

---

> *[Versão em Português](../pt-br/Git-Integration.md)*
