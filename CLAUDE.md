# WOLRAM - Enterprise Claude Code Orchestrator

## Project Overview
WOLRAM is an enterprise-grade orchestration layer for AI-assisted development. It applies REFramework-style state machine patterns to LLM coding workflows, with intelligent model/skill routing, retry logic, git integration, and full audit trails.

## Architecture
- **State Machine**: INIT > DEFINE AGENT > PROCESS > END (per job)
- **Failure Types**: Business (logic) vs System (infrastructure) — both retryable
- **Git Integration**: job = commit, job pool = branch, build = PR
- **Model Routing**: cost-optimized selection (haiku/sonnet/opus per job complexity)

## Tech Stack
TBD — Rust, Go, or TypeScript (Bun)

## Build Commands
TBD

## Development Conventions
- All architecture decisions documented in /docs/adr/
- State machine transitions must be explicit and logged
- Every job execution produces an audit record
- Git commits from jobs follow conventional commit format
