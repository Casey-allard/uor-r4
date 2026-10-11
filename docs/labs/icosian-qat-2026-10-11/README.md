# Icosian weights: a post-hoc snap costs +0.141 BPB, and training the snap in does not recover it (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** geometric weights · count 1/3 · headline "softmax at runtime: yes" → yes.
**Pre-registrations:**
- spin-snap probe: [6105073154](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105073154), amended in 6105188937;
- trained-in rung: [6106445338](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6106445338), amended in [6106573972](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6106573972).

**Test fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6106609174).
**Device amendment:** [6110389805](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6110389805).

## Question

[#2193](../quaternion-weights-2026-10-11/README.md) kept quaternion (Hamilton-block) maps: Q256 reaches 1.4959 BPB with 4× fewer map weights. The spin-native design wants each quaternion weight to be one of the **120 unit icosians** times a scale. A block product then becomes a Cayley-table read plus a shift. What does that cost in quality?

## 1. Spin-snap probe: snap a trained model after the fact

All three Q256 seeds were evaluated on the same 256 held-out blocks. T is the float model, at 1.4959 BPB.

| Arm | What is snapped | Δ BPB vs T |
|---|---|---:|
| W | weights → norm × nearest icosian (mean cosine 0.965) | **+0.141** |
| WAs | W + activation scale (2^e) | +0.265 |
| WAd | W + activation direction (icosian) | +0.327 |
| WA | W + full activation snap | +0.475 |

**Row 3: train the snap in.** The weight snap alone is the first bottleneck.

## 2. Trained-in snap: the model learns on the icosians

`quaternion_snap=icosian|icosian_f32` replaces every compact quaternion weight in the forward pass by norm × nearest unit icosian, with a straight-through gradient.
- **QSe** (`icosian`) also rounds the norm to the nearest power of two. A weight-side block product is then a table read and a shift, with no multiplier.
- **QSf** (`icosian_f32`) keeps the f32 norm.
- **Tests:** the snapped forward values, the exact straight-through gradient, logits equal to a float model holding the snapped weights (with a negative gate), and save/load. `data_parallel` refuses snapped training.

**Runs.** #2193's recipe: width 256, `rrarrarr`, 2,000 steps. They trained on one RTX 4090 through `uor-pod`: 7 runs in 13 minutes, about 107 s each, against about 1 h each projected on the laptop CPU. Evaluation used `evaluate max_blocks=320 tune_blocks=64` on the laptop. Under the device amendment, the base **Q256c** is Q256 re-trained on CUDA.

| Arm | BPB per seed | Mean | Δ vs Q256c |
|---|---|---:|---:|
| Q256c, float quaternion (CUDA) | 1.4932, 1.4956 | **1.4944** | — |
| Q256, float quaternion (CPU, #2193) | 1.4940, 1.4942, 1.4994 | 1.4959 | +0.002 |
| **QSe**, icosian + power-of-two norm | 1.6292, 1.6353, 1.6388 | **1.6344** | **+0.140** |
| QSf, icosian + f32 norm | 1.5971, 1.5981 | 1.5976 | +0.103 |
| Q256-W, post-hoc snap (probe) | — | 1.6369 | +0.141 |
| D152, dense at equal parameters (#2193) | 1.5158, 1.5114 | 1.5136 | +0.019 |

Noise is 0.0041: the pooled seed SD, which is larger than the paired per-block SE of 0.0020.

**Decision: row 4**, for the configuration `quaternion_snap=icosian`, width 256, 2,000 steps (D22 §1).
- QSe is 0.0025 better than the post-hoc snap, which is below 2 × noise.
- So the straight-through estimator does not recover the snap cost at this scale.

**What QSf locates:**
- The **direction** snap costs +0.103: the 120 icosians are about 15° apart.
- The **power-of-two norm** adds +0.037.
- The direction set is the part to make finer.

**Trained exponent range.** The stored norms span 2^−9 … 2^−2 in all three QSe seeds: 8 values, so **3 bits**, against the estimated 5. A snapped quaternion is therefore 7 + 3 = 10 bits against 128 bits in f32.

## Next (pre-registered row 4)

Use a finer exact set before any activation snapping. Two candidates:
- **residual icosian pairs** g₁ + 2^−k·g₂: 120² directions, two table reads and a shift;
- the 2,160-point second E8 shell for quaternion pairs.

QSf says the direction is the part that needs it. Each candidate gets a minutes-scale reconstruction check first: the cosine of the snapped weights against the float weights.

## Files

- [`score.py`](score.py) reproduces every number. Runs and evaluations are in cloud-store `claude/icosian-qat-20261011`; the CPU base runs are in the quaternion study's store.
- [`result.json`](result.json) holds the scored output.
