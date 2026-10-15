# #464 — test(certify): #458 — interaction information over context variable groups: no measurable synergy at D3 scale

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-06T14:56:06Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-06T15:16:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/464
- **Merged:** 2026-08-06T15:16:53Z
- **Merge commit:** a0c5e7bea04b91d9bd5860ae5dc88a13cfc7932c
- **Base / Head:** main / UOR-Foundation:issue-458-interaction-blocks
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

First slice of #458: an `#[ignore]`d interaction-information harness (`crates/uor-r4-graph-certify/tests/interaction_information.rs`) measuring which context variable groups carry dependency — the block-selection evidence for choosing interaction blocks. Same harness conventions as #459: pre-registered header (interpretation, null requirements N1–N3, exit rule), story-bounded offset variables (t₋₁…t₋₈ + next), exact packed keys, fixed-seed shuffled-next null, vacuous skip, deterministic (sorted-key f64 summation — a HashMap-seed ULP nondeterminism in "identical" tables was found and fixed during development; N2 now passes bit-identically).

Smoothing is pre-declared per order: Laplace add-one over per-table alphabets for pairwise MIs (MI ≥ 0 arithmetically guaranteed; a per-variable-alphabet bug that produced impossible negative MI was found and fixed); observed-support add-one for three-way quantities, always read against the null floor with the pre-declared 2× count-noise flag (the #459 N3 pattern).

## Measured result (fixture corpus, 500k records / 2507 stories)

- **MI(t₋ᵢ; next)**: thin and decaying — 0.026 bits at offset 1 → 0.003 at offset 8; null floor flat at 0.032. Each context token individually carries hundredths-of-a-bit about next.
- **MI(t₋ᵢ; t₋ⱼ)**: adjacency dominates — all 7 adjacent pairs top the table at ~0.082 bits, falling with |i−j|.
- **II(t₋ᵢ; t₋ⱼ; next)**: **every evaluated triple is in the count-noise regime** (15/15 flagged, |II_obs| ≤ 2×|II_null|). Null-corrected ranking surfaces adjacent pairs but their excess is negative; (1,2)'s raw II is 0.0046 bits — no measurable synergy between the two most recent tokens. Raw rankings are cardinality-bias artifacts (wider offsets → more atoms → more plug-in bias); the null-corrected reading is the honest one.

**Verdict (pre-declared rule): 0 of 15 pairs qualify as block-selection evidence.** Pairwise context synergy is not measurable at D3 scale — consistent with #459's count-limited k=3 bound (estimation error there is pure counting noise). The estimable k=2 evidence: dependency is *local* (adjacency) and *thin* (hundredths of a bit).

Consequence for the #458 program: interaction blocks over token-position variables are not selectable at this corpus scale — the spectrally-seeded cover experiment (gated on PR #462's reconstruction metric) and the #320 corpus/teacher upgrade are the remaining levers. Null checks N1–N3 PASS.

## Verification

Four local gates green (rustup-pinned toolchain): workspace tests, clippy `-D warnings`, fmt, no_std ladder. Harness runs on fixtures in ~12.5s release; two consecutive runs bit-identical; default test run unaffected.

Refs #458, #459, TF1 S-20 (interaction-block necessity — note the tension this measurement sharpens: the theory says interaction blocks are load-bearing, the corpus says their signal is below the counting-noise floor at D3 scale). Out of scope: add-half-smoothing robustness re-run, conditional-MI formulation, offsets >8, teacher-argmax targets — easy follow-ups if wanted.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

