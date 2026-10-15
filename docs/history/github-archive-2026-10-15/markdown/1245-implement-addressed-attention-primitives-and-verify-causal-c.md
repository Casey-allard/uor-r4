# #1245 — Implement addressed attention primitives and verify causal credit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T06:29:11Z
- **Updated:** 2026-10-09T13:17:05Z
- **Closed:** 2026-09-13T06:30:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1245
- **Merged:** 2026-09-13T06:30:23Z
- **Merge commit:** 7fd1f60d1a479ac894c9a5c370c41904f91306ce
- **Base / Head:** main / UOR-Foundation:codex/addressed-attention-primitives
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The addressed-attention specification had no executable implementation of its discrete credit or exact object protocol. This adds isolated native Rust LUT4/export, causal feature-packing, exact lease/offer/publication and offline sampling/leave-one-out primitives. Existing retained-model dispatch and source-bound loaders stay compatible.

The release primitive gate passes 18/18 tests. Its H4 fixture selects actual ObjectSession records, changes the second parameter row after the first read and matches analytic, event-score and finite-difference gradients. A second fixture verifies immediate and later emission credit through actual acknowledgment/cancellation. Other checks cover every input's circuit reach, deterministic export/legal fallback, scalar extremes and ordered/dependent operations, eviction, wrong/consumed offers, one-time KEY publication and bounded pending/ready/saturated snapshots.

Validation executed locally:
- `cargo test --offline --release -p uor-r4-core --lib addressed_attention -- --test-threads=1 --nocapture`: 18 passed, zero failed/ignored.
- `cargo fmt --all --check`, claim wording, diff whitespace, 217 local links and source/evidence binding checks pass.
- Original checkouts, retained model, previous binary and 21 prior sealed roots / 733 files verify unchanged. Of 141 prior source bindings, only the new module declaration changes.

Compilation/tests charged 101,708 / 240,000 ms, with 2.57 GB sampled peak RSS; no extension, paid compute or cleanup. The executable is retained separately from the build cache.

This is a primitive/credit result. The full eight-phase model, normal model artifact and corpus learner remain unimplemented; language fitting, generation qualification and serving allocation/latency/energy are not run. Retain 15baec48; no promotion. The result documents the dense caller-array footprint and namespace/checksum trust boundaries. Next is full forward integration and one measured frozen-parameter forward/backward dry-run before selecting a training dose.

References #973. References #964. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

