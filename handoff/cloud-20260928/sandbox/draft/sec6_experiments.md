## 6. Experiments run for this review

All runs were small, CPU-only and on a shared x86 box, and none is a language-quality claim about the project's model. Their scripts are listed in Appendix B.

### 6.1 The original idea, measured at matched bits (quantization report)

Setup: i.i.d. Gaussian vectors, d=64, with an 8-bit norm counted for every method. Also heavy-tailed, outlier-channel and mean-shifted variants.

| Bits/dim | 1.5 | 2 | 2.5 | 3 | 4 |
|---|---:|---:|---:|---:|---:|
| 4-D gain–shape with H4-derived codes vs TurboQuant-MSE | +0.19 dB | +0.30 | +0.24 | +0.64 | +0.93 |
| D4 lattice (Voronoi cell = 24-cell) vs TurboQuant-MSE | ≈ +0.6–0.7 dB across rates | | | | |
| E8 lattice vs TurboQuant-MSE | ≈ +0.85–1.22 dB across rates | | | | |

- **The H4 structure itself does not help.** Spherical k-means codes of the same size match the 600-cell/2I codes (relative MSE 0.1288 vs 0.1274 at 2 bits). Random codes are 0.8–1.3 dB worse.
- **Keeping the radius** is essential *inside* a 4-D block: dropping it costs 2–4 dB. But coding radius and direction as separate factors costs 0.3–0.5 dB against an unconstrained 4-D code. The literature agent's fixed 120-point code saturates at 3 bits/dim (§2.1).
- **The data distribution matters more than the codebook.**
  - Heavy-tailed coordinates need a random rotation; without it every method loses 1–4 dB.
  - Heavy-tailed norms need a per-vector norm.
  - Fixed outlier channels favour per-channel calibration.
- **Serving.** Inner products between 2I codewords take exactly 9 values in ½ℤ[φ]. An integer ℤ[φ] lookup-table scorer matched the float path (recall@10 0.483 vs 0.482). Quantizing the *query* with the same ~2-bit code costs about 0.12 recall@10, and the same holds for scalar, k-means and E8 codes. So keep the query at full precision, with per-query tables built from shift-add constants.
- **Best project use.**
  - Keys and values of the attention and event-memory read, scored by table lookup. This replaces the 64 software multiplies per key in the current integer read (model.rs:322-329).
  - For weights, E8 (icosian lattice) codewords up to about 2.2 bits/dim are exactly signed-4 integers, so the existing kernel is reusable.
  - Do not quantize the recurrent state at every step.

**Verdict.** A sound engineering component with about 1 dB of headroom over scalar quantization. It is not, by itself, a breakthrough, and its novelty is low: HQMQ, PolarQuant, FibQuant and Block-Sphere Quantization precede it.

### 6.2 State tracking with geometric lanes (state-tracking report; math report E6)

**Setup.** Word problems with a target at every position, trained on lengths ≤32 with a curriculum. All models share a state size (D=32) and an MLP readout.

**A5, three generators, tested to length 512** (chance 0.017):

| Model | Runs | Accuracy at L=512 | Exact at 512 |
|---|---|---:|---|
| Quaternion lanes, free parameterisation (8 lanes, 128 recurrent parameters) | 5 seeds | 0.61 ± 0.48 | **3/5 (5/7 including the learning-rate sweep)**; bimodal |
| Repo form q = normalize(e0 + 0.1·raw) | 3 learning rates, plus 3× steps | ≤0.017 | 0/4 |
| Householder pair H(v)H(e0) = v·x·v, free | 3 seeds | 0.71 ± 0.20 | 0/3 (best 0.987) |
| DeltaProduct-like (4 reflections, β<2) | 3 seeds | 0.017 (0.53 at L=128) | 0/5 |
| GRU (32 units) | 3 learning rates, plus 3× steps | ≤0.017 | 0/4 (a sample-efficiency result at this budget) |
| Diagonal (0,1), signed diagonal, unit-complex, LRU | 3 learning rates each | ≤0.017 | **0/12** |

