# ORDER: one-model-end-to-end, deepseek-15 (panic-safe D11 hook restoration)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `7b22e421162d79e7c8e27a45d112f41009ca35ed`.

## Goal

Make the D11 diagnostic/training scope functions restore their thread-local
mode after a panic, just as they do after `Ok` and `Err`. This closes the gap
Claude recorded in claude-265 item 4 without changing a successful forward,
artifact, training result or Claude-owned branch.

## Cancel probe

At the base, `with_d11_export_gates`, `with_d11_snaps`, and
`with_d11_training_snaps` perform restoration after the closure returns but
have no drop guard. A panic therefore leaves the gate hook, snap flags or
training bridge active on that thread. `with_served_surrogate_training` has
its own outer guard; the standalone D11 functions do not.

Claude's claude-266 branch is docs-only at
`57ba5095ce0872bb8818878983b975671e38f1e3`. It changes no file in this unit.

## Files in scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-15/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. Export gates, chosen D11 snaps, and the D11 training bridge are all
   restored by drop guards after a caught panic.
2. Normal `Ok` and `Err` behavior and persistent fields documented by the
   existing functions are unchanged.
3. Tests start from non-default hook state, panic inside each scope, and prove
   every field that the normal scope restores is restored.
4. Focused and full float host tests pass.
5. Exact-head review, PR checks, merge and fresh-main verification pass.

## Stop if

- A panic-safe change alters successful D11 arithmetic or state lifetime.
- The fix requires a GPU run, a model artifact or an edit to Claude's branch.
