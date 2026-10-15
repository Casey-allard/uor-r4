# #457 — Consistent reconstruction operator for overlapping blocks: IPF-style marginal consistency vs backoff+sum

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-06T13:10:46Z
- **Updated:** 2026-08-08T17:36:16Z
- **Closed:** 2026-08-08T17:36:16Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/457
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation

Overlapping cover regions contribute to the score by *summing* residuals: `S(v) = B(v) + ΣΔE(n,v) + ΣΔT(m,v) + ΔX(X,v)`. The confirmed double-counting bug (#64) — literal Σ-over-cloud scoring collapsed to 0.3% top-1 — forced the Rule 1 chain-telescoped residual + Rule 2 EXCT-precedence redesign. That redesign fixed the arithmetic but did not install a *consistency* operator: nothing guarantees the reconstructed joint's block marginals match the fitted ones.

Reconstructability Analysis uses iterative proportional fitting (IPF) over overlapping block marginals: the reconstruction operator iterates until the model's marginals on every block equal the empirical marginals, yielding the maximum-entropy joint consistent with the blocks. Backoff-plus-sum is a particular non-iterative reconstruction with no such guarantee.

## Proposal

A certifier-side comparison harness (measurement first, no compiler change):

1. Arm A (baseline): current Rule 1 + Rule 2 chain-telescoped scoring, EXCT disabled.
2. Arm B: IPF-consistent reconstruction over the same cover's overlapping region marginals (region = block, emission table = block marginal), same EXCT-disabled regime.
3. Null arm: shuffled block marginals — must sit at the unigram floor.
4. Report held-out bits/token and top-1 per arm on the pinned D3 fixtures, κ-pinned.

## Pre-registered exit criteria

- If Arm B beats Arm A by a pre-declared bits margin, file the compiler-side follow-up to make the consistency operator the scoring semantics (format impact: EMIT residual semantics, RFC amendment).
- If NEGATIVE, record the measured marginal-inconsistency (per-region KL between model-implied and empirical marginals) so the next arm knows whether inconsistency was even large enough to matter.

Refs: #64 (double-counting redesign), docs/transformerless/R4G1.md §9.5 (EMIT/EXCT semantics), TF1 D-4.


## Comments (2)

### comment 5225704465 — 2026-08-08T10:28:23Z — Casey-allard

Unassigning @auser and returning this to the open pool (no activity since 2026-08-06; no PR; not currently being worked).

**State: unstarted.** No harness exists yet (`#457` is referenced nowhere in the code). The issue is a well-specified, measurement-first certifier comparison: Arm A (current Rule 1 + Rule 2 chain-telescoped scoring, EXCT disabled) vs Arm B (IPF-consistent reconstruction over the same cover's overlapping region marginals), plus a shuffled-block null arm, reporting held-out bits/token and top-1 on the pinned D3 fixtures.

Context for whoever picks it up: the sibling RA measurements have now landed — #456 (reconstruction-bits certificate), #458 (interaction information: NEGATIVE, no synergy at D3 scale), #459 (estimation ladder: only k=1 reliable at half-corpus; k≥3 is counting noise). #459's bound matters here: if IPF's consistency gain lives in higher-order marginals, it may be count-limited at D3 scale, so the null-corrected reading and the per-region model-vs-empirical KL (the NEGATIVE-branch deliverable) are the important outputs regardless of direction.


### comment 5226115592 — 2026-08-08T12:32:15Z — Casey-allard

Cheap-check landed in PR #499 (the exit criteria's marginal-inconsistency deliverable, before committing to Arm B). **Kept open** for the definitive Arm B.

Measured on the default cover, 5000 held-out of the 500k fixture — and it **reversed** my prediction. Positions activate a mean of **7.52 regions** (heavy overlap). Over the root-mass + residual support:

| | mean pairwise JS | argmax agreement |
|---|---:|---:|
| real co-activated regions | 0.5511 bits | 0.3468 |
| null random regions | 0.6019 bits | 0.2611 |

The falsifier passes (real > null), but the absolute disagreement is **large**: a position's ~7.5 blocks concur on the next token only ~35% of the time. So the marginal inconsistency is **not too small to matter** — the "overlapping regions concur, so IPF is moot" hypothesis is refuted, and an IPF **Arm B is warranted**.

The caveat that keeps this from being a positive: #456 measured the reconstruction that combines these regions to be **sub-unigram**, and #459 that k≥3 marginals are **counting noise**. So this is very likely disagreement among near-noise residuals — IPF would reconcile them into a consistent-but-noisy joint, not into signal. The definitive test remains **Arm B** (IPF-consistent reconstruction vs backoff+sum, held-out bits/top-1), likely negative but not settled here. Unassigning; left open for whoever builds Arm B on top of #499's harness.



---
_Comment/review ids in this file: 5225704465, 5226115592_

