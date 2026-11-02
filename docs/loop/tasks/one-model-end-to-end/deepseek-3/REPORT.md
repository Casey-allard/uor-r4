# REPORT: one-model-end-to-end, deepseek-3

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `e1de1ff2c`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **This does not yet train through the full served graph.** The new wrapper
  enables the discrete boundaries, but `train-in-space` still runs its existing
  forward. The next unit must wire these primitives into the actual hidden-state
  path and checkpoint selection.
- **CPU Candle computation is unavailable by design.** D24/D29 make the CPU
  tensor backend refuse model compute. Exact scalar forward controls and bridge
  state run on CPU; the tensor autograd control is CUDA-only and was
  `NOT_RUN` on this machine.
- **A concurrent harness build in the shared target directory made an unrelated
  saved-trunk test fail with `owned artifact directory required`.** Rebuilding
  and testing the float crate in an isolated `CARGO_TARGET_DIR` passed; no
  source change was made for this target-directory collision.
- **The activation quantizer is a float shadow.** It reproduces the served
  dynamic i16 scale, clamp and straight-through decision, not integer route
  arithmetic for the entire trunk.

## What changed

- Q16 residual quantization now clamps to the served i32 grid and, under
  `with_served_surrogate_training`, forwards the exact rounded value while
  passing identity gradient inside the range and zero gradient at saturation.
  Its upper mask bound is exclusive because `i32::MAX as f32` rounds up to
  `2^31`; the new host oracle caught and pinned this boundary.
- Added dynamic per-row i16 activation quantization. It selects one
  power-of-two scale from the row maximum, uses the served half-up rounding
  rule, clamps to `[-32767, 32767]`, and applies the same straight-through rule.
- `with_served_surrogate_training` enables the existing D11 snap/Q12 bridges
  together with the Q16 and activation bridges, and restores all thread-local
  flags on success, error or panic.

## Measurements

Finite scalar controls:

- Q16: `1.4/65536 -> 1/65536`, `-2.6/65536 -> -3/65536`,
  `-0.5/65536 -> 0` pinning negative half-up, and `32768` saturates to
  `i32::MAX/65536`.
- Dynamic scale: row maxima `40,000` and `70,000` select `1/2` and `1/4`.
- The full f32 exponent range is covered: maximum `2^63` selects `2^-49`.
- A row `[40000, -12001, 3, -1]` at scale `1/2` quantizes/dequantizes to
  `[40000, -12000, 4, 0]`, pinning half-up and precision loss.
- Saturated scaled values are outside the `[-32767,32767]` gradient mask.
- Host scalar gradient oracles pin identity gradients inside both ranges and
  zero gradients at Q16 and activation saturation. Planted saturation-gradient
  faults make the controls fail, so they are not forward-only assertions.
- Wrapper tests seed every gate-hook field, then verify exact restoration after
  success, error and panic.

The CUDA autograd test pins identity gradients inside the clamp and zero at
Q16 saturation; it is compiled only with `cuda` and was `NOT_RUN` locally.

## Checks run at the head

- `cargo test -p uor-r4-float-train --release
  served_surrogate_scalar_and_state_controls_are_exact -- --nocapture` — passed
  (1 test; it deliberately catches and reports an internal panic while proving
  panic-time state restoration).
- Full float crate library suite on a fresh release target: 207 passed, 0
  failed at
  `CARGO_TARGET_DIR=local/deepseek/served-ste-final-target`; the focused host
  control was rerun after adding panic restoration and the rounding controls.
- Source formatting and diff check passed.
- Float crate-wide strict Clippy still reports six pre-existing findings in
  `fbs_export.rs`, `flock_read_memory.rs`, `flock_read_register.rs` and
  `in_space_long.rs`; none is in this change. The unit records that debt rather
  than rewriting unrelated files.
- Exact-head review and loop checks are recorded after they run.

## Skills used

No repository skill was invoked. The unit reused the existing D11 bridge and
Q12 precedent in `hamilton_sheet`.

## Still open

- Wire the quantizers into the real geometric hidden path and make served
  surrogate mode reject configurations that cannot be exported.
- Resolve Hamming/memory discrete selection and remove or formalize memory
  dropout and reply cuts in this mode.
- Run the bounded four-arm CPU/GPU A/B comparison against current
  `train-as-served` and rank-margin.

## Plain summary

The trainer now has tested building blocks for the two largest missing
quantization boundaries: the residual Q16 grid and the dynamic i16 activation
grid. They preserve exact served forward values while sending controlled
gradients through the float shadow. They are not yet connected to the full
training loop, so the existing `train-as-served` claim remains bounded to
rounded parameters.
