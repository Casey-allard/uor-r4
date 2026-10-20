# Exact activations: a post-hoc snap costs +0.020 BPB on exact-weight models, and training it in does not help (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** geometric weights · count 1/3 (after #2216's KEEP reset it) · headline "softmax at runtime: yes" → yes.
**Reading:** [6114831893](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114831893).
**Pre-registration:** [6115101545](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115101545).
**Test fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115118695), adversarial.

## Question

[#2216](../icosian-mantissa-2026-10-11/README.md) made every quaternion map **weight** exact: m·2^e·(g₁ + g₂/4), at dense quality. If the map **inputs** use the same exact set, every block product is exact-set × exact-set, which can be served by table reads, shifts and adds. What does that cost, and does training it in help?

## Changes

- **`ActivationSnap::IcosianPair` / `IcosianPairF32`.** Each 4-lane of every quaternion map input becomes the weights' exact set (or only its direction), on the device snap, with a straight-through gradient.
  - The snap covers the inputs of the ten quaternion maps; the head and NoRead are excluded.
- **`train activation_snap=`.** It needs `quaternion_weights=all`, refuses data-parallel training, and is saved in the config so evaluation reapplies it.
  - **Test:** changed logits, straight-through gradient, save/load with identical logits. **Negative gate:** not saving the mode fails the round trip.
- **`evaluate device=cuda`** (and `tf32`). The evaluation JSON records the device, `tf32` and the snap actually applied (FIT fix).
- **Snap speed (blocker fix).** The constant tables are cached per device, and each shared input (MLP gate/up, read q/k/v) is snapped once.
  - The first attempt tripped the pre-registered throughput gate at 32.9k tok/s and stopped itself after one run (about $0.08).
  - After the fix, training ran at **42.4k tok/s** (gate 39k).

**Compute:** everything ran on one RTX 4090: training, all 9 evaluations, and the unit tests on the pod's CPU at the exact commit. The laptop did no model or build work (owner direction).

## Result

| Arm (3 seeds, CUDA) | BPB per seed | Mean | vs QM2g |
|---|---|---:|---:|
| QM2g, exact weights, float activations | 1.5139, 1.5149, 1.5155 | 1.5148 | — |
| **QM2g-post**, the same models with activations snapped after training | 1.5335, 1.5343, 1.5352 | **1.5343** | **+0.0196** |
| QA, activation snap trained in | 1.5369, 1.5398, 1.5370 | 1.5379 | +0.0231 |

Noise is 0.0017. QM2g's GPU evaluation matches its earlier CPU evaluation to all four printed decimals.

**Decision: row 3** (QA > QM2g + noise, and QA > QM2g-post − 2×noise).
- Training the activation snap in with a straight-through gradient is slightly *worse* than snapping after training (+0.0036, about 2× noise). The same pattern held for single-icosian weights (#2202).
- This is a configuration result (D22 §1).

**What is usable now: fully exact quaternion maps at +0.021 against dense at equal parameters.**
- Train with exact weights (#2216), then snap activations onto the same set after training.
- Every map weight and every map input is then an element of the small exact set (icosian-pair direction and a 2-bit-mantissa scale): 1.5343 BPB, against dense D152 at 1.5136.

## Next (row 3)

A soft-to-hard schedule for the activation snap (blend α·snap + (1−α)·x with α rising to 1), compared with the post-hoc +0.0196. Separately, the served form: a table kernel for exact-set × exact-set block products.

## Files

[`score.py`](score.py) and [`result.json`](result.json). Evaluations and logs are in cloud-store `claude/activation-snap-20261011`; the models stay on the pod volume (non-canonical).
