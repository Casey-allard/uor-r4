# #458 — Interaction-information block search seeded by the spectral 4-block decomposition

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-06T13:10:49Z
- **Updated:** 2026-08-08T10:28:20Z
- **Closed:** 2026-08-08T10:28:20Z
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/458
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation

Cover blocks today come from *geometric* induction over signature space (`induction.rs`): regions are induced by where contexts land, not by which variable groups carry dependency. Two pieces of evidence say block selection should be information-driven:

1. TF1 already proves the necessity of interaction blocks: the spectral operator M gives a canonical 4-block decomposition of the carrier — global (dim 1), modality (T−1), context (O−1), interaction ((T−1)(O−1)) — and D-1/S-20 record that the per-block form is strictly weaker precisely because it discards the interaction block (15 of 24 dimensions). The theory knows higher-order blocks are load-bearing; the compiler doesn't use that knowledge.
2. #446 measured a 2-way interaction block (⟨left, right⟩ distribution classes): +14.9pp construction-side and strongly positive bits, but NEGATIVE top-1 at serving because the right context is latent and must be marginalized. That is exactly the RA distinction between information present in a block and information reconstructable under the query regime — block selection must optimize the latter.

## Proposal

1. Measurement: interaction information (3-way and selected higher-order) over Atlas-coordinate variable groups (left-context positions, scope/modality/context coordinates, refinement depth) on the D3 corpus — which variable groups carry dependency beyond what the cover already captures?
2. Experiment: covers seeded or constrained by the spectral 4-block structure vs. purely geometric induction, scored on reconstruction bits with EXCT disabled (the #456 metric), with a shuffled-variable null arm.

## Pre-registered exit criteria

- A table of measured interaction information by variable group, κ-pinned, with the falsifier (shuffled variables must show ~0 interaction).
- Success = a spectrally-informed cover that beats geometric induction on EXCT-disabled reconstruction bits by a pre-declared margin, or a recorded NEGATIVE identifying whether the interaction information is present but serving-unreachable (the #446 pattern) vs. absent.

Refs: #446 (two-sided arms), TF1 S-9..S-13, S-20 (spectral block necessity), docs/addr_prism_correspondence.md.


## Comments (1)

### comment 5225704238 — 2026-08-08T10:28:19Z — Casey-allard

Closing — delivered by **@auser** in PR #464 (merged 2026-08-06); the record was never closed. This is a **NEGATIVE result**, recorded in full so the branch can be retraced if it becomes relevant again (e.g. after a #320 corpus/teacher upgrade).

**Result (fixture corpus, 500k records / 2507 stories).** The interaction-information harness `crates/uor-r4-graph-certify/tests/interaction_information.rs` measures which context variable groups carry dependency:

- **MI(t₋ᵢ; next)** is thin and decaying: 0.026 bits at offset 1 → 0.003 at offset 8; null floor flat at 0.032. Each context token individually carries hundredths-of-a-bit about `next`.
- **MI(t₋ᵢ; t₋ⱼ)**: adjacency dominates — all 7 adjacent pairs top the table at ~0.082 bits, falling with |i−j|.
- **II(t₋ᵢ; t₋ⱼ; next)**: **every evaluated triple is in the count-noise regime** (15/15 flagged, |II_obs| ≤ 2×|II_null|); (1,2)'s raw II is 0.0046 bits — no measurable synergy between the two most recent tokens. Raw rankings are cardinality-bias artifacts; the null-corrected reading is the honest one.

**Verdict (pre-declared rule): 0 of 15 pairs qualify as block-selection evidence.** Pairwise context synergy is not measurable at D3 scale — consistent with #459's count-limited k=3 bound. The DoD's present-but-unreachable-vs-absent question resolves to **below the counting-noise floor at this scale** (not a serving-reachability problem — there is no measurable signal to seed on). Null checks N1–N3 PASS.

**Tension worth keeping (TF1 S-20):** the spectral theory says interaction blocks are load-bearing, but the corpus says their signal is below the counting-noise floor at D3 scale. That tension is the case for #320 (more corpus/teacher) — the remaining lever for interaction blocks — rather than the token-position block search, which this measurement closes as not-selectable at current scale.



---
_Comment/review ids in this file: 5225704238_

