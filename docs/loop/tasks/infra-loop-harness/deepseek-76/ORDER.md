# ORDER: reconcile harness registry-aware checks after current milestone status, deepseek-76

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 5 (registry-aware PR-check and status fixtures; no bar, result, hold or serving change). Base: `43809a41a`.

## Goal

Make the harness consume the current milestone registry consistently and remove stale command examples that make `cargo test -p uor-r4-harness` red. Do not change any milestone target, acceptance bar, hold, model result or source claim.

## Scope

- `crates/uor-r4-harness/src/**` and `crates/uor-r4-harness/tests/**` only for stale registry-aware test fixtures and logic.
- `docs/harness/milestones.json` only if current historical lines must be represented without changing headline/bar/target/status.
- `docs/labs/session-goal.md` only to reconcile its historical registry cross-check with the current registry.
- The three stale command examples under `docs/loop/tasks/`.
- `docs/loop/tasks/infra-loop-harness/deepseek-76/**`.
- `docs/harness/claims.json`.

No serving code, training code, corpus run or owner decision.

## Required outcome

- Preserve M1 complete at 6/6, M2/M3 in progress, M7 in progress, and every current headline/target.
- The real-repository briefing, harness-rule, PR-gate, headline and heartbeat tests agree with the current registry rather than the pre-M1-close fixture state.
- Keep historical line evidence explicit; do not relabel a historical 0/3 or 1/3 line as new progress.
- Replace dead documented commands with commands that exist today and remove shell prose from command fences so the documented-command parser sees only commands.
- `cargo test --locked -p uor-r4-harness` and `uor-harness check` pass with zero errors, plus formatting, Clippy and loop gates.

## Stop

If a fix would require changing an owner bar, milestone completion status, a hold, a serving path or an actual result, stop and report it.
