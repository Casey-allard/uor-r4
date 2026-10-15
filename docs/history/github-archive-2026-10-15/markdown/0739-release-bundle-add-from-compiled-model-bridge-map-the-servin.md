# #739 — release_bundle: add from_compiled_model bridge; map the serving cascade (#655)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T19:37:34Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-08-16T19:59:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/739
- **Merged:** 2026-08-16T19:59:00Z
- **Merge commit:** f3580343913528f37904d811b490b655cbf984f8
- **Base / Head:** main / UOR-Foundation:issue-655-c1a-bundle-discovery
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #655

## Summary

Scoping + a small grounded code slice toward #655-C1 (shared startup loader), which prior research on this issue found needed dedicated scoping rather than a quick continuation, since the codebase has three independent, disconnected model-loading systems (`ModelStore`, `src/server.rs`'s `#248` cascade, and the #655-C0 `ReleaseBundleManifest` schema).

- **New:** `docs/SERVING_MODEL_DISCOVERY.md` — a code-cited map of the `#248` cascade's four tiers (r4g1, transformerless, teacher-oracle, geometric), their exact discovery/file conventions, `.uor-models/last_engine.txt`'s pin/no-pin semantics, and `ModelStore`'s divergent required-file contract. Concludes `ReleaseBundleManifest` should stay scoped to one bundle/one engine (matching #655-D's own "one minimal bundle" framing) rather than becoming a multi-tier schema. Linked from README's doc map and a new `MODEL_LIFECYCLE.md` "8. Serve over HTTP" section.
- **Code:** `ReleaseBundleManifest::from_compiled_model` — a pure, in-memory constructor bridging `crate::compile::CompiledModel`'s real compile output into this schema, proving the field fit empirically. No filesystem I/O, no bundle-directory discovery, not wired into any server/CLI/client path — additive and dormant per #515's convention.

## Verification

- `cargo fmt --check`
- `cargo clippy -p uor-r4-api --all-targets -- -D warnings`
- `cargo test -p uor-r4-api` (27 tests, including 2 new)
- `cargo run -q -p xtask -- audit-limits` (R5)
- `cargo run -q -p xtask -- check-model` (CM-01)
- `python3 scripts/check_claim_wording.py`

All green.

Files: `crates/uor-r4-api/src/release_bundle.rs`, `docs/SERVING_MODEL_DISCOVERY.md` (new), `docs/MODEL_LIFECYCLE.md`, `README.md`.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

