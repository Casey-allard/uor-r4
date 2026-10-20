# #733 — format/graph-compiler/graph-cli/#637-2a: opt-in PROV/1 producer propagation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T15:34:53Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-16T15:55:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/733
- **Merged:** 2026-08-16T15:55:42Z
- **Merge commit:** d57a0e13725245b1071da4a58c4a84b602ab3621
- **Base / Head:** main / UOR-Foundation:issue-637-2-prov-propagation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Phase 2a of #637: wires the frozen PROV/1 section (phase 1, #732) into the real R4G1 producer path, additively and opt-in.

- `uor-r4-graph-format`: `parse_digest_hex` parses a canonical `blake3:<hex>` or bare-hex digest string into PROV's 32-byte wire form (core-only, no_std-safe; 6 unit tests).
- `uor-r4-graph-compiler`: `induction::emit_r4g1_with_provenance` builds and appends one PROV/1 section from a caller-supplied `ProvComponents`. `emit_r4g1`/`emit_r4g1_with_tokenizer_cid` now delegate to a shared private inner emitter with `provenance: None`, so their output stays byte-identical — **no pinned κ fixture moves**. New determinism/round-trip test: same components → same bytes; a changed geometry digest changes the artifact bytes/`artifact_cid` but not `head_cid`.
- `uor-r4-graph-cli`: `cover` gains an opt-in `--emit-provenance` toggle (off by default) that binds whichever of `--source-manifest-kappa` (#597) / `--geometry-projection` (#600) / `--attention-operator` (#602) / `--dense-operator` (#704) the caller already passes into a real PROV section on the artifact, not just the JSON report. Malformed digest strings are refused before any output mutation.

## Non-goals (deferred, tracked on #637)

- #601 tokenizer-adapter digest (the typed record lives one layer above this call site).
- A license / evidence-root CLI surface.
- The score / converter / cached-cover producer paths.
- Flipping `cover`'s *default* output to always carry PROV — that changes every future artifact's κ and needs an explicit maintainer-supervised re-pin per AGENTS.md, not bundled into an additive slice.

## Test plan

- [x] `cargo fmt --check` (uor-r4-graph-format, uor-r4-graph-compiler, uor-r4-graph-cli)
- [x] `cargo clippy --all-targets -- -D warnings` (same three crates)
- [x] `cargo test` — 439 tests, 0 failed, across the three crates
- [x] `cargo run -p xtask -- audit-limits` (R5: no unsanctioned error type)
- [x] `cargo build -p uor-r4-graph-format --no-default-features` (no_std build)
- [x] New test: `emit_r4g1_with_provenance_binds_a_prov_section_without_disturbing_the_unbound_path` (round-trip, determinism, CID-change-on-component-change)
- [x] New test: `cover_options_carry_the_emit_provenance_toggle`

References #637

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01MfUSQi4hgpvVFTgwVrQ3U8


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

