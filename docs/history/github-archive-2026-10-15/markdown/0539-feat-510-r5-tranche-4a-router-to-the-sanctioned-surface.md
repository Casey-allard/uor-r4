# #539 — feat(#510): R5 tranche 4a — router to the sanctioned surface

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T10:01:11Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-09T10:20:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/539
- **Merged:** 2026-08-09T10:20:19Z
- **Merge commit:** 8c3b51b4bd443ce3accb407f5c58eeeda6c83c47
- **Base / Head:** main / UOR-Foundation:r5/router
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of the #510 R5 rearchitecture. Converts `uor-r4-router` to the sanctioned Result<> surface, clearing all 21 audit-limits violations for the crate.

## What changed
- **geometry.rs** — remove `GeometryError`; `SemanticGeometry` is now total: `ground → Option`, `encode → FacetCoordinates`, `soft_route → Vec<WeightedRoute>`, `apply_operator → Option`. Callers in benchmark.rs/lib.rs follow.
- **lib.rs** — `verify_corpus_provenance → Option<usize>` (`ProvenanceError` removed); `import_state`/`import_state_native → bool` (total import); `init_wasm → ()` (`#[wasm_bindgen(start)]` never failed). Server + test callers updated.
- **fallback.rs** — `FallbackRouter::execute` closures `FnMut() -> EngineResponse` (total) instead of `Result<_, String>`; outcome rides in `EngineResponse.status`. Legacy mechanism kept (R4), surface totalized (R5).
- **xtask/src/audit.rs** — teach audit-limits multiline `Result<...>` return types (rustfmt wraps long ones, leaving a bare `-> Result<` the single-line scan mis-flags); carve out the two `uor_foundation` SDK contract error types (`ShapeViolation` from an `axis!` `route_query`, `PipelineFailure` from a `PrismModel::forward` impl) — external trait-fixed surfaces, the serde-carveout analogue.

## Verification
- `cargo build --workspace --all-targets` — clean
- router tests pass; `clippy -D warnings` clean (router/xtask/root); `cargo fmt --check` clean
- `wasm32-unknown-unknown` check clean; `audit-deferral` (R4) passes
- `audit-limits` router violations: **0**

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

