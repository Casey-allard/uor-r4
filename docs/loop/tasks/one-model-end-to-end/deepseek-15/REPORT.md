# REPORT: one-model-end-to-end, deepseek-15

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`7b22e421162d79e7c8e27a45d112f41009ca35ed`.

## What did not work, first

- **The first panic test assumed `with_gate_mode` restored its call index.**
  Its documented normal behavior deliberately leaves the active arm, seed,
  step and reset forward index set, so the test's initial state was corrected
  to that contract. The production behavior was not changed to satisfy the
  test.
- **Strict float Clippy is not green at the base.** Five unrelated lints
  remain in `flock_read_memory.rs`, `flock_read_register.rs` and
  `in_space_long.rs`. The changed `hamilton_sheet.rs` has no diagnostic.
- **The broader objective remains open.** This repair hardens the diagnostic
  lifecycle; it does not run a GPU, settle I-3, improve chat quality or change
  a model artifact.

## What changed

Added a small `ScopeRestore` drop guard and used it in:

- `with_gate_mode`;
- `with_d11_export_gates`;
- `with_d11_snaps`;
- `with_d11_training_snaps`.

The fields each function already restored after normal return now restore
after unwinding as well. Intentional lifetimes are preserved:
`with_gate_mode` still leaves its arm, seed, step and reset forward index set,
while the D11 scopes still restore the fields their docs say they restore.

The code change is **100 added and 29 removed lines**. It includes one
caught-panic test that begins with non-default hook state and checks the full
`GateHookState` after each scope.

## Measurements

No runtime performance is claimed.

- Float host library: **219 passed, 0 failed**.
- The caught-panic test passed for all four scopes.
- The non-denying Clippy run completed with only the five known baseline
  warnings; no warning points into `hamilton_sheet.rs`.
- Exact-head CI completed all required jobs successfully.

## Checks run at the head

- `cargo test --locked -p uor-r4-float-train --lib`: passed;
  [float-lib-tests.txt](evidence/float-lib-tests.txt).
- `cargo clippy --locked -p uor-r4-float-train --all-targets --no-deps`:
  passed with five pre-existing warnings;
  [float-clippy-warn.txt](evidence/float-clippy-warn.txt).
- Strict Clippy was attempted and retained:
  [float-clippy-strict.txt](evidence/float-clippy-strict.txt).
- `rustfmt --check` on the changed file: passed.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied.

## Still open

- Run the served-contract readout against claude-265's sealed checkpoints when
  a GPU is available.
- Add matching float and integer per-layer stage traces to localize the
  remaining I-3 differences.
- Continue the complete-reply/fresh-phrasing and capability work under
  #3407/#3506.

## Plain summary

Several diagnostic and training helpers temporarily change thread-local
rounding modes. They used to restore those modes only when the closure
returned normally. If the closure panicked, the next calculation on that
thread could silently run under the wrong model contract. This change makes
the restoration happen when the scope is dropped, including during a panic.
All 219 host tests pass, and successful training behavior is unchanged.
