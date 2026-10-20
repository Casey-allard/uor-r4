# #966 — S0: canonical lexical ingestion and hierarchical route state

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-27T04:00:50Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-08-27T04:02:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/966
- **Merged:** 2026-08-27T04:02:00Z
- **Merge commit:** ae85874a621005ca96f264cece4c24e18ecc225b
- **Base / Head:** main / UOR-Foundation:issue-961-canonical-lexical-ingestion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #961.

## Product

- adds the versioned uor-r4.canonical-lexical-route-manifest/1 envelope and reversible uor-r4.unicode-lexical-runs/1 codec
- keeps the complete codec address registry in the additive parent while embedding the frozen observed-only schema-2 spin manifest unchanged
- serializes exact lexical occurrences, prime/spin/zeta/chart/Hopf/icosian state, shared spin/torsion classes, content blobs, and bounded current through global hierarchy roots
- validates canonical ordering, fixed compiler/child parameters, every vocabulary surface/blob/address, the embedded child bytes and provenance, hierarchy chains, inverse witnesses, and reload/re-encode identity transitively
- exposes terminal and cursor-bound seven-level numeric traces for #952, with explicit absent slots before causal boundaries exist
- adds r4 lexical-ingestion-witness; no model, historical compiler, provider, or generative fallback is loaded

## Authorized fixed witness

Command: cargo run --bin r4 --offline -- lexical-ingestion-witness

Verdict: PASS_REVERSIBLE_STATE_PLUMBING_ONLY

- artifact κ before/after reload: blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd
- canonical byte CID: blake3:55ee35e2301e7c262c6acab2de0c8d007f5ba7d4ad41086a15185e63d9471969 (137222 bytes)
- byte identity, referenced κ identity, and lexical reconstruction: true
- unknown unit: REJECTED_UNKNOWN_UNIT_BEFORE_STATE_MUTATION
- p² semiprime, all roots/transitive references, shared-class 3/3 lookup, both bridge modes, tangent quarter-turn, and icosian inverses: present/exact
- maximum changed hierarchy nodes per event: 2
- every cursor state resolves numeric geometry; final seven levels exactly equal the terminal consumer trace
- candidate rows remain zero with NOT_IMPLEMENTED_S0_STATE_ONLY

This witness establishes reversible route-state plumbing only. It does not establish source-free attention, coherent generation, correctness, or reasoning.

## Focused static checks

- cargo clippy -p uor-r4-core --lib --no-deps --offline -- -D warnings
- cargo check --bin r4 --offline
- named-file rustfmt check
- scripts/check_claim_wording.py
- git diff --check

Per #961, workspace tests, BDD, doctests, no_std, audit, fuzz, Gate C, model evaluation, corpus compilation, and WASM certification were not run. Required GitHub statuses are transport acknowledgements and are not cited as test evidence.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

