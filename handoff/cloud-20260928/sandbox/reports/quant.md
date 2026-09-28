# Quantization experimentalist report: "TurboQuant, but keep the radius, in 4-D"

Date 2026-09-25. Role: quantization experimentalist on the review team. The repo was treated as
read-only. Code, raw JSON and logs are in `scratchpad/exp/quant/`. Labels: `[SOURCE file:line]`,
`[MEASURED]` (script named), `[LITERATURE url]` (retrieved this session), `[DERIVED]`, `[HYPOTHESIS]`.

## 1. Executive verdict

1. **The premise is wrong.** TurboQuant, PolarQuant and QJL all keep the radius.
   - TurboQuant stores ‖x‖ as a float and rescales.
   - PolarQuant keeps one FP16 radius per 16 coordinates.
   - QJL stores ‖k‖ with every key.

   What really distinguishes the owner's idea is the use of 4-D blocks rather than scalars or 2-D
   pairs, and a polytope code on S³ for the block's direction. [LITERATURE]
2. **Novelty is low.** The idea is classical shape–gain VQ (Sabin & Gray 1984), and it was published
   for KV caches in 2025–26:
   - PolarQuant (Wu et al.): 2-D pairs with a quantized radius, and q·k done by table lookup.
   - HQMQ: 4-D quaternion chunks, a separate radius, and a direction codebook built from the 24-cell
     (Hurwitz group 2T).
   - FibQuant and Block-Sphere Quantization: radial–angular block codes after TurboQuant's rotation.

   [LITERATURE]
3. **At equal total bits it beats TurboQuant-MSE only by the generic "vector beats scalar" margin, and
   lattices beat it.** Setup: i.i.d. Gaussian, d=64, the 8-bit norm counted for everyone.
   - H4 gain–shape gains +0.19 / +0.30 / +0.24 / +0.64 / +0.93 dB at 1.5 / 2 / 2.5 / 3 / 4 bits/dim.
   - The D4 lattice (Voronoi cell = 24-cell) gains +0.4–0.7 dB.
   - Unconstrained 4-D k-means gains +0.6–1.0 dB.
   - E8 gains +0.85–1.22 dB.
   - The same ranking holds in all five test distributions. [MEASURED]
4. **The H4 structure is not the source of the gain.**
   - Spherical-k-means codes of the same sizes match the 2I/600-cell codes (0.1288 vs 0.1274 at 2 bits).
   - Random codes are 0.8–1.3 dB worse.
   - Coding radius and direction as separate factors costs 0.3–0.5 dB against an unconstrained 4-D
     code, and up to 0.9 dB against E8. [MEASURED][DERIVED]
