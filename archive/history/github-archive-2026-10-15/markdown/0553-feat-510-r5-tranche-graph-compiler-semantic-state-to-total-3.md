# #553 — feat(#510): R5 tranche — graph-compiler semantic state to total (38→33)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T19:13:07Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-09T19:32:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/553
- **Merged:** 2026-08-09T19:32:41Z
- **Merge commit:** 0f271e938a93f534e5565ae27ce1130ee1d3a8ac
- **Base / Head:** main / UOR-Foundation:r5/gc-semstate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `semantic_state.rs` off its `Result<_, SemanticStateError>` surface to total forms. **The module is now fully total (0 sanctioned-violation sites).** The `SemanticStateError` enum and its `Display`/`Error` impls are removed (`use std::fmt` stays — the manual `fmt::Debug` impl for `Action` needs it).

## Surfaces — all five fallible fns become total `Option`, a uniform surface
- `distance_to -> Option<f32>` / `hamming_distance -> Option<u32>` — `None` when the two states have different vector/signature dimensions; no distance is a product of incomparable states.
- `TransitionEvaluator::apply -> Option<SemanticState>` — `None` when the transition produces no next state (precondition fail, action/state dimension mismatch, forbidden-state constraint, or postcondition fail). Four distinct meanings, none a single sanctioned bound type, so they unify as `None`.
- `Trajectory::step -> Option<&SemanticState>` — `None` on step-limit reached or an `apply` that produced nothing.
- `Trajectory::replay -> Option<Self>` — `None` if any step cannot advance.

None of the five have production callers (the only external references are in `tests/bdd.rs`; the `oracle.step`/`runtime.step`/`patch.apply` matches elsewhere are unrelated types); only tests and BDD call them.

## Ripple (same PR)
`tests/bdd.rs` — the `state_eval_res` World field becomes `Option<Option<SemanticState>>`; `trajectory_error` is dropped for `trajectory_step_rejected: bool`; the transition/step assertions use `.is_some()`/`.is_none()`/bool. The import drops `SemanticStateError`.

## Verification
audit-limits graph-compiler **38 → 33** (semantic_state.rs 0 sites). This clears the last validator — the remaining 33 are all host-I/O (`observation.rs`, `observation_text.rs`) and the CLI entry (`lib.rs`). 86 lib tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

