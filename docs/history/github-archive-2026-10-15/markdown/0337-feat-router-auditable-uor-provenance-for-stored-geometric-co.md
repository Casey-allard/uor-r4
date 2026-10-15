# #337 — feat(router): auditable UOR provenance for stored geometric coordinates (#259)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T21:48:20Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T23:33:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/337
- **Merged:** 2026-08-01T23:33:53Z
- **Merge commit:** 2c7712e5dc8bf1c7a0e9af4491dd27e0f1e3936b
- **Base / Head:** main / UOR-Foundation:issue-259-provenance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #259. Companion to #246 (merged via #313): stored geometric coordinates are now auditable — every indexed `CorpusItem` carries a UOR-addressed certificate chain for the text and transforms that produced it, computed through `uor-addr::json` (never a raw blake3 digest).

## What lands

- **`CoordinateProvenance`** on `CorpusItem` with three κ axes, populated at index time in `UorR4Router::index_sentence`:
  - `source_kappa` — canonical sentence-text axis (`uor-r4-router-source-v1`);
  - `projection_kappa` — versioned 16-window sparse-QR config (channels, windows, samples, subwindows, x-range, ρ, sparse radius, query-state blend, zeta-zero set id);
  - `vocabulary_kappa` — sorted word→prime assignments used for that sentence's content vector.
- **`verify_corpus_provenance(identity)`** — recomputes all three axes over the indexed corpus for an identity scope; fails closed with `ProvenanceError::{Missing, Mismatch}` identifying the sentence and axis. Deliberately outside the prediction kernel.
- **Retrieval surface** — `ResonanceResult` gains an optional `provenance` field; both `retrieve_geometric_resonance` and the angular path propagate the stored chain.
- **Compatibility** — provenance is `Option` with `#[serde(default)]`, so pre-#259 exported router states still load; verification rejects those legacy entries as missing evidence (re-indexing required for an auditable store). The content-free measurement arm (#255 harness) also records provenance and is verified, not skipped.
- Docs: `docs/provenance_259.md`.

## DoD mapping (issue #259)

- Provenance fields populated at index time via the uor-addr crate (not raw blake3) — **met**: `uor_json_address` goes through `uor_addr::json::address` (blake3 variant only as an internal fallback inside the crate's API surface).
- A verify pass over an indexed corpus — **met**: `verify_corpus_provenance`, covered by `tests/provenance.rs` (both the content-bearing and content-free arms).
- One retrieval surface exposing the chain — **met**: `ResonanceResult.provenance` on geometric resonance retrieval.

## Gates (worktree `.worktrees/pr-259`, `CARGO_TARGET_DIR=target-wt`, offline)

- `cargo fmt --all -- --check` ✓
- `cargo clippy -p uor-r4-router --all-targets --offline -- -D warnings` ✓
- `cargo test -p uor-r4-router --offline` ✓ (all router test binaries incl. `tests/provenance.rs`: 2/2)
- `python3 scripts/check_claim_wording.py` ✓

Branch is merged up to current `origin/main` (16 commits; clean ort merge, one auto-merged hunk in `crates/uor-r4-router/src/lib.rs` alongside #247's session-signature module).

## Notes for reviewers

- The projection-κ config constants (16 windows, 257 samples, 6 subwindows, etc.) mirror the #246 sparse-QR design; if the projection config is versioned later, `projection_provenance()`'s payload is the single place to bump.
- Scope is the f64 exploratory router (`uor-r4-router`), matching the issue's geometric-revival context; the graph-migration path is untouched.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

