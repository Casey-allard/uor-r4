# #307 — Add teacher parity benchmark suite

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T20:44:51Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-07-31T21:47:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/307
- **Merged:** 2026-07-31T21:47:02Z
- **Merge commit:** d0a9ec6c7753700c94098088d78a9a73514455d1
- **Base / Head:** main / UOR-Foundation:issue-303-hopf-sector-occupancy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Adds a BDD teacher-parity and benchmark suite for the legacy transformerless TLS runtime and the R4G1 graph runtime.

The suite records Blake3 provenance for teacher, artifact, store, graph, and corpus inputs; measures teacher-forced accuracy, generation speed, kernel operation constraints, steady-state allocations, witness consistency, and in-distribution corpus replay.

When pinned model fixtures are absent, scenarios follow the repository's existing vacuous-skip convention. Thresholds are empirical criteria, not equivalence claims.

## Measured validation

Using the local pinned SmolLM2 fixtures:

- S1 provenance: all inputs carry Blake3 kappas; graph provenance matches the compiled artifact
- S2 legacy replay: 96 positions, top-1 0.0104, top-8 recall 0.1771, Δbits 9.212
- S3 R4G1 replay: 96 positions, top-1 0.0104, top-8 recall 0.0521, Δbits 11.463, 3 abstentions
- S4 speed: legacy 40.9× and graph 10.4× the teacher in the workspace run
- S5 kernel: zero multiply/divide operations, zero steady-state allocations, witness/plain predictions agree
- S6 corpus replay: legacy top-1 0.4287; graph top-1 0.4436 with 11 abstentions

## Gates

- `cargo fmt --check`
- `git diff --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo test --workspace --offline`
- `cargo check -p uor-r4-graph-format --no-default-features`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`

All passed.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

