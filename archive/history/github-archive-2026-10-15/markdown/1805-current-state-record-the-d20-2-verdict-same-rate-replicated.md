# #1805 — current-state: record the D20 §2 verdict (same-rate, replicated) and the schedule findings

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T22:06:04Z
- **Updated:** 2026-10-09T13:19:46Z
- **Closed:** 2026-10-06T22:07:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1805
- **Merged:** 2026-10-06T22:07:33Z
- **Merge commit:** 4135f29a8b7b0e0d0c95486bbad5860f06577806
- **Base / Head:** main / UOR-Foundation:deepseek/current-state-d20
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Documentation only — no code, no behaviour change.**

`docs/integration/current-state.md` owns changing artifact/results/next action per AGENTS.md, but had **no October 6 entry** — zero occurrences of the verdict's numbers, of `D20`, or of the hybrid's attribution work.

Adds, newest-first:

- **The verdict**, on same-rate and replicated evidence: geometric **0.9992** vs parameter-matched control **0.2537** at lr=3e-4 → **+0.7455**; replicating at 2e-4 at **+0.7360**. Three sealed seeds per cell, 90 sealed roots, full trail on #820.
- **The rate/arm asymmetry**: the control holds 0.2310 → 0.2454 → 0.2537 → 0.2534 across two orders of magnitude while the geometric arm spans 0.175 → 0.981 → 0.999 → lottery; the floor is a **schedule** effect and the ceiling a **rate** effect.
- **The schedule-controlled head-to-head**: geometric **0.9862** vs attention-read hybrid **0.9149** — **+0.0713 capability, 22.6× stability**.
- **`arm=hybrid`** from PR #1798.
- **Next action**: the control is not on `main`; a prebuilt binary runs it; landing it needs a decision on the `reachability` subsystem. The firing counter *is* on `main` under different names.

The verdict was originally filed across rates (geometric@3e-4 vs control@1e-3); running the control at 3e-4 moves the margin by 0.0003, which the entry states explicitly rather than quietly restating the number.

References #820.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

