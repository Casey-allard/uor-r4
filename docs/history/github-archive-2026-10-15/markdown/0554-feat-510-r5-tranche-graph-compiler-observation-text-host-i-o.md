# #554 — feat(#510): R5 tranche — graph-compiler observation_text host-I/O to SourceUnavailable (33→23)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T19:53:17Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-09T20:12:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/554
- **Merged:** 2026-08-09T20:12:37Z
- **Merge commit:** 541808996d09512f6b7f84a170ee8871bc17ba08
- **Base / Head:** main / UOR-Foundation:r5/gc-obstext
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): first host-I/O module. Convert `observation_text.rs` off its `Result<_, String>` surface to `Result<_, SourceUnavailable>` — the sanctioned host-ingestion boundary (`uor-r4-model-source`). **The module is now fully total (0 sanctioned-violation sites).**

All ten fallible fns are host file I/O — reading the sealed text corpus and checkpoints, writing observation shards/stories/checkpoints — so every failure is "a declared external source/sink could not be ingested/persisted", which is exactly what `SourceUnavailable` names.

## Surfaces
- `StoryIndex::load`, `decode`, `input_kappa`, `read_checkpoint`, `write_checkpoint`, `reconcile_shard`, `reconcile_stories`, `append_story`, `build_report`, `observe_text_corpus`: return type `String` → `SourceUnavailable`.
- `io::Error` / `serde_json::Error` producers propagate via `?` (`SourceUnavailable` has `From<io::Error>` and `From<serde_json::Error>`); path-labeled sites use `SourceUnavailable::new(format!("{path}: {e}"))` to keep the operator diagnostic.
- `decode`'s malformed-checkpoint reasons are preserved as `SourceUnavailable` messages (bad source bytes); `read_checkpoint` propagates them directly.
- The u32 story-ordinal overflow maps into `SourceUnavailable` (a ~4e9-article corpus can't be ingested) rather than a separate bound type.

## WASM
`SourceUnavailable` is itself `#[cfg(not(target_arch="wasm32"))]`. This module is pure host I/O (`std::fs` throughout) reachable only via the graph-cli `observe-text` command (non-wasm) — nothing wasm-side references it (only a `stage_dag` string label). So the whole module is now gated `#[cfg(not(target_arch="wasm32"))]` at its lib.rs declaration; the wasm build drops it cleanly.

## Ripple
The two graph-cli callers (`observe_text_command`, the stories-index loader) map `SourceUnavailable` → `String` at their still-`String` boundary.

## Verification
audit-limits graph-compiler **33 → 23** (observation_text.rs 0 sites). Full workspace build + wasm32 graph-compiler + clippy + fmt clean; 86 lib + 9 observe integration tests, BDD 100/100, proof-model 24 tests, audit-deferral clean.

Part of the #510 bottom-up sweep — the final graph-compiler cluster (host-I/O).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

