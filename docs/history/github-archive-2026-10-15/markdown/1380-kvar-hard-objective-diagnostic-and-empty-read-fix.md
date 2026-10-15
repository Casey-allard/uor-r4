# #1380 — KVAR hard-objective diagnostic and empty-read fix

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T11:13:44Z
- **Updated:** 2026-10-09T13:17:37Z
- **Closed:** 2026-09-24T18:15:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1380
- **Merged:** not merged
- **Merge commit:** 46aabcdf8aa786f02a174b8698f37b3e1a7b3284
- **Base / Head:** main / UOR-Foundation:codex/kvar-hard-select-geometry-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- Fix hard served reads so an unwritten row cannot inject token zero. Preserve the historical unscaled read-gate update as an explicitly named optimization surrogate, with the exact derivative available and finite-difference tested.
- Add hard straight-through and mixed-objective diagnostic modes, fresh held seed groups, retained quantized parameters and per-row predictions, and partial-arm receipt labeling.
- Execute the locked KVAR successor. Direct/staged hard training underperform; the mixed hard auxiliary does not improve the retained soft-trained, hard-served candidate on fresh seed group 3. The complete `(a,b,c)` root still accepts the bounded addressed-memory mechanism.

## Evidence

- Complete sealed group-3 root: `/Users/casey.allard/uor-r4-investigations/kvar-20260924/final-complete-sg3-v2`. C is 4/204 near chance; hand-coded overwrite is 204/204. `(c)` serves at 0.8284/0.7451 accuracy and +3.8332/+3.5849 bits over C for two seeds; `(a)` and `(b)` remain near chance.
- Mixed-objective comparison root: `final-aux-sg3`. It is worse by 1.2915 and 0.1064 bits/query on the paired fresh rows, so it is not promoted. Negative development attempts are preserved.
- The complete replay's receipt, served parameters and individual predictions are byte-identical to the earlier full group-3 run. Source/executable hashes and limits are in the result record.

## Validation

- `cargo test -p uor-r4-core --bin kvar-recall --offline` (10 passed)
- `cargo build -p uor-r4-core --bin kvar-recall --release --offline`
- `cargo fmt --check`
- `python3 scripts/check_claim_wording.py`
- `git diff --check`

## Scope

Synthetic keyed retrieval only. Ordinary matched `(f)`, geometric `(h)`, transport-off ablation, language-path integration, whole-path energy, and general prose/coding qualification remain open. The served prototype still inspects 8,771 parameter slots per token. References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5819631273 — 2026-09-24T18:15:47Z — Casey-allard

Delivered as part of owner-authorized consolidated PR #1386, merged through the protected main queue at `b020f34a09816ba235984cedde1c50f4c8e9cae2`. The main tree equals reviewed stack head `d1f5524c1d2b302cf2c07fea6b47309460b85f66` exactly (tree `c68b8b332080d09884b27191731b56296dc4a1f4`). Closing this component PR as incorporated; its experimental limitations remain as recorded. Programme #820 and #973 remain open.



---
_Comment/review ids in this file: 5819631273_

