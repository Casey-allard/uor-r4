# #723 — research: run Octeract route-trace screen

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-15T02:46:01Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-15T03:07:17Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/723
- **Merged:** 2026-08-15T03:07:17Z
- **Merge commit:** 3941aa53c06e8101ce1a19fbddf1b343a010faf5
- **Base / Head:** main / UOR-Foundation:issue-722-octeract-trace-screen
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- freeze the preregistered Octeract route-trace screen contract and canonical report schema for #661-B
- implement the collision/reachability arms, three typed nulls, exact collision oracle, fail-closed input validation, and closed evidence registry
- append the Phase B evidence record without changing the deployed runtime, attention operator, or artifact format

## Result

No registered pinned-real `full/1` route-trace evidence exists, so the empirical screen terminates canonically as `UNAVAILABLE`; synthetic/manual rows remain instrument-conformance evidence and cannot be promoted to empirical PASS/FAIL.

- canonical report: 6,587 bytes
- payload kappa: `blake3:8b8c3bdc41f04ac2d6b9a15ef843f5064fae92e8b6bfe57cafbc6803eca7c5a2`
- final envelope kappa: `blake3:eab7b1bb12d9508d9815da0c4fbac248eab8b93b8258e717829542e41ac75e5e`
- disposition: retain the mechanism as dormant/unavailable; do not open #661-C or activate the negative-only fallback

Every evaluated arm/null owns the counts for the domain it actually consumed. The discriminating fixtures pin the narrower prefilter domain and the deranged-support cardinality separately from the matched base domain.

## Verification

- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all --check`
- `cargo check -p uor-r4-graph-format --no-default-features`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`
- `cargo test -p uor-r4-graph-certify --test octeract_trace_screen_661 --offline` (24 passed)
- non-vacuous canonical Gate E with `/tmp/ref/out/model.bin` present (3 passed)
- `cargo deny check bans`
- `python3 scripts/check_claim_wording.py`
- `cargo run -q -p xtask -- check-model`
- `cargo test -q -p repo-conformance`
- `cargo run -q -p xtask -- audit-deferral`
- `cargo run -q -p xtask -- audit-limits`
- `git diff --check`

Closes #722
References #661


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

