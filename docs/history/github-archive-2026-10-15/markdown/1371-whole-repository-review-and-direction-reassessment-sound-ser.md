# #1371 — Whole-repository review and direction reassessment: sound serving-arithmetic goal vs unsupported geometry-advantage bet (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T23:03:04Z
- **Updated:** 2026-10-09T13:17:53Z
- **Closed:** 2026-09-23T23:04:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1371
- **Merged:** 2026-09-23T23:04:07Z
- **Merge commit:** 10388ac28bc5b957fdf4690d63c7a702384b77f5
- **Base / Head:** main / UOR-Foundation:codex/repo-review-direction-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

Docs-only. A **whole-repository review and direction reassessment** commissioned by the owner: it reads the full
authority chain, the complete mechanism/decision history, the current runtime mechanics, the available/dormant
geometric mechanisms, and the external prior art, integrates **eight independent specialist reviews** (history,
runtime mechanics, mechanism inventory, roadmap falsification, literature, mathematics, CS, systems) plus a **K3
architecture review**, and puts a staged proposal to the owner. No model compute, no artifact change, no run.

## Central findings

1. **The serving-arithmetic goal is sound and provable; the representational bet is not.** {bounded add/sub/shift/
   compare/table-read} provably emulates any fixed-precision ReLU/threshold net, any finite-state transducer, and
   any bounded-state fixed-precision RNN. But **no geometric mechanism has ever beaten an information- and
   compute-matched ordinary control** on any task, and the best language artifact (`transferable_lexical.rs`)
   contains **no geometry at all**.
2. **The project is bound by capacity/context/objective, not geometry or multiply-freeness:** 64-dim state,
   **64-token independent windows**, input-independent `>>3` transition with **no gate**, 4k vocab, <3 epochs, and
   a **servable** two-token count-prior objective. The constraint's measured quality cost is 0.000 bits (integer ==
   float surrogate).
3. **The "rank-64 interface" framing is imprecise** — the readout input is **147**-wide, so the bound is ≤147; the
   real limit is linearity/higher-order features, not state dimension.
4. **"World models" does not describe this project** (neither the Dreamer/planning sense nor the Sora/generative
   sense). Recommend retiring the term from claims.
5. **Two headline numbers are not what they appear:** TinyStories 1.2372 BPB is continuous *training-time*
   (shipped 1.8055); 617 vs 36.1 tok/s is not quality-matched and energy is UNAVAILABLE.

## Proposed plan (awaiting owner decision; not adopted by this PR)

Stage 1 frozen-state adjudication **including the already-ordered transport-decode arm** (settles the
2.1% vs 70.96%/75.90% `prev` contradiction on byte-identical states) → Stage 2A `h_dim`/gate ablation with a
matched float control → Stage 3 cross-window state + an **induction/copy-at-distance language probe** where the
count control is at chance by construction → Stage 4 LUT kernel + integer/LUT-only context layers → scale.
Geometry becomes a **conditional promotion gated on a matched comparator**, preserving the owner's terminal
objective; the K3 review's corrections (goal-change honesty, arm (iv), Stage-1 confounds, attention boundary,
standing count control + natural-text panel) are incorporated.

`python3 scripts/check_claim_wording.py` passes. References #820, #973. Owner decisions are listed in §11 of the
document.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

