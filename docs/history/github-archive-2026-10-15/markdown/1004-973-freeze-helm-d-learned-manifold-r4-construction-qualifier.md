# #1004 — #973: freeze HELM-D learned-manifold R4 construction qualifier

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-29T23:20:34Z
- **Updated:** 2026-10-09T14:17:20Z
- **Closed:** 2026-08-29T23:21:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1004
- **Merged:** 2026-08-29T23:21:50Z
- **Merge commit:** 40d2c94259d34e153923835faf180b51130eac20
- **Base / Head:** main / UOR-Foundation:issue-973-helm-d-learned-manifold-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add the post-projection, pre-RoPE Q/K/V intervention seam and decoder-owned audit
- implement source-faithful full-head HELM-D Lorentz attention over transported R4 blocks
- implement the independently fitted, exactly equal-capacity Euclidean arm and three geometry-destroying controls
- freeze the deterministic eight-shard fitter, causal checkpoint/admission boundary, evidence identities, replay, and terminal logic
- freeze a fresh target-blind 16-fit/8-validation non-D3 partition
- synchronize the living research/programme direction while keeping D3, resonance, recurrence, lowering, E8 expansion, and #954 blocked

## Frozen identities

- partition manifest: `blake3:359b3270aaa0d3ac157280c9206ad820d18ee93932e0530552ebbb7935ac6410`
- partition: `blake3:5c5a7dab9d7a0fbc9d176faafd49b42094ef89138cc32699dfc1b4fe937d1bde`
- tokenizer: `blake3:70af0cb08bbcd3b323d3387ca1d7d33da39873820604d183711e8e99f9903fc1`
- learned capacity: 144,060 scalars per Lorentz/Euclidean arm

Construction validation and D3 remain `NOT_RUN`. The decision run is deliberately deferred until this implementation has a protected merge revision.

## Focused verification

- `cargo test -p uor-r4-model-source --offline pre_rope_projection` — 1 passed
- `cargo test -p uor-r4-core --offline helm_d_learned_manifold_r4_construction` — 6 core plus 5 integration preflight tests passed; only freezer/decision ignored
- focused core, model-source, and integration clippy with `-D warnings` — passed
- `python3 scripts/check_claim_wording.py` — passed
- `cargo fmt --all -- --check` — passed
- wasm library compile — passed
- target-blind partition freezer — passed
- independent P0-P2 contract/implementation review — passed

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

