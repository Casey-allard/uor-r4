# GitHub-Style Tracking Model

## Purpose
Use the existing git repository at `/Users/adminamn/ai-router` as the history layer while keeping router research tracking explicit and local inside `router-research`.

## Source Of Truth
1. Research state: `docs/research/`
2. Increment records: `docs/research/increments/`
3. Local issue registry: `docs/program/ISSUE_REGISTRY.md`
4. Local board: `docs/program/PROJECT_BOARD.md`
5. Branch policy: `docs/program/BRANCHING_STRATEGY.md`
6. Git history: parent repo `/Users/adminamn/ai-router/.git`

## Issue Model
- Use `RR-###` identifiers for local tracking.
- Every live increment gets one issue line in `ISSUE_REGISTRY.md`.
- Every closed increment should keep its evidence in the increment doc, not in the registry only.

## Labels
- `research`
- `systems`
- `math-review`
- `translation`
- `runtime`
- `confirm`
- `queued`
- `active`
- `done`
- `blocked`

## Branch Model
- Prefer `codex/<issue-id>-<slug>` if a dedicated branch is created.
- Keep one mechanism family per branch where possible.
- Do not mix deep geometry and systems rescue work in one branch unless the increment explicitly requires both.

## Pull Request Model
Every meaningful change should be reviewable with:
1. problem statement
2. exact files changed
3. evidence artifacts
4. promotion / kill decision
5. next branch

## Agent Rule
When an agent closes or materially redirects a branch, it must update:
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/HANDOFF_CURRENT.md`
- `docs/research/LIVE_WORKLOG.md`
- `docs/research/PROGRESS_TRACE.md`
- `docs/program/ISSUE_REGISTRY.md`
- `docs/program/PROJECT_BOARD.md`

## Current Use
This project is not required to have a GitHub remote. The GitHub-style layer exists so humans and agents can track work with issue/PR discipline anyway.

## Current Git Constraint
The parent git repo currently has no valid `HEAD` commit.
That means:
- the working branch name can be set
- local tracking docs can be formalized
- additional branch refs and normal branch history require an initial commit before they become fully real