5. **"Preserve the radius" helps only in narrow senses.**
   - Inside a 4-D block the radius should be kept. The MSE-optimal split always gives it at least 1
     bit (about 15–20% of the block's bits), and dropping it costs 2.1–4.4 dB at 2.4–3 b/dim.
   - It is not "radius first": at ~2 b/dim a finer direction code without radius beats a coarse one
     with 8 radius levels by 0.7–1.8 dB.
   - Rescaling reconstructions to the stored norm costs no bits and raises TurboQuant's recall@10 by
     +0.02–0.03 on Gaussian data and +0.06–0.14 on data with a common offset. The gauss and aniso
     results were rechecked with 3 seeds × 1000 queries and sit 6–30 paired SE from zero.
   - TurboQuant can adopt this trivially. [MEASURED]
6. **The data distribution matters more than the codebook.**
   - Heavy-tailed coordinates need a rotation (0.9–3.7 dB).
   - Heavy-tailed norms need a per-vector norm (1.9–3.8 dB).
   - Outlier channels or a common mean need centering: TurboQuant's error falls 0.127→0.059, and
     serving recall rises from 0.08–0.20 to 0.50–0.59. [MEASURED]
7. **The serving argument is real but not unique to 2I.**
   - 2I×2I codeword inner products take exactly 9 values in ½ℤ[φ]. Integer ℤ[φ] accumulation matched
     the float path (recall 0.483 vs 0.482).
   - Quantizing the query with the same ~2-bit code costs ≈0.12 recall@10, equally for scalar,
     k-means and E8 codes.
   - Keep queries at full precision (per-query lookup tables), or use a finer query code. [MEASURED]
8. **Best project use: the event-memory read, plus E8-coded weights.**
   - Keys and values as 4-D or E8 codes, scored by table lookup, would replace the 64 software
     products per key in today's integer read [SOURCE crates/uor-r4-integer/src/model.rs:324-331].
   - E8 codewords up to 2.2 bits/dim are exactly signed4 integers, so weights can reuse the existing
     kernel.
   - Do not quantize the recurrent state per step. [DERIVED][HYPOTHESIS]
9. **Verdict: a sound component with about 1 dB of headroom over scalar quantization. It is not a
   breakthrough path on its own.**

## 2. Facts: what the Google methods do with the radius

Sources retrieved this session:
- TurboQuant [LITERATURE https://arxiv.org/abs/2504.19874]
- PolarQuant [LITERATURE https://arxiv.org/abs/2502.02617]
- QJL [LITERATURE https://arxiv.org/abs/2406.03482]
- PolarQuant (Wu) [LITERATURE https://arxiv.org/abs/2502.00527]
- Google blog [LITERATURE https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression/]

| Method | Quantized object | Radius / norm | Inner-product bias |
|---|---|---|---|
| **TurboQuant** (arXiv v1 28 Apr 2025; ICLR 2026) | Haar rotation, then per-coordinate Lloyd–Max for the Beta marginal. D_mse ≈ 0.36 / 0.117 / 0.03 / 0.009 at b = 1..4. KV runs split out outlier channels (32 ch at 3 b + 96 at 2 b = 2.5 b) | Theory uses ‖x‖=1; otherwise "compute and store the L2 norms in floating-point precision and rescale" | MSE variant shrinks (2/π factor at 1 bit). The prod variant, (b−1)-bit MSE + 1-bit QJL on the residual + ‖r‖₂, is unbiased |
| **PolarQuant** (Han et al.; AISTATS 2026) | Recursive polar transform over 16-dim blocks: 4-bit level-1 angles, 2-bit higher angles | **FP16 radius per 16 coordinates** (62 bits / 16). Pair radii are re-encoded by the next level's angles, not discarded | — |
| **QJL** (AAAI 2025) | sign(Sk) | **Stores ‖k‖₂ per key**; estimator √(π/2)/m·‖k‖·⟨Sq, sign(Sk)⟩ | Unbiased |
| **PolarQuant** (Wu et al., NeurIPS 2025) | Each RoPE pair → (ρ, θ), r + t bits | **ρ kept, quantized**; q·k "a table lookup" | — |

The blog calls TurboQuant's first stage "the PolarQuant method" (the radius-and-angle pair
picture), which is probably where the "2 dimensions" idea came from. Four charitable readings of
"ablating the radial direction" each fail [DERIVED]:
- (a) MSE-optimal codes shrink norms, E‖x̂‖² = (1−D)‖x‖². That is a bias, and the prod variant removes it.
- (b) The analysis runs on the unit sphere. The radius is factored out, not dropped.
- (c) PolarQuant does not store pair radii directly, but it keeps the block radius in FP16.
- (d) QJL keeps only signs, but it stores ‖k‖.

**The repo's router paper is inaccurate.** In `research/ai-research/ai-router/router-research/docs/research/ANGULAR_MANIFOLD_ROUTING_PAPER.md`:
- It cites a non-existent title, "TurboQuant: Extreme KV-Cache Compression via Angular
  Rotation-Invariant Quantization" [SOURCE :296].
- It says TurboQuant "demonstrated that L2-normalized embeddings ... are angularly non-uniform"
  [SOURCE :7, :22]. The paper is data-oblivious and makes no such claim; a text search confirms this.
- It says "Our work predates the TurboQuant public release" [SOURCE :46]. arXiv v1 is dated
  April 2025.
- It states its own routing is "purely angular — the radial component does not contribute"
  [SOURCE :154]. That is the opposite of the radius-preserving framing.

## 3. Experimental setup (exact)

- **Code:**
  - `quantlib.py`: codebooks, Lloyd, lattices.
  - `qcodecs.py`: quantizers, data, metrics.
  - `sweep.py`: selection and evaluation.
  - `serving.py`, `radial_test.py`, `renorm_check.py`, `tables.py`, `summary_table.py`.
- **Raw outputs:**
  - `results_<ds>_{A,B,C,D}.json` and `run_*.out`.
  - `serving_<ds>[_c].*`, `radial_<ds>.*`, `renorm_<ds>.out`.
  - The retrieved paper texts are `turboquant_2504.19874.txt` and `polarquant_2502.02617.txt`.
- **Compute:**
  - numpy, one thread, at most two processes at once.
  - Runs took 1–8 min each. The one-off 4-D k-means codebook build took 9.6 min.
  - Total ≈1.5–2 core-hours.
- **Dimension:** d = 64, the project's read width [SOURCE crates/uor-r4-integer/src/config.rs:36-39].
  - Block sizes: 1 (scalar), 2 (polar), 4 (gain–shape, D4, 4-D k-means), 8 (E8), 16 (PolarQuant).
- **Data (synthetic; the project's real vectors were not available offline):**
  - `gauss`: N(0,I).
  - `t3`: i.i.d. Student-t, ν=3.
  - `mvt3`: multivariate t, ν=3 (heavy-tailed radius).
  - `outlier`: channels {3,17,40,58} are ±6 + 3·N(0,1), mimicking KV-key outlier channels.
  - `aniso`: mean with ‖μ‖² = d/2, plus a 1/i spectrum in a random basis (an embedding "cone").
- **Splits (fixed seeds):**
  - 8k training vectors (calibrated parts).
  - 2k validation vectors (configuration selection).
  - 10k database + 100 queries (all reported metrics).
  - Data-oblivious codebooks (TurboQuant-style) are designed on an isotropic Gaussian model.
- **Bit accounting (fixed rate, everything counted):** bits/vector = side + ⌈Σ log₂|alphabet|⌉.
  - Vector norm: 8 bits (0.125 b/dim). TurboQuant's paper uses a float, so this favours TurboQuant.
  - Power-of-two scale: 6 bits. Free scale: 8 bits.
  - PolarQuant block radius: 4, 6 or 8 bits.
  - TurboQuant-prod: 64 sign bits plus an 8-bit residual norm.
- **Selection:** at 1.5 / 2 / 2.5 / 3 / 4 bits/dim, the best configuration, or the best two-config
  time-share across block positions, by validation relMSE, with total bits ≤ target·64.
- **Rotations:** Haar (`qr`) or randomized Hadamard (`rht`, H·diag(±1)/8), which uses only add/sub
  and a shift.
- **Lattice grids:** the Gaussian runs used 20 ball sizes per lattice and a finer scale search. The
  other datasets used 16–18 ball sizes and a coarser scale grid, to fit the time budget. For
  no-rotation Gaussian lattices only the rotated variants were kept, since the data are isotropic.
- **Families:**
  - `TQ`: TurboQuant-MSE. `TQprod`: TurboQuant-prod.
  - `SC-pow2`: the project's signed grid with a per-row power-of-two scale
    [SOURCE crates/uor-r4-training/src/joint_quantization.rs:24]. `SC-opt`: the same grid with a free
    scale. `SCchan`: per-channel Lloyd, no rotation, no side bits (KIVI-like).
  - `P2`: 2-D polar gain–shape. `PQ`: recursive PolarQuant.
  - `GS4`: the owner's idea, as a Sabin–Gray 4-D gain–shape code.
    - The shape is argmax⟨y,c⟩; the gain is a Lloyd-quantized projection.
    - Shape codes: 2T (24-cell), 2I (600-cell, 120), 120-cell (600), rectified 600-cell (720), face
      centroids (1200), and geodesic subdivisions of the 600-cell's cells (840 / 2760 / 6480 / 12600).
    - `-vec` means with the vector norm; `-abs` means absolute block gains and no norm.
    - Controls: `GS4rnd` (random codes of equal sizes) and `GS4skm` (spherical k-means codes).
  - `VQ4km`: unconstrained 4-D k-means (16–4096 codewords).
  - `D4`, `E8`: lattice ball codebooks with Conway–Sloane decoding and a tuned scale.
  - `-c`: centered on a global mean (no per-vector bits).
- **Metrics:**
  - relMSE = mean ‖x−x̂‖²/‖x‖².
  - IP error = mean|Δ⟨q,x⟩| / mean|⟨q,x⟩|; slope = Σ⟨q,x̂⟩⟨q,x⟩/Σ⟨q,x⟩².
  - recall@10 on the MIPS top-10. The SE is ≈0.02–0.03 at 100 queries [DERIVED], so differences below
    0.04 are noise.
  - Serving also reports softmax total-variation over 256 keys (temperature set to mean max-weight 0.3).
- **Implementation checks [MEASURED]:**
  - Gaussian Lloyd–Max: 0.3635 / 0.1177 / 0.0345 / 0.0095 (textbook values).
  - TurboQuant-MSE at d=64: 0.358 / 0.115 / 0.033 / 0.0091 (paper: 0.36 / 0.117 / 0.03 / 0.009).
  - 2I is a group with exactly 9 Gram values.
  - Geodesic code sizes match 120 + 720(f−1) + 600(f−1)(f−2) + 100(f−1)(f−2)(f−3).
  - E8 ball counts match 1 + 240Σσ₃.
  - Small-ball lattice decoding agrees 100% with brute force.

## 4. Results

### 4.1 i.i.d. Gaussian, d=64 [MEASURED `sweep.py gauss`]

Each entry is relMSE, with actual bits/dim in brackets, and the dB change against TurboQuant-MSE
(positive is better).

| Family | 1.5 | 2.0 | 2.5 | 3.0 | 4.0 |
|---|---|---|---|---|---|
| TurboQuant-MSE (TQ-qr, +8-bit norm) | 0.2479 [1.50] 0 | 0.1367 [2.00] 0 | 0.0734 [2.50] 0 | 0.0393 [3.00] 0 | 0.0108 [4.00] 0 |
| TurboQuant-prod | – | 0.5455 [2.25] −8.7 | – | 0.2827 [2.84] −8.6 | 0.0850 [3.84] −8.9 |
| 2-D polar gain–shape (P2-rht) | 0.2580 −0.17 | 0.1382 −0.05 | 0.0761 −0.15 | 0.0397 −0.04 | 0.0107 +0.06 |
| PolarQuant, recursive (PQ-rht) | – | 0.1870 −1.36 | 0.0842 −0.60 | 0.0418 −0.26 | 0.0111 −0.12 |
| **4-D gain–shape, H4 codes (GS4-rht-vec)** | **0.2373 +0.19** | **0.1274 +0.30** | **0.0695 +0.24** | **0.0340 +0.64** | **0.0087 +0.93** |
| 4-D gain–shape, spherical-k-means codes | 0.2370 +0.20 | 0.1288 +0.26 | 0.0702 +0.19 | 0.0331 +0.75 | – |
| 4-D gain–shape, random codes | 0.2914 −0.70 | 0.1624 −0.75 | 0.0928 −1.02 | 0.0430 −0.39 | 0.0115 −0.25 |
| 4-D unconstrained k-means VQ | 0.2150 +0.62 | 0.1141 +0.79 | 0.0596 +0.90 | 0.0315 +0.96 | – |
| D4 lattice ball | 0.2152 +0.61 | 0.1172 +0.67 | 0.0626 +0.69 | 0.0337 +0.68 | 0.0098 +0.42 |
| **E8 lattice ball** | **0.2040 +0.85** | **0.1031 +1.22** | **0.0570 +1.10** | **0.0302 +1.14** | **0.0087 +0.98** |
| Per-channel Lloyd, no rotation/side bits | 0.2175 +0.57 | 0.1177 +0.65 | 0.0655 +0.50 | 0.0347 +0.54 | 0.0097 +0.50 |
| 4-D gain–shape, no vector norm (GS4-rht-abs) | 0.2145 +0.63 | 0.1049 +1.15 | 0.0619 +0.74 | 0.0283 +1.43 | 0.0074 +1.67 |
| Project-style signed grid, pow2 scale | – | 0.2052 [1.69] | 0.0919 [2.42] | 0.0571 [2.91] | 0.0232 [3.56]; 15 lv: 0.0169 [4.02] |
| Shannon D(R) = 2^(−2R) | 0.125 | 0.0625 | 0.0313 | 0.0156 | 0.0039 |

Recall@10:

| Method | 2 bits | 3 bits |
|---|---|---|
| TQ | 0.479 | 0.676 |
| TQprod | – | 0.389 (at 2.84) |
| P2 | 0.514 | 0.697 |
| PQ | 0.445 | 0.714 |
| GS4-vec | 0.498 | 0.714 |
| GS4-abs | 0.533 | 0.744 |
| VQ4km | 0.527 | 0.733 |
| D4 | 0.538 | 0.728 |
| E8 | 0.533 | 0.717 |
| per-channel | 0.510 | 0.719 |

Between reasonable methods, recall differs by only a few points at equal bits.

### 4.2 Other distributions [MEASURED `sweep.py {t3,mvt3,outlier,aniso}`]

Each entry is relMSE @ 2 b / relMSE @ 3 b / recall@10 @ 2 b. `-c` means centered, and was run only
where the mean is non-zero. SC-pow2 entries are at 1.69 / 2.91 b/dim.

| family | gauss | t3 | mvt3 | outlier | aniso |
|---|---|---|---|---|---|
| SC-pow2 | 0.205 / 0.0571 / 0.29 | 0.300 / 0.1036 / 0.62 | 0.204 / 0.0529 / 0.62 | 0.323 / 0.1436 / 0.15 | 0.217 / 0.0565 / 0.25 |
| SCchan | 0.118 / 0.0347 / 0.51 | 0.246 / 0.0917 / 0.06 | 0.302 / 0.1145 / 0.33 | 0.047 / 0.0144 / 0.53 | 0.077 / 0.0228 / 0.50 |
| TQ-qr | 0.137 / 0.0393 / 0.48 | 0.137 / 0.0393 / 0.74 | 0.137 / 0.0394 / 0.71 | 0.127 / 0.0370 / 0.51 | 0.135 / 0.0390 / 0.54 |
| TQ-qr-c | – | – | – | 0.059 / 0.0172 / 0.67 | 0.088 / 0.0256 / 0.55 |
| P2-rht | 0.138 / 0.0397 / 0.51 | 0.135 / 0.0387 / 0.78 | 0.138 / 0.0397 / 0.70 | 0.129 / 0.0381 / 0.59 | 0.140 / 0.0399 / 0.53 |
| PQ-rht | 0.187 / 0.0418 / 0.44 | 0.182 / 0.0422 / 0.73 | 0.188 / 0.0443 / 0.63 | 0.216 / 0.0416 / 0.47 | 0.182 / 0.0420 / 0.55 |
| GS4-rht-vec | 0.127 / 0.0340 / 0.50 | 0.125 / 0.0334 / 0.78 | 0.127 / 0.0340 / 0.71 | 0.108 / 0.0312 / 0.58 | 0.127 / 0.0337 / 0.54 |
| GS4-rht-vec-c | – | – | – | 0.054 / 0.0144 / 0.70 | 0.083 / 0.0220 / 0.59 |
| GS4-none-vec | 0.127 / 0.0341 / 0.50 | 0.154 / 0.0463 / 0.53 | 0.127 / 0.0341 / 0.72 | 0.068 / 0.0200 / 0.72 | 0.114 / 0.0319 / 0.56 |
| GS4-rht-abs | 0.105 / 0.0283 / 0.53 | 0.122 / 0.0393 / 0.56 | 0.198 / 0.0544 / 0.65 | 0.097 / 0.0263 / 0.22 | 0.106 / 0.0287 / 0.34 |
| GS4-rht-abs-c | – | – | – | 0.046 / 0.0123 / 0.62 | 0.072 / 0.0197 / 0.43 |
| VQ4km-rht | 0.114 / 0.0315 / 0.53 | 0.112 / 0.0310 / 0.78 | 0.114 / 0.0316 / 0.72 | 0.105 / 0.0288 / 0.53 | 0.113 / 0.0314 / 0.54 |
| VQ4km-rht-c | – | – | – | 0.048 / 0.0134 / 0.75 | 0.074 / 0.0205 / 0.62 |
| D4-rht | 0.117 / 0.0337 / 0.54 | 0.114 / 0.0327 / 0.79 | 0.117 / 0.0337 / 0.72 | 0.101 / 0.0291 / 0.63 | 0.116 / 0.0334 / 0.58 |
| D4-rht-c | – | – | – | 0.049 / 0.0140 / 0.74 | 0.075 / 0.0218 / 0.60 |
| E8-rht | 0.103 / 0.0302 / 0.53 | 0.101 / 0.0299 / 0.81 | 0.103 / 0.0307 / 0.72 | 0.094 / 0.0271 / 0.61 | 0.101 / 0.0297 / 0.56 |
| E8-rht-c | – | – | – | 0.044 / 0.0129 / 0.78 | 0.067 / 0.0198 / 0.61 |
| E8-none | – | – | – | 0.089 / 0.0260 / 0.60 | 0.098 / 0.0282 / 0.59 |

- **Rotated, norm-stored codes rank identically in all five sets, lowest error first:** E8, then 4-D
  k-means ≈ D4, then H4 gain–shape (always 4th), then TurboQuant ≈ 2-D polar, then random codes, then
  PolarQuant.
- **t3.** Without rotation, per-channel scalar, polar, PolarQuant and gain–shape lose 0.9–3.7 dB.
  With rotation, results look Gaussian.
- **mvt3.** Absolute block radii without a vector norm (0.198) and per-channel scalar (0.302) lose
  1.9–3.8 dB against GS4-vec (0.127).
- **outlier and aniso.** Uncentered rotated codes waste bits on the offset. Centering cuts
  TurboQuant from 0.127 to 0.059 (outlier) and from 0.135 to 0.088 (aniso). After centering, E8 is
  best (0.044 / 0.067), with 4-D VQ, D4, gain–shape-abs and per-channel calibration close behind.
- **Uncentered offset data makes recall erratic.** Examples: 0.009 for SC-opt and 0.22 for GS4-rht-abs
  on outlier. The ranking lives in small variations on a large common component.

### 4.3 Direct test of "preserve the radius" [MEASURED `radial_test.py`, `renorm_check.py`]

Setup: identical rotation, codes and bits; only the radial treatment changes. The configurations
compared:
- Per-block gain options: the MSE-optimal projection (proj), the true radius r, or ablated (one
  shared gain).
- TurboQuant-MSE with or without rescaling to the stored norm.

**Dropping the per-block radius:**

| Code | Ablated | With 1 bit of radius |
|---|---|---|
| 2I | 0.171 @ 1.86 b | 0.104 @ 2.11 b |
| 120-cell | 0.137 @ 2.44 | 0.068 @ 2.69 |
| G2760 | 0.117 @ 2.98 | 0.047 @ 3.23 |

- At equal or fewer bits, keeping the radius wins by 2.1–4.4 dB (0.5–2.3 dB on outlier), e.g.
  2I×4 gains reaches 0.080 @ 2.36 b.
- At the lowest rate, direction resolution matters more. 2I without radius (1.86 b) beats 2T×8 radius
  levels (2.03 b) by 0.7–1.8 dB in all five datasets.
- The sweep's optimal splits give the radius 1 bit at 1.5–2 b/dim, 2 bits at 3 b/dim, and ≈3.3 bits
  at 4 b/dim. This is the classical gain–shape allocation.
- Recall follows MSE on isotropic data (gauss 0.463→0.554).
- Recall reverses on offset data (outlier 0.698→0.594), because the per-block gains add noise along
  the common component.

**Quantizing r instead of the projection:**
- The IP slope moves toward 1 (0.896→0.929).
- MSE worsens by 1–6%.
- Recall is unchanged (±0.01).

**Rescaling TurboQuant reconstructions to the stored norm (zero bits):**
- recall@10 gains +0.017…+0.031 on gauss and +0.058…+0.101 on aniso (3 seeds × 1000 queries,
  paired SE ≈0.003).
- In the 100-query runs: t3 +0.006…+0.012, mvt3 ≈0, outlier +0.06…+0.14.
- The IP slope moves 0.885→0.941 (L=4).
- MSE changes −3% to +4%.

Mechanism [DERIVED]: MSE-optimal codes shrink each vector by a vector-dependent (1−Dᵢ). With a
large common component μ, that shrink multiplies the large ⟨q,μ⟩ and scrambles the ranking.
Similar norm corrections may exist in the vector-search literature; I did not check this session.

### 4.4 Serving: lookup-table scores [MEASURED `serving.py`]

Setup: keys use RHT and absolute gain–shape codes with no norm, so scores are pure table reads.
Gaussian data, 10k keys, 100 queries.

| Keys (bits/dim) | Query | recall@10 | IP err | attn TV |
|---|---|---|---|---|
| 2I×2 gains (1.98) | full precision (per-query table) | 0.512 | 0.326 | 0.319 |
| | same 2I code (9 Gram values) | 0.393 | 0.447 | 0.429 |
| | G2760×16 (57 Gram values vs 2I) | 0.507 | 0.339 | 0.328 |
| 2I×4 gains (2.23) | full / same code | 0.608 / 0.482 | 0.284 / 0.392 | 0.281 / 0.378 |
| | same code, **integer ℤ[φ] accumulation** | 0.483 | 0.392 | 0.378 |
| 120-cell×4 (2.81) | full / same (31 values) | 0.683 / 0.586 | 0.207 / 0.291 | 0.207 / 0.273 |
| G2760×8 (3.61) | full / same (329 values) | 0.823 / 0.753 | 0.111 / 0.156 | 0.104 / 0.146 |
| scalar 4 levels (2.00) | full / same (4×4 table) | 0.484 / 0.349 | 0.341 / 0.469 | 0.317 / 0.424 |
| scalar 16 levels (4.00) | full / same | 0.843 / 0.792 | 0.097 / 0.137 | 0.095 / 0.133 |
| 4-D k-means 256 (2.00) | full / same (256² Gram) | 0.542 / 0.408 | 0.313 / 0.431 | 0.291 / 0.388 |
| E8 ball (1.97) | full / same (integer dot) | 0.532 / 0.419 | 0.307 / 0.422 | 0.288 / 0.385 |

**The algebra holds.**
- The 2I Gram takes the 9 values {0, ±½, ±φ/2, ±1/(2φ), ±1}.
- ⟨a,b⟩ = Re(a·b̄) is a class function of a·b̄, and 2I has 9 conjugacy classes. These are the
  project's "nine scores" [SOURCE docs/integration/stuck-point-review-response-2026-09-24.md:112].
- An integer (A,B) accumulation of A + Bφ, with 1/16-step gains and a 16-bit φ, matched float.

**The query penalty is generic.** It is ≈0.12–0.13 recall at 2 bits, ≈0.07 at 3–3.6 bits and ≈0.05
at 4 bits, for every codebook.

**Dynamic range.** Without a per-vector norm, the gain quantizer helps. On mvt3, 2I×8 gains reach
0.756 against 0.220 for fixed-scale E8 and 0.337 for 4-level scalar. An 8-bit per-vector norm removes
the difference.

**Centering (outlier data).** Serving recall at 2 bits rose from 0.08 / 0.12 / 0.20 / 0.50 to
0.50 / 0.57 / 0.59 / 0.63 (scalar / E8 / 2I / k-means), and the query penalty nearly vanished.

**Cost per key at d=64 [DERIVED]:**
- Today: 64 software shift-add products [SOURCE crates/uor-r4-integer/src/math.rs:61-83].
- 2I keys with a full-precision query: 16 table reads + 16 adds. The per-query table (16×120×gains)
  is built with shift-adds, because the coordinates lie in {0, ±½, ±φ/2, ±1/(2φ), ±1}.
- Symmetric scoring: 32 reads + 16 adds.
- These are operation counts, not timings.

## 5. Findings (numbered)

1. **Premise and record.** No published method ablates the radius, and the router paper mis-cites
   TurboQuant (§2). [LITERATURE][SOURCE]
2. **Prior art exists.**
   - Sabin & Gray 1984 [LITERATURE https://doi.org/10.1109/tassp.1984.1164346].
   - Hamkins & Zeger 2002: within 1 dB of D(R) with wrapped Leech shape codes
     [LITERATURE https://doi.org/10.1109/tit.2002.804056].
   - HQMQ: 4-D quaternion chunks, a b_r-bit radius, 2T·random codebooks, Pareto over naive int at
     3–5 b [LITERATURE https://arxiv.org/abs/2605.27646]. Its preliminary note that "E8 underperforms
     24-cell" contradicts my lattice-VQ result; the variants differ.
   - FibQuant, which proves vector codes beat TurboQuant's scalar tables
     [LITERATURE https://arxiv.org/abs/2605.11478].
   - Block-Sphere VQ, SPHQuant and IsoQuant [LITERATURE abstracts: 2605.19972, 2609.24875, 2603.28430].
3. **The product code is structurally handicapped** [DERIVED]. Gain × shape places N directions at
   every radius, so the point density falls like r⁻³. The optimal fixed-rate density follows
   f^{k/(k+2)}, which lattice balls approximate far better. Measured cost at 2 b: 0.36 dB against D4,
   0.48 dB against 4-D k-means, 0.92 dB against E8. The factorization is worth keeping only when you
   need dynamic range or group equivariance.
4. **The 600-cell is a near-optimal code, not a capability source.** E[1−cos²] is 0.069 against 0.098
   for a random 120-point code; spherical k-means only matches it. [MEASURED]
5. **Unbiasedness does not help ranking.** TurboQuant-prod is unbiased (slope 1.004), but at m=d its
   MSE is (π/2)·D_mse(b−1): 0.546 measured at 2.25 b, matching the derivation. Its recall is the worst
   of the rotated methods (0.389 at 2.84 b). Top-k and softmax ignore a common scale; HQMQ reports the
   same negative. [MEASURED][DERIVED]
6. **The project's weight grid is weak on Gaussian rows** [MEASURED, synthetic]. The pow2-scaled
   signed grid [SOURCE joint_quantization.rs:24] gives 0.0169 at 4.02 b. A free scale gives 0.0101
   and E8 0.0087; E8 matches 0.0169 at ≈3.5 b. The project's learned rounding (+0.024/+0.046 nats over
   continuous [SOURCE docs/integration/current-state.md:242]) may absorb part of this; untested.
7. **E8 ≤2.2 b/dim is signed4-compatible.** Doubled, every point of the E8 ball of norm ≤14 (199,921
   points, 2.20 b/dim) has integer coordinates in [−7,7]. The signed4 product-table kernel
   [SOURCE docs/integration/current-state.md:42-47] could run it unchanged, at about half the weight
   storage. [MEASURED enumeration]
8. **2I-snapped transport is exact but coarse.**
   - Left multiplication by q ∈ 2I permutes 2I indices through a 120×120 table and preserves radii.
   - But the smallest non-identity step is a 36° isoclinic rotation.
   - The transport is parameterized near identity, q = normalize(e0 + 0.1·raw)
     [SOURCE crates/uor-r4-training/src/joint_model.rs:28,1653], and the GRU blend breaks exactness.
   - The plan already defers 2I snapping
     [SOURCE docs/integration/quantized-recurrent-plan-2026-09-25.md:82-83]. [DERIVED]
9. **This experiment is still owed.** The "matched H4/k-means/random vector-codebook comparison
   remains separately unrun" [SOURCE docs/integration/quantized-recurrent-plan-2026-09-25.md:108-110],
   and the 600-cell diagnostic is NOT_RUN [SOURCE docs/integration/current-state.md:487]. My synthetic
   results are a prior for it, not a substitute.

## 6. Strong / novel vs weak / wrong

- **Strong:**
  - 4-D blocks do beat scalars.
  - The 600-cell is an excellent S³ code.
  - The 2I/ℤ[φ] algebra gives exact multiplier-free score tables that fit the existing ZPhi code
    [SOURCE docs/integration/architecture-2026-09/mathematics.md:19].
  - The separate radius gives dynamic range when no norm is stored.
- **Novel:** little. 2I-based codes with geodesic refinements, and ℤ[φ] score accumulation, are small
  steps from HQMQ, FibQuant and the classical literature.
- **Weak or wrong:**
  - The TurboQuant premise.
  - Crediting "radius preservation" with the gain; it is generic VQ, and the product structure costs
    0.3–0.9 dB against lattices.
  - Stopping at 4-D, when E8 (8-D) and Leech (24-D) are standard.
  - The router paper's citation.
  - No measurement on the project's real vectors.

## 7. Recommendations (ranked)

1. **Run the owed codebook comparison offline, on the retained checkpoint** (hours).
   - Codes: signed4-pow2 (current), E8 ≤2.2 b (signed4-compatible), D4, 2I gain–shape, 4-D k-means.
     Apply them to RHT'd matrices and to dumped read Q/K/V.
   - Measure comparison-tail NLL and read-mass TV.
   - Expected: E8 matches the current MSE with ~0.5 fewer bits/weight. NLL effect unknown.
   - Falsify: if E8 at ~2.2 b costs more than +0.1 nats beyond today's +0.024/+0.046, drop sub-4-bit
     weights. If nothing beats signed4 by ≥0.01 nats, keep scalar.
2. **Prototype table-lookup scoring in the event-memory read.**
   - Full-precision query, keys as centered 4-D or E8 codes, per-key ⟨μ,k̂⟩ stored as one scalar.
   - Expected: 64 software products per key become 16 reads + adds. A full-256 Read call, which covers
     more than q·k, averages ≈3.7 ms today [SOURCE docs/integration/current-state.md:47].
   - Falsify: a speedup under 2×, an NLL loss over 0.02 nats, or TV over 0.1. Synthetic TV reached ≈0.1
     only at ≥3.6 b/key.
3. **Adopt the free fixes wherever vectors are quantized.**
   - Store an 8-bit per-vector norm, and rescale reconstructions to it (+0.02–0.14 recall).
   - Subtract a global mean before rotation.
   - Rotate with the add-only RHT for heavy-tailed data.
   - Never use QJL-style unbiasing for attention or ranking.
4. **For "H4-native" quantization, use D4 (24-cell Voronoi) or E8 (the project's H4 ⊕ φH4 icosian
   shorthand) lattices, not a gain × 600-cell product.** Keep the 2I product only where its algebra is
   used.
5. **Do not quantize the recurrent state per step, or snap transport to 2I, without a retrained
   no-loss result.** Cheap falsifier: inject 2- and 4-bit gain–shape noise per step in the F32
   emulator and measure NLL drift by position. [HYPOTHESIS]
6. **Correct the router paper.** Fix the citation and the priority claim, and cite HQMQ, FibQuant,
   Block-Sphere and PolarQuant (Wu).

## 8. Open questions

- Behaviour on the project's real keys, values, weights and state, which live on the owner's machine.
  The synthetic results say centering and calibration matter more than the codebook.
- Whether ~1 dB of MSE moves NLL for a ~1.7M-parameter model with learned rounding.
- Rust wall-time and energy of lookup scoring on an M1 (only operation counts here).
- d=128, where the norm overhead halves (not run).
- HQMQ's E8-vs-24-cell result (not reproduced).
