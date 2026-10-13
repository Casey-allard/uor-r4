# Spin-native weights: quaternion and icosian maps

**Milestone:** M4 #2032 (served cost). **Lab:** claude. **Design:** [spin-native model](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104974064).

**Current direction:** the dense quaternion contraction/expansion is inactive
under D25, including handwritten CUDA sums. Preserve all results below at their
original scope; none authorizes another dense comparator or teacher. The owner
requested [SpiralCore BFS plus retained ternary FBS projection](spiralcore-fbs-projection.md).
Local Hamilton rotations remain geometric operators; whole-path training is held.

## The historical idea

The model's dense projection matrices are 87.6 % of its per-token arithmetic, while its geometry (the quaternion rotation) is 0.06 %. The bridge has two steps:
1. **Quaternion weights.** Every 4×4 block of a matrix becomes one quaternion acting by the Hamilton product the recurrence already uses. That gives 4× fewer weights and 4× fewer bytes read per token.
2. **Icosian weights and activations.** Snap both factors to the 120 unit icosians (SpiralCore's spin double cover 2I, exact over Z[φ]) times a power of two. A block product is then **one read from a 120×120 Cayley table** plus adds: no multiplier, and the spin sign is kept exactly.

## In code

- `quaternion_weights=all`: `crates/uor-r4-training/src/geometric_stack.rs` (`QuaternionWeights`, `quaternion_dense_weight`).
- Post-hoc snap tools: `examples/quaternion-snap.rs` and `evaluate activation_snap=icosian|direction|scale` (`ActivationSnap`). These are probes, not trained in.

## Tried

- **Rung 0, quaternion weights, KEEP** ([record](../labs/quaternion-weights-2026-10-11/README.md)). At width 256, it beats dense at equal total parameters by 0.018 BPB (6× noise, equal weight bytes). Against dense at equal width it uses 4× fewer matrix weights for +0.088 BPB. It does not yet save arithmetic.
- **Rung 1, icosian snap** ([record](../labs/icosian-qat-2026-10-11/README.md)):
  - Post-hoc, snapping the weights to norm × nearest icosian costs +0.141 BPB; snapping the activations too costs +0.475.
  - Trained in with a straight-through gradient, the cost is still +0.140 with a power-of-two norm and +0.103 with an f32 norm.
  - So the 120 directions, about 15° apart, are too coarse. The norm needs only 3 bits.
  - Row 4 next: a finer exact set (residual icosian pairs, or the second E8 shell).
- **Rung 2, residual icosian pairs** 2^e (g₁ + g₂/4) ([record](../labs/icosian-pair-2026-10-11/README.md)):
  - With an f32 norm, the cost is **+0.016**: the direction is fixed.
  - With a power-of-two norm it is +0.071, so the norm costs +0.055.
  - Next: a finer exact norm set. The host-side snap made training 11× slower, so it moves to the GPU first.
- **Rung 3, KEEP: 2-bit-mantissa norms** ([record](../labs/icosian-mantissa-2026-10-11/README.md)):
  - m·2^e·(g₁ + g₂/4) reaches 1.5148 BPB, level with dense at equal parameters.
  - About 19–20 bits per quaternion, multiplier-free on the weight side.
  - The snap runs on the GPU, bit-identical and 7.8× faster.
  - Next: activation snapping.
- **Rung 4, activations** ([record](../labs/activation-snap-2026-10-11/README.md)):
  - Snapping every map input to the same exact set after training costs +0.0196: fully exact maps at +0.021 against dense.
  - Trained in with a straight-through gradient it is slightly worse (+0.0231).
  - Next: a soft-to-hard schedule.

## A fair test needs

- Equal total parameters **and** equal weight bytes per token against dense.
- A panel that first shows it separates model sizes.
- At least 2 seeds, with paired per-block noise.
- For the table-product rung: the snap trained in, not only applied post hoc, before any negative counts.

## Open questions

- How much quality does snapping activations cost, and is the cost in the direction or the scale? The probe answers this.
- Snap during training (straight-through) or construct directly in served form (the M2 constructor)?
- The finer group: 2I × {2^(e/2)}, or the 600-cell plus a scale codebook.
