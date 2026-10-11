# Residual icosian pairs: the pair fixes the direction (+0.016 BPB with an f32 norm); the power-of-two norm is the remaining cost (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** geometric weights · count 2/3 · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6111364163](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6111364163), amended in [6111530535](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6111530535).
**Test fitness:** [FIT, round 2](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6111540766). Round 1 was NOT FIT: a test never ran, and no arm separated direction from norm. Both were fixed before compute.
**Owner cap approval:** [6111589309](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6111589309).

## Question

[#2202](../icosian-qat-2026-10-11/README.md): trained-in snapping to the 120 unit icosians costs +0.140 BPB, and the direction is the coarse part. Does a finer exact set fix the direction while staying multiplier-free on the weight side?

## The finer set

`quaternion_snap=icosian_pair` turns each quaternion weight into 2^e (g₁ + g₂/4):
- g₁ is the nearest unit icosian;
- g₂ is the icosian nearest the residual;
- e is the power of two nearest |q| / |g₁ + g₂/4|.

On the weight side a block product is two icosian table entries, a shift and an add. `icosian_pair_f32` keeps the pair's direction with an f32 norm, to separate the two costs. Both use a straight-through gradient.

**Before compute,** on 20,000 trained Q256c quaternions, the pair cut the mean angle to the set from 14.75° to **4.73°**: 9× less direction error energy.

**Tests:**
- the values are exactly 2^e (g₁ + g₂/4);
- the mean cosine is above 0.99 and beats one icosian;
- logits equal a float model holding the snapped weights, for both modes, with a negative gate: dropping the dispatch fails the test.

## Runs

- **Recipe:** #2202's (width 256, `rrarrarr`, 2,000 steps).
- **Training:** 2 × RTX 4090 on `uor-pod`, about 25 min of pod time.
- **Evaluation:** on the laptop, on the same 256 blocks.

**Throughput finding:** the snap runs on the host each step, so training ran at **15.6k tok/s, about 11× slower** than plain quaternion training, with the GPUs about 2% busy. Any further snap rung moves the snap onto the GPU first.

## Result

| Arm | BPB per seed | Mean | vs Q256c |
|---|---|---:|---:|
| Q256c, float quaternion (#2202) | 1.4932, 1.4956 | 1.4944 | — |
| D152, dense at equal parameters (#2193, CPU) | 1.5158, 1.5114 | 1.5136 | +0.019 |
| **QPf**, pair direction + f32 norm | 1.5085, 1.5125 | **1.5105** | **+0.016** |
| **QP**, pair direction + power-of-two norm | 1.5643, 1.5668, 1.5644 | **1.5652** | **+0.071** |
| QSf, one icosian + f32 norm (#2202) | 1.5971, 1.5981 | 1.5976 | +0.103 |
| QSe, one icosian + power-of-two norm (#2202) | 1.6292, 1.6353, 1.6388 | 1.6344 | +0.140 |

**Noise:**
- for QP: 0.0015, the pooled seed SD of QP and Q256c, which is larger than the paired SE;
- for QPf: 0.0028, which includes QPf's own seed SD (the FIT nit, applied before scoring).

D152 was trained on CPU. The CPU–CUDA gap for Q256 is 0.0015, below the noise.

**Decision: row 3.** The pair fixes the direction, and **what remains is the power-of-two norm**:
- QP beats QSe by 0.069, far more than 2 × noise.
- QPf ≤ Q256c + max(0.02, 2 × noise) = 1.5144.
- With an exact pair direction the model is within 0.016 of float quaternion weights and level with dense at equal parameters.
- Rounding the norm to a power of two adds +0.055.

**Bits.** The trained norms span 2^−9 … 2^−2 in all three seeds: 3 bits, plus at most 1 for the pair's norm. A weight is about 17–18 bits, against 128 in f32.

## Next (row 3)

A finer exact norm set that stays shift-and-add: 2^(e/2), via a √2 table entry, or 2^e·{1, 1.5}, via one more shifted add. Two things first, both under the owner's rule for targeted GPU runs:
1. move the pair snap onto the GPU;
2. a minutes-scale check of norm-reconstruction error on the trained weights.

## Files

- [`score.py`](score.py) and [`result.json`](result.json). Runs and evaluations are in cloud-store `claude/icosian-pair-20261011`; bases are in `icosian-qat-20261011` and `quaternion-study-20261011`.
