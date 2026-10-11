# Spin-native weights: quaternion and icosian maps

**Milestone:** M4 #2032 (served cost). **Lab:** claude. **Design:** [spin-native model](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104974064).

## The idea

The model's dense projection matrices are 87.6 % of its per-token arithmetic, while its geometry (the quaternion rotation) is 0.06 %. The bridge has two steps:
1. **Quaternion weights.** Every 4×4 block of a matrix becomes one quaternion acting by the Hamilton product the recurrence already uses. That gives 4× fewer weights and 4× fewer bytes read per token.
2. **Icosian weights and activations.** Snap both factors to the 120 unit icosians (SpiralCore's spin double cover 2I, exact over Z[φ]) times a power of two. A block product is then **one read from a 120×120 Cayley table** plus adds: no multiplier, and the spin sign is kept exactly.

## In code

- `quaternion_weights=all`: `crates/uor-r4-training/src/geometric_stack.rs` (`QuaternionWeights`, `quaternion_dense_weight`).
- Post-hoc snap tools: `examples/quaternion-snap.rs` and `evaluate activation_snap=icosian|direction|scale` (`ActivationSnap`). These are probes, not trained in.

## Tried

- **Rung 0, quaternion weights, KEEP** ([record](../labs/quaternion-weights-2026-10-11/README.md)). At width 256, it beats dense at equal total parameters by 0.018 BPB (6× noise, equal weight bytes). Against dense at equal width it uses 4× fewer matrix weights for +0.088 BPB. It does not yet save arithmetic.
- **Rung 1, spin-snap probe:** pre-registered at [6105073154](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105073154) (FIT), running.

## A fair test needs

- Equal total parameters **and** equal weight bytes per token against dense.
- A panel that first shows it separates model sizes.
- At least 2 seeds, with paired per-block noise.
- For the table-product rung: the snap trained in, not only applied post hoc, before any negative counts.

## Open questions

- How much quality does snapping activations cost, and is the cost in the direction or the scale? The probe answers this.
- Snap during training (straight-through) or construct directly in served form (the M2 constructor)?
- The finer group: 2I × {2^(e/2)}, or the 600-cell plus a scale codebook.
