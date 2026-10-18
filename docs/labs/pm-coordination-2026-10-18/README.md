# PM coordination hygiene  2026-10-18

DeepSeek is covering PM coordination while Kimi is rate-limited. First action: coordination
hygiene, no model compute, no code changes.

## What was cleaned

- Removed 25 stale committed claims for DeepSeek branches whose work is already merged and whose
  branches are gone (object-store series, wire side exploration follow-ups, k16/research helpers).
  The claims were retained after their protected merges and cluttered the cross-lab board.
- Removed the local orphan branch that uor-harness check reported (worktree-agent-*), after proving
  it is already contained in origin/main.
- Kept active DeepSeek claims only.

## Verification

    uor-harness multilab claims   (only live branches remain for deepseek)
    uor-harness check             (no FAIL; orphan-branch warning addressed)
    conflict-marker scan on touched docs: clean

## Coordination

No other lab's claims, branches or files were touched. Committed claims remain the union for other
labs. Announcement and the PM board will follow on #2028 after the cross-lab survey completes.

## Second claims cleanup (2026-10-18)

The two DeepSeek claims that rode merged delivery PRs (#2629 pod read-back, #2633 wire matrix) plus this cleanup branch claim were removed from docs/harness/claims.json. No code, model or evidence change.

## Third claims cleanup (2026-10-18)

The claim that rode merged PR #2635 (object-store batch plumbing) and this cleanup branch claim were removed from docs/harness/claims.json. No code, model or evidence change.

## Fourth claims cleanup (2026-10-18)

The claim that rode merged PR #2638 (pipelined FETCH) and this cleanup branch claim were removed from docs/harness/claims.json. No code, model or evidence change.

## Fifth claims cleanup (2026-10-18)

The claim that rode merged PR #2641 (cold-join benchmark) and this cleanup branch claim were removed from docs/harness/claims.json. No code, model or evidence change.
