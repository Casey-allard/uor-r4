# #1493 — Stack export: refuse a snapped-model save on every model-directory export path (steward)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T06:12:04Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-29T12:08:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1493
- **Merged:** 2026-09-29T12:08:26Z
- **Merge commit:** 719cda6e53efb5a82a73953003ef1349e9975a04
- **Base / Head:** main / UOR-Foundation:codex/s4-serving
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Lab 1 serving task 1 (class C): no model-directory export may write a transport-snapped save while the integer engines compute the free transport.

## What changed

- `stack_export::check_export_transport(model_dir)` is now the single refusal. It reads `StackModel::saved_transport_snap` and refuses with the explicit `transport_snap=<name>` error.
- The `geometric-stack` example's `export` calls it (the inline copy is removed; same message).
- `load-candidate-forward-report` (the candidate flow) calls it before loading the model, so a snapped candidate cannot be exported raw.

## Notes

- The example's export path already refused snapped saves (shipped with #1483); the actual gaps were the missing regression test and the second, unguarded model-directory export path.
- `uor-r4-integer`'s `uor-r4-stack` serves an exported artifact (not a model directory) and `stack_aerm` loads a checkpoint for evaluation; neither is an export path, so neither changes.

## Checks (local, executed)

- `cargo fmt -p uor-r4-training -- --check` — clean.
- `cargo test --release -p uor-r4-training --lib stack_export::tests` — 8 passed, 0 failed (0.54 s run; release build reused a warm dependency cache).
- New test `export_refuses_a_snapped_save_and_an_unsnapped_save_still_exports`: a snapped save is refused with the explicit message; the same directory saved without the snap passes and `export_stack` writes an artifact.
- `cargo check --release -p uor-r4-training --example geometric-stack --bin load-candidate-forward-report` — clean.

References #973.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5884967163 — 2026-09-29T06:32:40Z — Casey-allard

**Lab 1 review of `635b7d47`: one required addition, then approved.**

`check_export_transport(model_dir)` covers every path that takes a **directory**, and its test is right. But `export_stack(&model, …)` still accepts an **in-memory** model whose `transport_snap()` is set. A training mode that exports straight from memory, like the dialogue path's export phase, would write the free transport with no error.

**Required:**
- `export_stack` refuses when `model.transport_snap().is_some()`, with the same message.
- Add one test that sets the snap in memory and asserts the error.

With that in place both guards hold, the directory record and the live model. Then deliver by the procedure.

### comment 5889906954 — 2026-09-29T12:05:08Z — Casey-allard

**Addition applied — `f7a5c4f0` (plus the main merge `73db3eb5`).**

- `export_stack` now refuses `model.transport_snap().is_some()` with the same message as the directory guard ("... serve the unsnapped transport, so no export writes this model yet"), placed with the other preconditions before any shape/numerics work.
- New test `export_refuses_an_in_memory_transport_snap`: a live snapped model is refused, the same model with the snap cleared exports.
- Both guards' tests pass together: `cargo test --release -p uor-r4-training --lib stack_export::tests` → **9 passed, 0 failed** (0.21 s), including `export_refuses_a_snapped_save_and_an_unsnapped_save_still_exports` and the new test. `cargo fmt --check` clean; `cargo check --release --bin load-candidate-forward-report` clean after the main merge (which only conflicted in that bin's imports; resolved to main's list plus `check_export_transport`).

Delivering by the procedure now.



---
_Comment/review ids in this file: 5884967163, 5889906954_

