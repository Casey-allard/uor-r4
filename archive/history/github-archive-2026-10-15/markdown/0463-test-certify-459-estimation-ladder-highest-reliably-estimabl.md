# #463 — test(certify): #459 — estimation ladder: highest reliably estimable interaction order at D3 scale

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-06T14:16:23Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-06T14:39:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/463
- **Merged:** 2026-08-06T14:39:21Z
- **Merge commit:** 59059f253f0e9c6d5d58b3d9834bcb4e073c8eb2
- **Base / Head:** main / UOR-Foundation:issue-459-estimation-ladder
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Closes the measurement half of #459: an `#[ignore]`d estimation-ladder harness (`crates/uor-r4-graph-certify/tests/estimation_ladder.rs`) that names the highest reliably estimable interaction order at D3 corpus scale — the input #458 (interaction-information block search) needs before spending effort on higher-order blocks.

Per (k, fraction) over story-bounded context windows, k = 1..4, prefix subsample ladder {1/16, 1/8, 1/4, 1/2, 1} (story-run-boundary truncation per `mc1_subsample_corpus.py`):

- add-one-smoothed KL(full ‖ subsample) in bits
- atom sparsity (fraction of full-corpus atoms below counts {1, 5})
- unique-atom counts

Null arm (fixed-seed shuffled `next` stream): unigram-marginal invariance + KL-identity at f=1, both PASS. Fully deterministic — prefix truncation, exact u128 atom keys (k≤4 collision-free), xorshift64* shuffle; output bit-identical across runs. Vacuous-skip when fixtures are absent (κ-test convention); `R4_CORPUS_META`/`R4_CORPUS_RECS` env overrides.

## Measured result (fixture corpus, 500k records / 2507 stories)

KL(bits) full‖sub at f=1/2: **k=1: 0.0123** (under the pre-declared 0.05-bit floor) · k=2: 0.0626 · k=3: 0.0917 · k=4: 0.0942. Sparsity at f=1/2 (% atoms unseen): k=1 14.4%, k=4 45.0%.

Verdict: **only k=1 clears the floor at half-corpus rigor**; k=2..4 reach it within the full corpus (multipliers 0.63–0.94×, interpolations not extrapolations). Key finding for #458: at k=3 the observed KL (0.0917) ≈ the null's (0.0922) — estimation error is **pure counting noise**, so more corpus (the #320 lever), not better estimation, is what higher-order blocks need. At k=2 the joint is *better* estimated than an independence table — structure concentrates atoms.

## Verification

Four local gates green (rustup-pinned toolchain): workspace tests, clippy `-D warnings`, fmt, no_std ladder. Harness executes on fixtures in ~0.7s (`--release --test estimation_ladder -- --ignored`), does not affect the default test run.

Refs #459, #458, TF1 D-4. Out of scope (easy follow-ups if #458 wants them): k=5 (needs hashed keys, exceeds u128 packing), teacher-argmax target variant.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

