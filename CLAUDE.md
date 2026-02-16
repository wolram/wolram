# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WOLRAM is an enterprise orchestration layer for AI-assisted development, inspired by UiPath's REFramework. It applies state machine governance, retry logic, model routing, and audit trails to LLM coding workflows.

**Status: Pre-development** — architecture is defined, no application code exists yet. Tech stack is under evaluation (Rust, Go, or TypeScript/Bun).

## Architecture

Each job flows through four states: **INIT → DEFINE AGENT → PROCESS → END**

- **Skill Router** (DEFINE AGENT phase) assigns agent type per job
- **Model Selector** picks cost-optimal model: haiku (simple) → sonnet (medium) → opus (complex)
- Failures are classified as **Business** (logic/validation) or **System** (infra/timeout), both retryable up to a configurable max
- Every job produces a structured audit record (JSON schema in `docs/architecture.md`)

Full architecture details: `docs/architecture.md`

## Git Integration

- Job completed → `git commit` using conventional commits (`feat(scope): description`)
- Pool of related jobs → `git branch` (e.g., `feature/hero-section`)
- Full build complete → Pull Request with audit trail summary

## CI/CD

Two GitHub Actions workflows (both use `anthropics/claude-code-action@v1`):

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `claude.yml` | `@claude` mentions in issues/PRs | Interactive Claude assistant |
| `claude-code-review.yml` | PR events (opened, synchronize, reopened, ready_for_review) | Automated code review via `code-review` plugin |

Both require `ANTHROPIC_API_KEY` as a repository secret.

## Build Commands

No build system configured yet (pre-development).

## Environment

- Copy `.env.example` to `.env` and set your keys
- CI/CD uses `ANTHROPIC_API_KEY` (repository secret)

## Development Conventions

- Architecture decisions documented in `/docs/adr/`
- State machine transitions must be explicit and logged
- Git commits follow conventional commit format: `type(scope): description`
