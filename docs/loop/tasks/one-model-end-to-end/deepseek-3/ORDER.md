# ORDER: served-quantization straight-through primitives, deepseek-3

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `e1de1ff2c`.

## Goal

Add the first host-completable part of training through the served FBS graph:
canonical Q16 residual and dynamic i16 activation quantization with
straight-through gradients. Provide one scoped training wrapper that enables
these bridges and the existing D11 snap/query bridges together. This unit does
not yet replace the full trainer forward.

The current `--train-as-served` rounds parameter tables before an ordinary
HamiltonActive float forward. That is not a gradient through the served
nonlinear graph.

## Cancel probe

Check whether an existing training wrapper already combines D11 snaps, Q16
residual quantization and dynamic i16 activation quantization with tested
backward behavior. If so, reuse it. Otherwise deepen the existing
`hamilton_sheet` bridges rather than creating a second quantizer.

## Scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-3/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

No serving change and no trainer wiring in this unit.

## Done when

- Q16 residual quantization has exact finite CPU forward values, i32 saturation,
  straight-through gradient inside the range and zero gradient at saturation.
- Dynamic i16 activation quantization chooses one binary exponent per row,
  rounds and clamps like the served form, and supplies a straight-through
  gradient to the unquantized activation inside the clamp.
- A scoped `with_served_surrogate_training` wrapper enables the existing D11
  snap/Q12 bridges plus both new bridges and restores every thread-local flag.
- Focused CPU tests pin forward values and backward gradients, including a
  planted wrong-gradient or clamp fault.
- Formatting and focused tests pass. Record any pre-existing crate-wide Clippy
  debt rather than rewriting unrelated code.
