# #346 — feat: compile transformerless artifacts from recorded observations

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T20:01:10Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-02T20:18:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/346
- **Merged:** 2026-08-02T20:18:11Z
- **Merge commit:** b4622a217512039f5707af44c8957a44d352f9e4
- **Base / Head:** main / UOR-Foundation:issue-346-observation-first-compile
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add an observation-first transformerless compiler path that consumes completed `corpus.meta` / `corpus.records` directly
- derive deterministic native token representations from recorded top-k next-token distributions without constructing a teacher or invoking model forward/matmul code
- parallelize Hamming calibration with ordered integer reductions and use the existing parallel store builder
- expose the path through `r4 compile --corpus-meta ... --corpus-recs ... --vocab-size ...`
- document the capture-vs-compile boundary and add regression coverage for records without a hidden sidecar

## Why

The previous `compile --source` workflow coupled corpus generation and artifact compilation, so a 360M compile spent hours executing the CPU Hugging Face transformer. The reference `uor-semantic` project makes capture an optional offline boundary and keeps artifact compilation observation-only. This PR brings that boundary to uor-r4.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- focused core and graph-cli tests
- end-to-end release compile of the completed 200k-record fixture: 48.5s wall time, no teacher loaded

The full workspace test reaches the existing long-running allocation census; the change does not touch the deployed runtime.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

