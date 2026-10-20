# #1447 — B1 closure: reflection-pair replication fails, transformer control, exact anchor; Stage C parked (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T02:00:44Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-09-28T05:30:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1447
- **Merged:** 2026-09-28T05:30:23Z
- **Merge commit:** 4a9a4100a8b327eaee503bcbbbd351c8277cc0b2
- **Base / Head:** main / UOR-Foundation:lab/claude/b1c-natural-tracking
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Claude, Lab 1. **Track:** ROADMAP T1(b), B1. **Pre-registration:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863149284), posted before any closure run and owner-approved.

**Results** (*Measured*; every report root sealed; evidence in `docs/evidence/b1-closure-2026-09-28.json`):
- **Fresh replication of reflection-pair lanes, seeds 4–6: FAIL.**
  - A5 is 1.000 in 3 of 3 seeds, and every served 60-state automaton passes `verify_exact`.
  - But ΔNLL is +0.069 in seed 4, and the gate was ≤0.05 in every seed.
  - So **no lane type is kept in the stack**, and finite-group state stays a Stage A result.
- **Stage B transformer control (formerly NOT_RUN).** It does not track A5: 0.008–0.027 at position 128. Its text NLL is 0.002–0.044 nats worse than the lane-free stack. It ran at the stack's learning rate; its own optimum was not searched.
- **Same-binary anchor: exact.** The development NLL, A5 accuracies and every training record are identical to Stage B.
- **Stage A rerun:** 36 of 36 runs reproduce the record. `verify_exact` is true on all 17 exact automata, and on no others.
- **Stage C: parked at this scale on pilot evidence.** Its grid is NOT_RUN.
  - In three pilots no arm learned swap tracking, even with a dense state head.
  - Context lanes cost 0.13–0.20 nats and rotate at almost every token.
  - The pilots and amendments are disclosed in `docs/integration/b1c-context-lanes-swap-stories-2026-09-28.md`.

**Scope.** An exact automaton proves exact A5 state. It does not show that any language is interpreted correctly. Nothing here is a language-capability result.

**Code.**
- The stack's `run_layers`/`finish` split.
- Context-conditioned lanes.
- The swap-story task with a state head, and the `tracking-lanes stories` driver.
- The matched transformer control arm.
- `LaneAutomaton::verify_exact` and a panic-free `run`.

**Checks.** 7 focused `stack_tracking` tests pass (release). `cargo fmt --check` is clean, and `scripts/check_claim_wording.py` passes.

References #973, #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

