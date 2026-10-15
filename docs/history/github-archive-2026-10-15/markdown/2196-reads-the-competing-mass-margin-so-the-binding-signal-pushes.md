# #2196 — reads: the competing-mass margin, so the binding signal pushes the read off the other stated values (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T08:16:22Z
- **Updated:** 2026-10-11T09:05:03Z
- **Closed:** 2026-10-11T08:20:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2196
- **Merged:** 2026-10-11T08:20:39Z
- **Merge commit:** a612cab9c3181851560cb58cf215895f8a3aad58
- **Base / Head:** main / UOR-Foundation:deepseek/competing-margin
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The keeper's read-binding supervision maximizes the mass on the *asked* value only. Its remaining failure mode is the read putting mass on a *competing* stated value — 30 wrong-value failures over 120 rows — and the brief names the fix as its own open question: *"A competing-mass margin loss, rather than only maximizing bound mass?"* This PR adds it.**

Line: read binding / competing-mass margin (D22 order 4) · count 0/3 · headline unchanged (preparation)

## What changed

| | |
|---|---|
| `read_binding_competing_weight=W` (default 0) | the objective gains `W · mean(relu(competing mass − M))` — a hinge, inert below the margin, pushing down above it |
| `read_binding_competing_margin=M` (default 0.03) | the mass the competing values must stay under |
| defaults | `0.0` leaves every objective **bit for bit** as it was; the term is **training-only**, not saved, and changes no serving path |
| validation | the weight needs a positive supervision weight; the margin must be a mass in `0..1`; both recorded in the run's settings and lineage |
| reporting | the run's curve gains `train_binding_margin_term` beside the bound and competing masses |

## Checks at this head

- `cargo build --release -p uor-r4-training --lib` and `--example geometric-stack` — clean.
- **Focused test** `a_competing_margin_adds_a_nonnegative_term_and_changes_nothing_at_zero`: the default leaves the objective untouched, the term is non-negative, it does not change the binding term, and out-of-range margins/weights are refused. The four existing read-binding tests still pass.
- `python3 scripts/check_claim_wording.py` — **pass**.

## What it prepares

**Three seeds** at weight `0.5`, margin `0.03` — chosen so the term is *active from the first step* (the keeper's competing mass is 0.081–0.090, above the margin on every seed) and contributes ≈ 0.028, about a quarter of the binding term's own contribution. Bar: ≥ 5 fewer wrong-value failures than the **matched keeper arm** on ≥ 2 of 3 seeds, or ≥ +4 memory rows in aggregate, with BPB within 0.01 — row-paired, no new controls needed because the keeper's arms are the matched baseline.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

