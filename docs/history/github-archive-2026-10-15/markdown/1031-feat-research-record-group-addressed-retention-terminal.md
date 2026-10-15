# #1031 — feat(research): record group-addressed retention terminal

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T04:39:13Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-09-01T04:40:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1031
- **Merged:** 2026-09-01T04:40:30Z
- **Merge commit:** 318cd8eae62faacdf088700af088a5412ce19e7b
- **Base / Head:** main / UOR-Foundation:issue-973-group-addressed-retention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and records the bounded `R4GroupAddressedRetentionLMV1` campaign. Exact H4 geometry, matched cyclic/permuted controls, population/reachability, causal gradients, memory, and equal-work ledgers passed. The frozen timing projection and disposable learning smoke did not. The terminal verdict is therefore `UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`.

No main optimization or held-out scoring ran; this PR makes no attention, H4-advantage, generation, inference, or reasoning claim. The exact cell is retired without tuning. #973 now owns selection of a fuller source-free decoder block: token/residual path, RMSNorm, 2-4 stacked retention blocks, SwiGLU/channel mixing, residual connections, and tied output head with matched controls.

## Evidence and provenance

- Canonical record: `docs/r4_group_addressed_retention_973.md`
- Terminal result CID: `blake3:72ed93ec7e1356091cb40bdc6f35d89cf6520f4262778149908817a514832018`
- Shipping trainer tree CID: `blake3:066ac85303e791dcfe024a434e1dd259aeac60fcfac9c022f3bc547b1cfeeab2`
- Raw started/result envelopes are preserved byte-for-byte in `docs/`.
- Held-out material remained mode `000`; no authorization, reveal marker, held-out opener, or main-run command exists.
- The exact executor contract now binds both new Rust source files, preserving complete local source closure.

## Verification

- 28 focused Python tests: PASS
- Focused Rust geometry/exporter tests: PASS
- `uor-r4-model-source` source-closure test: PASS after binding the two new files
- Full affected Rust crate, serial harness: 167 passed, 4 fixture-dependent ignored
- Scheduling-overlap test: PASS isolated; its parallel-harness-only miss is documented rather than hidden
- Workspace clippy, no_std ladder, formatting, claim wording, WASM check, and diff hygiene: PASS (affected crate clippy rerun after the provenance fix)
- Independent final audit: APPROVE

References #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