**Full 60-letter alphabet** (math E6): four quaternion lanes with a curriculum reached 1.00 at 16× the training length, reproduced on a second seed. Two learned reflections matched them. The repo's Householder control could not represent the task, and diagonal lanes failed. One lane, or no curriculum, stayed at chance.

**Exact integer serving.** Converged lanes were snapped to 2I and served as a **120-state automaton**: state = T[state, token], class = C[state]. Per token that is one byte of state and two table reads, with no arithmetic.
- 4 of 5 successful runs scored 1.000 at every length up to 4,096; the fifth held a flat 0.984.
- The float32 originals of two of those runs drifted to **0.74 and 0.62** at lengths 2,048–4,096.
- On the abelian Z60 control, float rotation models collapsed by length 128. Snapped to exact characters, they held 1.000, 0.967 and 0.749 at every length.

**Limits.**
- Only 1–2 of 8 lanes converged; the rest behaved like lottery tickets.
- S5 reached only parity-level accuracy (0.016), as the theory predicts for rotation-only lanes.
- Everything here is toy scale: 3–60 generators, D = 32, about 1–2k steps.

**ℤ[φ] kernel** (verified on all 120 elements × 200 vectors): a 2I rotation of a doubled-coordinate icosian vector costs 24 add/sub plus 8 shifts. The 8 axis elements cost 0. Coefficients stayed bounded (max |coef| = 6 over 20,000 steps).

### 6.3 Real-text comparison of time-mixing mechanisms (lead; WikiText-2 bytes)

**Setup.** A 3-layer byte-level language model:
- d = 128, a GLU channel mixer and RMSNorm;
- about 0.56M parameters (0.76M for the GRU);
- 1,000 steps × 4,096 bytes = 4.1M training bytes;
- AdamW with warmup and cosine decay;
- evaluated on the first 256 KiB of WikiText-2 validation in windows of 512 bytes.

The linear recurrences are trained with an associative scan. Every time-mixer uses four d×d maps; the GRU has six.

| Time-mixer | Bits/byte, seed 0 | Notes |
|---|---:|---|
| Diagonal real decay (Mamba/minGRU-like, commutative) | **1.869** | Fastest to train |
| Quaternion rotation + radial decay (continuous) | 1.881 | Snapped to 2I after training: **2.060** |
| Quaternion, trained with straight-through snapping to 2I in every lane | 1.984 | All lanes forced onto the 120-element group |
| Complex (2-D rotation + decay, commutative) | [running] | |
| Quaternion snapped to the integer small-rotation codebook | [queued] | |
| Softmax attention + RoPE (transformer-style reference only) | [queued] | |
| GRU (nonlinear, sequential; +35% parameters) | [queued] | |
| Second seeds: quaternion, diagonal | [queued] | |

**Reading (preliminary; one seed).**
- Non-commutative rotation gives **no** language-modelling advantage over diagonal decay at this scale, as the literature predicts.
- Forcing *all* lanes onto the coarse 2I group costs about 0.10 bits/byte.

Both support the lane-mixture design in §8.4: exact 2I only for tracking lanes, finer rounded lanes for content.

### 6.4 Golden-gate rotation codebooks (lead)

- **Level 1** (words c·T·c with T = (2+φ)i + j + k): exactly 3,600 rotations, matching the theory. After scaling by √(7+5φ), every coordinate is exactly in ½ℤ[φ].
- **Covering** (angle from Haar-random test rotations to the nearest codeword, SO(3)):
  - Mean 9.32° vs 8.83° for a random codebook of the same size; maximum 30.5° vs 22.8°.
  - A level-2 sample of 51,657 rotations is about equal to random: mean 3.65° vs 3.65°.
- **Near the identity** (T-count ≤1, 3,660 rotations): mean 13.4° vs 10.0° for random.
- **Reading.** Parzanchevski–Sarnak prove optimal *almost*-covering asymptotically and also predict rare big holes. At practical sizes these codebooks bring exact algebra and navigability, not better compression. The measured hole near the identity makes them a poor fit for recurrences, whose useful transitions are mostly small rotations.
