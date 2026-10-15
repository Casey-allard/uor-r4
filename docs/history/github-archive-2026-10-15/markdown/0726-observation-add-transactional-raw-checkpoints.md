# #726 — observation: add transactional raw checkpoints

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-15T07:09:07Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-15T07:29:52Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/726
- **Merged:** 2026-08-15T07:29:52Z
- **Merge commit:** cf7b8988e56b4baac5427d213bd7c1a5026769d4
- **Base / Head:** main / UOR-Foundation:issue-725-transactional-raw-checkpoint
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add the canonical `R4RAWCOM` schema-1 `raw-committed.bin` checkpoint, binding teacher state, observation identity/layout, and every shard’s committed record count
- synchronize shard, probability, and trace payloads before atomically publishing the authoritative checkpoint, then publish the existing 25-byte `state.bin` compatibility mirror
- recover through a two-phase all-entry preflight, truncating tentative aligned tails only after every shard and companion validates
- repair missing or corrupt state mirrors, resume interrupted finalization/merge idempotently, and fail closed for unfinished legacy or mixed raw/text directories
- retain fully finalized, kappa-validated legacy bundles without migration or historical byte changes
- document the structural guarantee, filesystem assumption, canonical footprint, and small-fixture overhead measurement

No teacher arithmetic, observation-row wire format, trace format, routing, deployed runtime, or kappa fixture changes are included.

## Verification

- `cargo test -p uor-r4-graph-compiler --offline`
  - 131 unit tests; 26 induction tests with 1 existing release-only ignore; 39 observation tests with 1 manual measurement ignore; 4 route-fit tests
- `cargo test -p uor-r4-graph-cli --lib --offline` — 78 passed
- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all --check`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- `python3 scripts/check_claim_wording.py`
- `cargo test -p uor-r4-graph-compiler --offline --test observe raw_checkpoint_small_fixture_overhead_measurement -- --ignored --nocapture`
  - 4 shards; 120-byte authoritative checkpoint; 25-byte mirror
  - median 16,612,542 ns; mean 16,473,677 ns across 31 publications; diagnostic only, no threshold

Closes #725

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

