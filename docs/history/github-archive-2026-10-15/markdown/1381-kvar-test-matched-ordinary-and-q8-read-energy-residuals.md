# #1381 — KVAR: test matched ordinary and Q8 read-energy residuals

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T12:56:22Z
- **Updated:** 2026-10-09T13:17:37Z
- **Closed:** 2026-09-24T18:15:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1381
- **Merged:** not merged
- **Merge commit:** c14680a514052cb4ebfc80f6b2c1475c3efba60a
- **Base / Head:** codex/kvar-hard-select-geometry-20260924 / UOR-Foundation:codex/kvar-relative-energy-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- Runs the pre-registered matched C8 versus exact Q8 query/value read-energy residual on the retained KVAR memory, with transport-off and exact kernel-off replay.
- Preserves a fresh sealed seed-group-4 report, fitted tables, per-prediction scores, controls, executable/source hashes and resource projection.
- Records the negative result and narrows the next dependency to native-language memory integration. This is a finite residual pilot; the original full `(f)/(h)` arms remain `NOT_RUN`.

## Result

The final panel is valid (order-2 count 2/204; hand-coded overwrite 204/204). Q8 minus ordinary bits/query is -0.1337 [-0.3708, +0.0980] for seed 1 and 0 [0, 0] for seed 2. The pre-declared >=0.5-bit, two-seed geometry gate fails. The unmodified `(c)` model is better than either residual for seed 1; seed 2 learns zero kernels. No geometry, language or energy advantage is claimed.

## Verification

- `cargo test -p uor-r4-core --bin kvar-recall --offline` (12/12)
- `cargo build -p uor-r4-core --bin kvar-recall --release --offline`
- Quick/development panels and one fresh final panel; kernel-off exact replay on all final rows
- `cargo fmt --check`
- `python3 scripts/check_claim_wording.py`
- `git diff --check`

Stacked on #1380. References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5819631963 — 2026-09-24T18:15:50Z — Casey-allard

Delivered as part of owner-authorized consolidated PR #1386, merged through the protected main queue at `b020f34a09816ba235984cedde1c50f4c8e9cae2`. The main tree equals reviewed stack head `d1f5524c1d2b302cf2c07fea6b47309460b85f66` exactly (tree `c68b8b332080d09884b27191731b56296dc4a1f4`). Closing this component PR as incorporated; its experimental limitations remain as recorded. Programme #820 and #973 remain open.



---
_Comment/review ids in this file: 5819631963_

