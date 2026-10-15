# #547 — feat(#510): R5 tranche — graph-compiler emit_r4g1 to total (50→49)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T15:42:20Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-09T16:01:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/547
- **Merged:** 2026-08-09T16:01:06Z
- **Merge commit:** a32a56417549d2d0aa5ad34a05fc9f9b9c7a13dd
- **Base / Head:** main / UOR-Foundation:r5/gc-emit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert the R4G1 emitter (`induction::emit_r4g1`) off its `Result<_, String>` surface to the sanctioned vocabulary. **induction.rs is now fully total (0 sanctioned-violation sites).**

## Surface
`Result<(Vec<u8>, CoverArtifactInfo), ObservedBound>`

- **i32 wire bound** (root-prior + region-emission tokens/counts) → `ObservedBound { observed, bound }`. A token/count past the i32 R4G1 wire encoding is a property of the caller's chosen corpus/vocab, not a limitation of the emitter. The two loops that previously used `map_err(String)?` and `.unwrap()` are now consistent — both report the same observed bound.
- **Fail-closed re-validation** (`ArtifactBuilder::build`, `GraphView::parse`, `verify_cids`) runs on bytes the compiler *just serialized from its own cover*. A failure there is a defect in this emitter or the format layer, never a property of caller input, so it is an infallible-construction invariant that **panics** rather than a reported condition. Still fails closed — never returns an artifact the validator or CIDs reject.

The two host-facing callers (`graph-compiler compile()`, `graph-cli cover_command()`) map the bound into their still-`String` surfaces, which convert in a later host-I/O tranche.

## Verification
audit-limits graph-compiler **50 → 49** (induction.rs 0 sites). 86 lib + 24 induction integration tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

