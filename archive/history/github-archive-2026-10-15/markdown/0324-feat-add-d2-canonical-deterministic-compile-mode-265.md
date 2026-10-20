# #324 — feat: add D2 canonical deterministic compile mode (#265)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T01:23:14Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T03:54:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/324
- **Merged:** 2026-08-01T03:54:37Z
- **Merge commit:** 3b70d283f245635e089ade42cc96e204ffb316ed
- **Base / Head:** main / UOR-Foundation:issue-265-d2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #265.

## Summary

- adds `--canonical-deterministic` for certificate-bearing compile runs
- disables accelerated teacher matmul and routes teacher transcendental math through portable pure-Rust `libm`
- applies the same mode to transformerless softmax/packing/projection and graph-cover normalization/entropy
- documents the mode and keeps `--exact-scalar` as the faster local-iteration option

## Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --offline`
- workspace clippy with `-D warnings`
- graph-compiler tests: 80 unit + 20 integration passing, with one fixture workload ignored
- model-source tests: 4 passing
- core library tests, including canonical-mode run: 19 passing
- graph-format no-std checks: default-off and `alloc`
- claim-wording gate passing

The full workspace test command still hits the pre-existing R4G1 allocation census failure: 2 allocations / 594 bytes in the runtime steady-state assertion, unrelated to this compiler-side change.

The final #265 acceptance proof remains a macOS/Linux differential compile of the same pinned model and corpus, followed by maintainer-reviewed canonical fixture re-pinning. This PR supplies the canonical mode needed for that proof; it does not claim that cross-OS result yet.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

