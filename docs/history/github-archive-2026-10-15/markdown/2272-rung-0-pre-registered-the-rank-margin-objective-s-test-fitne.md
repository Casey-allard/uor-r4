# #2272 — Rung 0 pre-registered: the rank-margin objective's TEST FITNESS sheet (PENDING)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:17:06Z
- **Updated:** 2026-10-13T03:20:56Z
- **Closed:** 2026-10-13T03:19:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2272
- **Merged:** 2026-10-13T03:19:03Z
- **Merge commit:** 92592f9cb503461f9206241bf6f8d205ee4a98d7
- **Base / Head:** main / UOR-Foundation:kimi/rung0-prep-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

## Rung 0 pre-registered: the rank-margin objective's TEST FITNESS sheet

Docs-only. Record = the sheet itself: `docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`.
Decision: **NOT YET PROMOTED** — the sheet is `TEST FITNESS: PENDING` and the run is **BLOCKED**. No model
compute, no count reset, no headline moved.

### What landed

- **`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`** — the pre-registration, generated with
  `python3 scripts/harness/cli.py prep new -m m1 --mechanism rank-margin-objective --seeds 2` and then
  completed: every `MISSING` field filled (fresh rows, fresh-transfer plan, pod/lease plan, and the six cost
  lines written out one by one rather than repeating one token), plus the sections the sheet needs to be
  checkable: **the exact executed path** (gate merged #2260, projection in flight per
  [#2029 comment 6136794551](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6136794551),
  head not built, objective in tree, recurrence sites below), **the remaining dense/squashing/host-fallback
  sites with verified file:line references**, the **configuration**, the **lawful raw-score quality measures**
  (top-1 with a deterministic tie rule, target rank/recall@8, the hinge, the unviolated-target rate — no BPB,
  no NLL, no normalized-exponential evaluator anywhere), **panel noise**, **mechanism-own target and distinct
  outcome decisions** (KEEP / ITERATE / BLOCKER; a mechanism stop stays the owner's), **tokens/s** on the
  named harness, **lawful device qualification** (stored/analytic expected values, no CPU reference), and the
  **four dependencies that must merge first**. Exactly one verdict line, `TEST FITNESS: PENDING`.
- **`docs/mechanisms/rank-margin-objective.md`** — the missing mechanism brief: the hard hinge
  `mean_k relu(1 − z_target + z_rival)` in raw-score units with no exponential in either direction, what it
  replaces (softmax CE and the NLL/BPB unit), where it lives in code (`Objective::MarginHinge` `:1374`,
  `margin_row`/`margin_row_grad` `:18574`/`:18598`, the CUDA f32 kernels `margin_rank_rows`/`margin_rank_grad`
  `cuda_stack_kernels.rs:738`/`:744`, the pointer refusal at `:2773`), what has been tried (implementation and
  focused tests landed; **no training run has executed it**), success/failure, the fair test (this sheet) and
  the open questions. Its row is added to `docs/mechanisms/README.md`.
- A newest-first `docs/integration/current-state.md` entry naming the sheet, its PENDING state and the four
  dependencies.

### Two things the sheet surfaces rather than hides

1. **The served `DecayRate` codec is not a lawful replacement for the exp decay.** It encodes
   `8 softplus(-decay)` (`geometric_stack.rs:13055`), so pointing the recurrence at that table would *move*
   the softplus, not remove it. The sheet names an **exact** rate selection as the source item instead.
2. **The current bootstrap parity file compares against CPU references** and is itself out of step with
   D24/D25. Device qualification is registered as stored/analytic expected values, and replacing that file is
   dependency (4) — a parity file that exits zero on a CPU reference is not device qualification.

### Verification

- [x] `python3 scripts/harness/cli.py check` — 0 errors; the sheet's single rule hit is
      `prep.fit_before_run` ("TEST FITNESS: PENDING — not cleared for compute"), which is the **correct**
      state for a pre-registration awaiting its review; the other warns are other milestones' unfrozen panels,
      other labs' orphan branches and the standing caps notice
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 scripts/check_no_softmax.py` — **OK, 1649** (baseline 1649)
- [x] `python3 scripts/check_no_matmul.py` — **OK, 95 sites / 26 files** (baseline 95)
- [x] Every relative link in the two new documents resolves on disk; the sheet's markdown tables are
      well-formed; exactly one `TEST FITNESS:` line in the sheet
- [x] Docs only — no Rust source touched, so no cargo build or test was run (`cargo fmt` is not applicable)
- [x] Files a *deliberate* lint warn: `prlint.record_path` wants a dated `docs/labs/<topic>-<date>/README.md`,
      and this unit's instruction is that **the sheet is the record** (`docs/harness/prep/` is where sheets live;
      the directory had only a `.gitkeep` before this PR). Nothing was duplicated to satisfy the rule; the
      current-state entry points at the sheet.

Pre-registration: this PR **is** the pre-registration. `TEST FITNESS: FIT` is Claude's review to record; FIT
lifts no hold, and the owner releases the training hold separately.

Cost: one documentation session on the owner's laptop, no compute, no pod leased.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

