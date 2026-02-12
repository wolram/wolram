# WOLRAM Architecture

## Inspiration

WOLRAM's architecture is directly inspired by UiPath's REFramework (Robotic Enterprise Framework), the industry standard for production-grade RPA automation. The key insight is that the same patterns that make enterprise automation reliable — state machines, retry logic, transaction-based processing, and audit trails — apply perfectly to AI-assisted development workflows.

## State Machine

Each job (task) flows through four states:

### 1. INIT
- Parse the job from the TODO queue
- Validate inputs and preconditions
- Set up the execution context (working directory, branch, files)

### 2. DEFINE AGENT
- **Skill Router**: analyze the job description and assign the appropriate skill/agent type
  - Code generation, refactoring, testing, documentation, etc.
- **Model Selector**: pick the most cost-effective model based on job complexity
  - Simple tasks (rename, format, small edits) → haiku
  - Medium tasks (implement function, write tests) → sonnet
  - Complex tasks (architecture, multi-file refactor) → opus
- Generate the prompt with full context for the selected agent

### 3. PROCESS
- Execute the job using the assigned agent + model
- Monitor for completion, errors, or timeouts
- On **success**: capture output, validate results
- On **failure**: classify as Business or System failure
  - **Business Failure**: output doesn't meet requirements (wrong code, tests fail)
  - **System Failure**: infrastructure issue (API timeout, rate limit, network error)
- If retries remaining: re-queue with incremented retry count
- If max retries exceeded: log failure and move to END

### 4. END (Finalization)
- Generate audit record (timestamp, model, skill, status, retries, cost)
- If successful: create git commit with structured message
- If failed: log failure details for review
- Update job status in the queue
- Trigger next job if dependencies are met

## Git Integration

```
Job completed successfully → git commit (conventional format)
  - feat(hero): implement hero section layout
  - fix(auth): resolve token refresh logic

Pool of related jobs → git branch
  - feature/hero-section (contains all hero-related commits)
  - feature/auth-flow (contains all auth-related commits)

Full build complete → Pull Request
  - PR body includes audit trail summary
  - Links to individual job reports
  - Total cost, time, success/failure ratio
```

## Audit Trail

Every job produces a structured audit record:

```json
{
  "job_id": "job-001",
  "description": "Implement hero section responsive layout",
  "status": "success",
  "state_transitions": ["INIT", "DEFINE_AGENT", "PROCESS", "END"],
  "agent": {
    "skill": "code_generation",
    "model": "claude-sonnet-4-5-20250929"
  },
  "timing": {
    "started_at": "2026-02-12T04:30:00Z",
    "completed_at": "2026-02-12T04:31:45Z",
    "duration_ms": 105000
  },
  "retries": 0,
  "max_retries": 3,
  "cost_usd": 0.0045,
  "git": {
    "commit": "a1b2c3d",
    "branch": "feature/hero-section"
  }
}
```

## Cost Optimization

The Model Selector is key to keeping costs manageable at scale:

| Job Complexity | Model | Approx Cost | Use Case |
|---------------|-------|-------------|----------|
| Simple | Haiku | $0.001 | Renames, formatting, small edits |
| Medium | Sonnet | $0.005 | Function implementation, test writing |
| Complex | Opus | $0.05 | Architecture, multi-file refactoring |

A 100-job build with 60% simple / 30% medium / 10% complex ≈ $0.71 total.
