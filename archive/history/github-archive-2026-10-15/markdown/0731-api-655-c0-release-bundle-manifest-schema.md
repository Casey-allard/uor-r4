# #731 — api/#655-C0: release-bundle manifest schema

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T13:46:21Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-08-16T14:04:38Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/731
- **Merged:** 2026-08-16T14:04:38Z
- **Merge commit:** 4f6597869d5a77927702576450493b6ae7970388
- **Base / Head:** main / UOR-Foundation:issue-655-c0-bundle-manifest
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## References #655

**Not** Closes — this is one focused slice of the A–F decomposition on #655.

### Gap audit (posted on #655)

Re-checked the decomposition against current main before proposing this:

- **A (matrix-operation census + CI guard)** — already done (#699).
- **B (uor-matmul production arithmetic ownership)** — already done and taken further than originally scoped (#701/#703, plus #704/#727/#730 for the GPT-2 certified-exact path). Zero `cblas_*` call sites remain anywhere in the tree.
- **C (release-bundle manifest + shared startup loader)**, **D (packaged bundle)**, **E (Production/Experimental profiles)** — unstarted before this PR.
- **F (default flip)** — blocked on explicit Casey sign-off, not touched here.

### Scope

Adds `uor_r4_api::release_bundle::ReleaseBundleManifest` — schema + structural validation only:

- Schema version (`RELEASE_BUNDLE_MANIFEST_SCHEMA = 1`), public model id, `BundleCapability` (continuation / instruction-chat).
- `BundleAbi` — owned/serializable copy of the ABI/contract version surface (`AbiVersion`/`ContractVersion` carry a `&'static str` field and are intentionally not `Serialize`/`Deserialize`).
- `UorMatmulProvenance` — pinned rev, operation/codec profile, license, optional source digest (#655-B).
- `BundleComponentDigests` — blake3 digests of graph/signature-artifact/tokenizer/score-report/compile-report, field-for-field mirroring `compile::ComponentDigests`.
- `TokenizerAdapter` reused directly (already `Serialize`/`Deserialize` in `uor-r4-core`).
- `ReleaseBundleManifest::validate(&self) -> Option<String>` — first-violation structural check (schema support, non-empty identity, well-formed digests/rev, instruction-chat requires a tokenizer family). Returns `Option<String>`, not `Result<_, E>`, mirroring `engine::validate_quality_report` in the same crate — so this shipped function names no custom error type, satisfying R5 (`cargo run -q -p xtask -- audit-limits` passes).

### Non-goals

- No discovery/loading wiring into `src/server.rs` / `src/chat.rs` (#655-C1, follow-up).
- No bundle production (#655-D).
- No Production/Experimental engine-profile split or `.uor-models/last_engine.txt` migration (#655-E).
- No default-model change (#655-F, requires explicit Casey approval).
- Zero behavior change anywhere in the tree; the only existing file touched is `crates/uor-r4-api/src/lib.rs` (module declaration + re-exports).

### Compatibility / era implications

None. This is a new, unread, unwritten type — no artifact, CID, κ, or era is affected.

### Tests and gates run locally

- `cargo test --workspace --offline` — pass, 0 failed (full suite, including the new module's 11 unit tests: round-trip, `deny_unknown_fields`, and one negative case per validated field).
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- `cargo check -p uor-r4-graph-format --no-default-features` / `--features alloc` — clean.
- `python3 scripts/check_claim_wording.py` — pass.
- `cargo run -q -p xtask -- audit-limits` (R5) — pass.

### Known deferrals

- `#655-C1`: shared startup loader that discovers/validates/loads a manifest + bundle, wired into server/CLI/client/WS/WASM.
- `#655-D`: produce and package an actual pinned instruction-chat bundle.
- `UorMatmulProvenance::source_digest` is `Option` and unpopulated until a future step wires a content-digest computation over the pinned dependency source.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

