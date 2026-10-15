# #350 — test: calibrate session signature by teacher entropy

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T21:25:01Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-08-02T21:41:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/350
- **Merged:** 2026-08-02T21:41:23Z
- **Merge commit:** 2e34f63ea1e60d7afd72db228437764642710f4b
- **Base / Head:** main / UOR-Foundation:issue-247-session-calibration
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- extend the #247 held-out session-signature A/B harness with optional aligned teacher probability sidecars
- report full-corpus and sampled held-out teacher bits/token
- partition session corrections, regressions, prediction changes, and teacher-argmax agreement by deterministic teacher-entropy quartile
- keep all calibration math in the compiler/evaluation side; the graph runtime and serving path are unchanged

## Usage

```bash
cargo bench -p uor-r4-graph-runtime --bench session_signature -- \
  <bundle-root> 256 <probability-observation-dir>
```

The probability directory must contain `manifest.json` and aligned `*.prob` sidecars. The harness verifies sidecar/corpus row alignment.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo test -p uor-r4-graph-compiler --offline`
- `cargo test -p uor-r4-graph-runtime --offline`
- graph-format no-std checks with and without `alloc`
- full workspace tests reach the existing R4G1 allocation-census failure: 2 allocations / 594 bytes in the pre-existing kernel census

Refs #247.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

