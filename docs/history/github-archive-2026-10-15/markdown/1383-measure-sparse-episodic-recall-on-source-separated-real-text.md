# #1383 — Measure sparse episodic recall on source-separated real text

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T15:10:52Z
- **Updated:** 2026-10-09T13:17:50Z
- **Closed:** 2026-09-24T18:15:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1383
- **Merged:** not merged
- **Merge commit:** 3e60107ccbf21d03ea481c40a016b35fe36d84b7
- **Base / Head:** codex/addressed-lexical-bridge-20260924 / UOR-Foundation:codex/realtext-sparse-recall-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Scope

References #820. Stacked on #1382. This is the next architecture decision after the KVAR synthetic memory result and the explicit-address loaded lexical bridge: test whether a sparse, causal exact-pair read has useful **real-text** signal beyond a newly fitted count-family reference. It is a diagnostic, not a default language-model change.

## Frozen method and result

- Source-hash-separated project Markdown, pinned 4096-token BPE: 400,000 Fit tokens, 100,000 Tune tokens, 24 exposed Dev documents, 87,564 scored positions. The selected document paths and SHA-256 identities are in the sealed receipt; source tree at the run was `09d5c74e`.
- C fits unigram/order-1/order-2 counts on Fit and tunes interpolation on Tune. A within-document exact `(previous,current)` key retrieves only a previously observed successor. A dyadic age-bucket mixture is fitted on Tune, then frozen for Dev. The same-bucket candidate-rotation null retains hit positions and weights.
- Dev bits/target: **C 6.410664**, **cache mixture 6.173020**, **rotated null 6.531288**. Cache-minus-C is **-0.237644 [-0.292814, -0.190741]** by document bootstrap. Distant (>=64 token) hits cover 24.23% of targets; 4.08% of all targets have a correct distant cached successor when C top-one is wrong. Every selected document improves.
- All age buckets chose the same 1/4 weight. This supports the exact sparse read signal but gives no evidence for an age-dependent learned admission policy. The project-document Dev population is exposed and mostly integration reports. No loaded TlModel, fresh general prose/code transfer, geometry comparator, D0-b/D5 served kernel or energy result is claimed.

## Evidence and direction

One offline Rust release build and one bounded sealed run; all 87,564 per-position records retained at `/Users/casey.allard/uor-r4-investigations/realtext-sparse-recall-20260924/final1`. The CSV was independently reaggregated, and formatting, diff and claim-wording checks passed. No lib-test suite or training campaign was run. See `docs/integration/realtext-sparse-recall-result-2026-09-24.md`.

The next implementation is to connect a bounded addressable candidate to the **same loaded native sequence model** and learn source admission/read choice from observed text. Compare against this ordinary exact-pair cache and C at matched access on source-separated prose and Rust code with actual continuations, NoRead/source edits and per-token access counts. Only then test H4/Hamiltonian routing against an equally informed ordinary comparator. This avoids spending the next cycle on authored copy-fixture repair.

Research basis: [Zoology](https://arxiv.org/abs/2312.04927), [Large Memory Layers with Product Keys](https://arxiv.org/abs/1907.05242), [Titans](https://arxiv.org/abs/2501.00663). Their mechanisms are not imported as evidence of this model's capability.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5819633387 — 2026-09-24T18:15:56Z — Casey-allard

Delivered as part of owner-authorized consolidated PR #1386, merged through the protected main queue at `b020f34a09816ba235984cedde1c50f4c8e9cae2`. The main tree equals reviewed stack head `d1f5524c1d2b302cf2c07fea6b47309460b85f66` exactly (tree `c68b8b332080d09884b27191731b56296dc4a1f4`). Closing this component PR as incorporated; its experimental limitations remain as recorded. Programme #820 and #973 remain open.



---
_Comment/review ids in this file: 5819633387_

