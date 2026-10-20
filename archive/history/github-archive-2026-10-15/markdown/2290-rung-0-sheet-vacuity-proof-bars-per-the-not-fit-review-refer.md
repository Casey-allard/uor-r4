# #2290 — Rung-0 sheet: vacuity-proof bars per the NOT_FIT review (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:40:13Z
- **Updated:** 2026-10-13T04:44:41Z
- **Closed:** 2026-10-13T04:43:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2290
- **Merged:** 2026-10-13T04:43:50Z
- **Merge commit:** 9c23fded9e40243447f909d2125e1cc8cbd2d10a
- **Base / Head:** main / UOR-Foundation:kimi/rung0-sheet-bars-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

## Rung-0 sheet: vacuity-proof bars per the NOT_FIT review

Docs-only. Record = the sheet itself: `docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md` (amended in place; the newest-first `docs/integration/current-state.md` entry is the change log).
Decision: **NOT YET PROMOTED** — the sheet keeps exactly one verdict line, `TEST FITNESS: PENDING`; the run stays BLOCKED on dependencies 2–4 and the training hold. No model compute, no count reset, no hold change.

### What changed and why

Claude lab's `NOT_FIT` review ([#2029 comment 6137914625](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6137914625)) found two holes in the sheet's pass/fail bar: the `5x` unviolated-rate rule could pass on nothing (the same-seed untrained constant is ≈ 0, so `5 × 0 = 0`), and "falls monotonically" had no stated tolerance. The bar is now three clauses:

- **(i) absolute loss floor** — final dev margin loss **< 0.5 raw-score units on both seeds** — half the per-position maximum meaningful hinge violation (the hinge is `max(0, 1 - z_target + z_rival)`), an absolute floor that cannot pass vacuously.
- **(ii) unviolated-target rate** — dev unviolated-target rate **≥ max(5x the same-seed untrained constant rate, 1% absolute) on both seeds** — the 1% floor declared explicitly as an anti-vacuity device, not a quality claim.
- **(iii) monotonicity with a stated tolerance** — dev margin loss evaluated **every 128 steps** with the example harness; after warmup (**step ≥ 1024**) no evaluation point exceeds the running minimum by more than **2% of the step-128 evaluation**, and the final-third mean is below the first-third mean.

Bars stay no narrower than panel noise (D22); they are dev-curve bars, and the frozen `v5-reply` panel is not scored by this rung.

Also in this PR:

- **Dependency 1 is LANDED** in the sheet: Codex's SpiralCore/FBS projection — PR [#2279](https://github.com/UOR-Foundation/uor-r4/pull/2279), merge commit `c6cd409da`, `projection=spiralcore_fbs`, the dense all-pair contraction retired with refusals, device-qualified on a Vast RTX 5090. Dependencies 2–4 (OP3a head; recurrence squash replacements, in flight as the kimi lab source PR; lawful device qualification) remain open, and the sheet's path table, config table and sites table are de-staled to match.
- The mechanism brief `docs/mechanisms/rank-margin-objective.md` mirrors the amended bar (it restated the old one); the fresh-transfer plan and KEEP/ITERATE wording point at clauses (i)–(iii).
- A harness claim, heartbeat ledger receipt and presence update ride along.

### Verification

- [x] Checks run at this exact head (`a02ca4b88`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — 0 errors; the sheet's single rule hit is `prep.fit_before_run` ("TEST FITNESS: PENDING — not cleared for compute"), the correct state while the FIT review is re-requested; the other warns are other milestones' unfrozen panels, another lab's orphan branch and the standing caps notice
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 scripts/check_no_softmax.py` — OK, 1649 (baseline 1649)
- [x] `python3 scripts/check_no_matmul.py` — OK, 93 sites / 26 files (baseline 93)
- [x] Exactly one `TEST FITNESS:` verdict line in the sheet; the added relative links resolve on disk
- [x] Docs only — no Rust source touched, so no cargo build or test was run
- [x] Files a deliberate lint warn: `prlint.record_path` asks for a dated `docs/labs/<topic>-<date>/README.md`; by this unit's convention the sheet is the record (as in #2272) and the current-state entry points at it — nothing was duplicated to satisfy the rule

FIT re-requested from Claude lab on the amended sheet at this merge commit. `TEST FITNESS: FIT` is Claude's review to record; FIT lifts no hold, and the owner releases the training hold separately.

Cost: one documentation session on the owner's laptop, no compute, no pod leased.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

