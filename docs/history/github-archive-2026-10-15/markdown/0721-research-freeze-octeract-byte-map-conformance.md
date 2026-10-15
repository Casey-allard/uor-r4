# #721 — research: freeze Octeract byte-map conformance

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-15T00:53:25Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-15T01:12:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/721
- **Merged:** 2026-08-15T01:12:26Z
- **Merge commit:** 598cba902c6d16fb9d7687601744728856a4197f
- **Base / Head:** main / UOR-Foundation:issue-720-octeract-conformance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- bind the two supplied Octeract research inputs by exact metadata/displayed title, filename, byte size, SHA-256, provenance, audit date, and `NOASSERTION` license state without redistributing either PDF
- add an independent digit-sort/subtract oracle and a separate closed-form byte map
- make valid byte weights, block distances, shells, and oriented classes explicit bounded value types whose admitted operations are total
- exhaust the complete byte and full-mask pair domains, partial masks, all 45 canonical oriented states, anchor relabeling, and deterministic 288-bit composition
- document source corrections and keep the result certifier-only: no operator, runtime, artifact, default, or kappa mutation

## Evidence

- all 256 bytes reproduce the published five-output map
- all 65,536 full-mask pairs satisfy the folded/oriented relations
- 47,616 non-equator complement pairs collide under the lossy fold; 17,920 equator pairs do not
- the partial-mask lower bound is strict in 213,862 cases
- the deterministic 288-bit example reconstructs exact distance 146; the lossy fold totals 106

## Verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo test --workspace --offline`
- `cargo check -p uor-r4-graph-format --no-default-features`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`
- `TLESS_CANONICAL_DETERMINISTIC=1 cargo test -p uor-r4-core --release --offline --test kappa_reproduction -- --ignored` (3/3, non-vacuous `/tmp/ref/out/model.bin`)
- `cargo run -q -p xtask -- check-model`
- `cargo run -q -p xtask -- audit-deferral`
- `cargo run -q -p xtask -- audit-limits`
- `cargo test -q -p repo-conformance`
- `cargo deny check bans`
- `python3 scripts/check_claim_wording.py`
- `git diff --check`

Closes #720.
References #661.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

