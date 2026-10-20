# Exact multiplier-free weights at dense quality: residual icosian pairs with a 2-bit-mantissa norm (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** geometric weights · **KEEP, count resets** (D22 §2) · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6114450465](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114450465).
**Test fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114483483), an adversarial review of completion and decision value, with its corrections applied before compute.

## Question

[#2206](../icosian-pair-2026-10-11/README.md) showed that residual icosian pairs fix the direction: +0.016 BPB with an f32 norm. Rounding the norm to a power of two added +0.055. Can a short exact norm set remove that cost while keeping the weight side multiplier-free?

## Before compute: a minutes-scale check

On 20,000 trained quaternions from the f32-norm pair model, the RMS relative error of the norm under each set:

| Norm set | Error |
|---|---:|
| 2^e | 0.203 |
| 2^e·{1, 1.5} | 0.106 |
| 2^e·{1, 1.25, 1.5, 1.75} | 0.053 |
| 3 mantissa bits | 0.027 |

## Changes

**1. The snap runs on the tensor's own device.**
- It uses a matmul against the 120 icosians, argmax, `index_select`, and an exact power-of-two table. The host loop it replaces made #2206 train 11× slower, with the GPU at 2%.
- It is **bit-identical** to the host loops on 20,000 random quaternions in all four earlier modes; those loops remain as the test reference. **Negative gate:** a wrong exponent rounding fails the test.
- **Measured throughput:** 122,000 tok/s on a 4090, against 15,600 with the host snap, **7.8× faster**. That passes the pre-registered gate of at least 39,000.

**2. `quaternion_snap=icosian_pair_m1|icosian_pair_m2`.**
- Each weight becomes m·2^e·(g₁ + g₂/4): two icosian table entries, with the scale an exact short float.
- The 1- or 2-bit mantissa m is chosen nearest in log.
- On the weight side, a product is table reads, shifts and at most two adds.
- **Tests:** same direction as the pair; norm error within half a mantissa step; logits pinned for both modes. **Negative gate:** a wrong mantissa grid fails the test.

## Runs

- **Training:** #2202/#2206's recipe (width 256, `rrarrarr`, 2,000 steps) on one 4090, with `tf32=false`. All 6 runs took 8.5 min, about $0.15.
- **Evaluation:** on the laptop, which re-snaps exactly on CPU, on the same 256 blocks.

## Result

| Arm | BPB per seed | Mean | vs Q256c |
|---|---|---:|---:|
| Q256c, float quaternion weights | 1.4932, 1.4956 | 1.4944 | — |
| QPf, pair + f32 norm (#2206) | 1.5085, 1.5125 | 1.5105 | +0.016 |
| **QM2, pair + 2-bit mantissa** | 1.5139, 1.5149, 1.5155 | **1.5148** | **+0.020** |
| D152, dense at equal parameters | 1.5158, 1.5114 | 1.5136 | +0.019 |
| QM1, pair + 1-bit mantissa | 1.5301, 1.5343, 1.5292 | 1.5312 | +0.037 |
| QP, pair + power of two (#2206) | 1.5643, 1.5668, 1.5644 | 1.5652 | +0.071 |

Noise is 0.0012 for QM2 and 0.0027 for QM1: the largest of the pooled seed SD, the paired per-block SE and the arm's own seed SD.

**Decision: QM2 is row 3, KEEP for the norm.**
- QM2 ≤ QPf + max(0.01, 2×noise): **the norm cost is gone**. It falls from +0.055 to +0.004 over the f32 norm.
- QM2 is level with dense at equal parameters (D152: +0.0012, about one noise).
- QM1 is row 4: one mantissa bit recovers about 70% of the norm cost.

**What is now established.** Every quaternion weight in the maps can be an exact element of a small multiplier-free set:
- 2 icosian indices (7 + 7 bits), 2 mantissa bits, and 3–4 exponent bits over the observed range of 8–9 exponents;
- **about 19–20 bits against 128 for f32**;
- quality equal to a dense model with the same number of parameters.

The remaining +0.016–0.020 against float quaternion weights is the pair's direction error. Activations are still float: the weight side is multiplier-free, and the activation side is the next rung.

**CUDA training versus CPU evaluation** (FIT correction): CUDA snaps can differ from CPU snaps at exact boundaries. The evaluation re-snaps on CPU, so any such flips are a small train/eval mismatch, far below the noise.

## Next

Activation snapping, trained in: the weight side is now exact. The direction cost (+0.016) can be revisited with k per weight or a third icosian term, if the activation rung needs the margin.

## Files

[`score.py`](score.py) and [`result.json`](result.json). Runs and evaluations are in cloud-store `claude/icosian-mantissa-20261011`; the bases are in `icosian-qat-20261011`, `icosian-pair-20261011` and `quaternion-study-20261011`.
