# #552 — feat(#510): R5 tranche — graph-compiler semantic-emission decoupling to total (41→38)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T18:37:35Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-08-09T18:56:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/552
- **Merged:** 2026-08-09T18:56:56Z
- **Merge commit:** f8174e59c2173fd304bcafd030ff5cbc060926c6
- **Base / Head:** main / UOR-Foundation:r5/gc-emission
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `semantic_emission_decoupling.rs` off its `Result<_, SemanticEmissionError>` surface to total forms. **The module is now fully total (0 sanctioned-violation sites).** The `SemanticEmissionError` enum, its `Display`/`Error` impls, and `use std::fmt` are removed.

## Surfaces
- `SemanticReasoningEngine::execute_pure_reasoning(...) -> Option<SemanticStateTrace>` — `None` when the transitions do not form a coherent trace: a non-sequential source, or a state whose pre-emission status is Contradictory, Exhausted, Novel, or Uncertain below the confidence threshold. Reporting the rejection before any token emission is the whole point of the decoupling; a non-coherent trace is not a product. All five error variants collapse to `None`.
- `SemanticReasoningEngine::trace_from_plan(...) -> SemanticStateTrace` (was `Result`) — the body never returned `Err` (every planner trajectory maps to a coherent trace), so the `Result` was spurious. Now infallible.
- `LanguageEmissionAdapter::emit_language(...) -> Option<LanguageEmissionResult>` — `None` when the trace is not Coherent. Language must not be emitted for a non-coherent trace (the decoupling exists so a non-coherent internal state cannot be masked by fabricated text). The never-constructed `NonSequentialTransition`/`EmissionAdapterFailed` diagnostics go with the enum.

None of the three fns have production callers (the proof-model reference is a string; `trace_from_plan` has no callers at all); only tests and BDD call them.

## Ripple (same PR)
`tests/bdd.rs` — the `decouple_error` World field is dropped for `decouple_rejected: bool`; the execute step matches `Some`/`None`; the contradictory assertion checks it. The import drops `SemanticEmissionError`.

## Verification
audit-limits graph-compiler **41 → 38** (semantic_emission_decoupling.rs 0 sites). 86 lib tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

