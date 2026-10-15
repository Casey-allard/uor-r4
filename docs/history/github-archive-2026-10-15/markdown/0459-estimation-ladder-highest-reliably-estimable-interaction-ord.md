# #459 — Estimation ladder: highest reliably estimable interaction order at D3 corpus scale

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-06T13:12:05Z
- **Updated:** 2026-08-08T10:28:18Z
- **Closed:** 2026-08-08T10:28:18Z
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/459
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation

RA's known limitation is ours: higher-order block marginals need sample size, and TF1 D-4 already records the symptom — `ρ̂_d` is NOT monotone and degrades once atoms become sparse, and past the injectivity depth `ρ*_d = 0` by memorization while carrying no information. Before investing in higher-order interaction blocks (#458), we need to know which interaction orders are even *estimable* at D3 corpus scale with the SmolLM2-135M teacher — and what #320 (teacher/corpus upgrade) would buy.

## Proposal

An estimation-ladder certificate (certifier-side, measurement only):

1. Subsample ladder over the construction corpus (e.g. 1/16 … 1/1, extending `scripts/mc1_subsample_corpus.py`): for k-way marginals over the variable groups measured in #458 (k = 1, 2, 3, selected 4), report marginal estimation error (KL between subsample-estimated and full-corpus marginals) and atom-sparsity (fraction of atoms below support thresholds) vs. corpus size.
2. Extrapolate: the corpus size at which each order k crosses a pre-declared reliability floor.
3. Output: a "highest reliably estimable interaction order at D3 scale" certificate, feeding the block search and the #320 teacher-upgrade decision.

## Pre-registered exit criteria

- κ-pinned ladder report; null arm: same-order marginals over shuffled variables must show estimation error consistent with independence.
- The certificate names the highest k whose marginals are reliable at current scale, and the corpus multiplier needed for k+1 — no extrapolation without the ladder data behind it.

Refs: #320 (teacher upgrade), #446 (interaction blocks), TF1 D-4 (sparsity degradation), docs/formal_vocabulary.md.


## Comments (1)

### comment 5225704140 — 2026-08-08T10:28:17Z — Casey-allard

Closing — delivered by **@auser** in PR #463 (merged 2026-08-06); the record was never closed. Recorded here so GitHub reflects the true state.

**Result (fixture corpus, 500k records / 2507 stories).** The estimation-ladder harness `crates/uor-r4-graph-certify/tests/estimation_ladder.rs` names the highest reliably estimable interaction order at D3 scale. KL(full‖sub) at f=1/2: **k=1: 0.0123** (under the 0.05-bit floor), k=2: 0.0626, k=3: 0.0917, k=4: 0.0942. Only **k=1 clears the floor at half-corpus rigor**; k=2..4 reach it within the full corpus (multipliers 0.63–0.94×, interpolations not extrapolations). Null arm (shuffled `next`): unigram-marginal invariance + KL-identity at f=1, both PASS.

**Key finding for the program:** at k=3 the observed KL (0.0917) ≈ the null's (0.0922) — estimation error is **pure counting noise**, so more corpus (the #320 lever), not better estimation, is what higher-order blocks would need. At k=2 the joint is *better* estimated than an independence table (structure concentrates atoms). This is the estimability bound that #458 needed and that DoD (name the highest reliable k + the multiplier for k+1) required.

Exit criteria met (κ-pinned ladder + null arm + named bound). Optional follow-ups noted in the PR (k=5 needs hashed keys; teacher-argmax target variant) are not required for this issue — file separately if a later arm wants them.



---
_Comment/review ids in this file: 5225704140_

