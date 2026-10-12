# The D23 replacement learning law: geometric replacements for softmax, built for GPU

References #2028 (claim, [comment 6124719596](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6124719596)),
[D23](../../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project),
[D24](../../integration/DECISIONS.md#d24---no-cpu-model-compute-and-no-gpu-on-non-decisive-work),
the [softmax census](../softmax-census-2026-10-12/README.md). Lab: Kimi, session `kimi/s1`.
Revised 2026-10-12 under two owner directions of the same day: **no softplus or any other softmax-shaped
term** (the carve-out is withdrawn), and **rebuild every softmax mechanism with the project's geometric
processes, all on CUDA** — with the SpiralCore site and the elliptic-curve murmurations paper (both
owner-supplied) deep-reviewed as the geometric basis. Design + a first implemented rung; model compute
waits for TEST FITNESS and runs on pods only.

> **One line.** Softmax does three jobs in this project — the training objective, the read's weighting, and
> the served output distribution — and the lawful replacements are geometric and already begun: a
> **rank-margin objective** (hard hinge, no exponential of any kind, **implemented with its CUDA f32
> kernel**), the **rank-table read** (already served, exp-free forward; its score-gradient stop and CUDA
> kernel are the named next kernels), **exact hard selection over the 600-cell with rank-margin address
> credit** (the murmuration read, teacher-free), and **raw-score greedy output** (already clean). The
> geometric processes these replacements ride on are the SpiralCore-verified structures below, carried in
> the CUDA kernels themselves.

## 1. The geometric basis (the owner-supplied deep review)

**SpiralCore v69 (the site, reviewed in full).** A geometric instrument whose coordinates are
*computationally verified*, and which supplies the project's exact high-dimensional substrates:

- **The 120 unit icosians are the real H4 root system** (the 600-cell; Spin(H3) = 2I, Dechant 2021 Thm
  2.16), 12-nearest-neighbor, root trace form exactly 30×I — the same structure the stack's starling ring
  buffer already addresses (nearest-of-120 cells + 12 neighbours). The addressing substrate is real,
  verified geometry on both sides.
- **D4 (24 unit quaternions) and F4 (48)** verified with their Killing forms (6×I, 18×I), and the **E8
  240-root system** (112 integer + 128 half-integer; rank 8 by Gaussian elimination, Cartan det 1,
  57,600 reflection checks), with the **15 Cl(0,6) bivectors from six octonion left-multiplication
  generators** acting on it: 3,600 labelled action records, 2,580 distinct directed pairs, **15 disjoint
  16-root operator orbits** — an exact, finite, typed routing plane with no arithmetic approximation.
- **The six intrinsic H2 decagons** inside the 30 H3 roots (10-root rings, 36° spacing, edge 1/φ, Cartan
  [[2,−φ],[−φ,2]]), pairing bijectively with the six 5-fold axes — the K6 ↔ half-turn correspondence the
  repo's E1 pre-registration already cites. Plus the Grand Antiprism 20/100 split inside the same 120
  points (two mutually orthogonal 10-rings), and the **FBS binder tree** with support completion
  (Q = 2L+1) and sparse marks — the structure behind the repo's landed persistent base-3 starling tree.
- **Kuramoto phase coupling** (`dθᵢ/dt = ωᵢ + K Σ Aᵢⱼ sin(θⱼ − θᵢ)`): a synchronization dynamics that
  produces *global weighting from local phase rules with no normalization anywhere* — the one mechanism in
  the site that computes attention-like global structure without a softmax-shaped step, and the natural
  candidate readout law for the murmuration read (pre-register against a shuffled-geometry null).
- Honesty the site models and the project must keep: it is a verified *instrument*, and nowhere claims the
  geometry carries predictive signal. "Verified coordinates" is not "useful attention" — the geometry
  audit's 0.05 % lesson applies here too.

**Murmurations of Elliptic Curves (He, Lee, Oliver, Pozdnyakov, arXiv:2204.10140v2, reviewed in full).**
The average Frobenius trace a_p over rank-and-conductor-fixed elliptic curves oscillates with growing
amplitude and period — the murmuration — fitted by `A x^α sin(B x^β)` with β ≈ 0.5; the oscillation's
*shape classifies the rank* (logistic regression > 0.96–0.9998; PCA separates; the first ~10–100
coefficients carry the signal; the PC1 weights themselves oscillate at the same frequency). What the
project takes from it, precisely: **a sequence's role is recoverable from the oscillation pattern of its
traces** — role information can ride on *trace patterns* and be read by rank/margin classifiers, with no
semantic embedding and no softmax. What it is not: the repo's starling read (a flock-like local route) —
the earlier record's distinction stands: the paper is the arithmetic-murmuration family, the read is the
starling-flock family, and the bridge between them is the *style* of evidence (oscillation patterns as
carriers), not a shared operator.

## 2. The replacements, by role

| role | banned today | the geometric replacement | state |
|---|---|---|---|
| training objective | softmax CE (LSE fwd, exp bwd) | **rank-margin hard hinge** `mean_k relu(1 − z_t + z_k)`, no exponential at all | **implemented + CUDA f32 kernel (this PR family)** |
| read weighting | causal softmax | fixed rank table over exact hard flock selection (exp-free fwd) + **score-gradient stop** (kills the softmax STE in the bwd) | fwd served; the stop + the read's CUDA kernel are the next kernels |
| murmuration addresses | softmax surrogate / KL / dense bootstrap (all banned) | **rank-margin address credit** (true cell out-scores rivals by a hinge) over the 600-cell; Kuramoto phase readout as the named alternative with a shuffled-geometry null | design; needs the read kernel |
| served output | `softmax(z)` head + mixture | raw-score greedy argmax (non-pointer models already compute no softmax in `next_scores`); a geometric (rank/sparse) mixture for pointer models is the next rung | non-pointer clean today |
| evaluation | LSE NLL | margin curves, unviolated-target rate, the frozen panels' deterministic checks | active |

The margin objective's CUDA kernel (`margin_rank_rows`, `margin_rank_grad`, one warp per row, ties to the
lower index as the CPU stable sort) is the first of the **vector-space-transition kernels** the owner
asked for: the next kernels carry the transitions themselves on-device — quaternion/H4 transport (already
in the recurrence kernels), icosian cell assignment and the 12-neighbour gather (the starling read),
the rank read's forward/gradient with the score-gradient stop, and the E8/Cl(0,6) operator orbits where
the E8 memory line needs them. No host fallback: a path that lands on the host is a defect (D24), so each
of these is a kernel, not a bridge.

## 3. Rung 0 (pre-registered; TEST FITNESS before compute)

**Question:** does the hard-hinge margin objective train a small geometric stack to attempt the recall
task at all — cleanly enough to be the project's objective?

- **Model:** geometric stack, width 256, **recurrence-only pattern `rrrrrr`** (no read layers, so no read
  softmax in forward or gradient and no host fallback), 4 heads, MLP 384, context 384, vocab 4096, **no
  pointer** (the pointer's own objective is the next rung). Serving for scoring computes no softmax by
  construction.
- **Data:** the project's published `mw-balp` train (7,512,797 tokens) + dev split.
- **Arms (2 seeds each, one pod, GPU only):** M1 = hinge at 1 rival; M8 = hinge at 8 rivals. **No CE arm
  and no soft arm** (both banned); the reference is the historical CE artifacts' recorded numbers at their
  exact scope, quoted as cross-objective readings, never as like-for-like baselines.
- **Bars (fixed in advance):** an arm works if the dev margin loss falls monotonically and the
  unviolated-target rate on dev exceeds the panel's no-model constant rate by ≥ 5× on both seeds. Below
  that is a tooling/objective blocker to fix, not a mechanism negative (D22).
- **Readouts:** dev margin loss and unviolated-target rate per arm. The recall panels (mqar, v5) are
  **rung 0b**, which needs the rank read's CUDA kernel and the score-gradient stop so a read can train
  lawfully.
- **Cost:** one 1×4090 pod, ≤ $2, leased/released per the pod rules.
- **Stop rule:** two arms × two seeds, one run each; no silent additions.

**Rung 0b (named next):** the rank read's CUDA kernel + the score-gradient stop, then the read-bearing
margin objective and the murmuration address credit (the murmuration read's lawful training law), then the
recall panels.

## 4. What this is not

- No softmax, no softplus, no sigmoid-shaped loss term, no teacher, no dense bootstrap — anywhere, in
  training, serving, scoring, or gradient.
- Not a claim that the margin objective reaches CE-level language quality: the rung measures exactly that
  at its scope, with margin curves and panel accuracy, not incomparable NLLs.
- Not a softening of D24: every model-compute step above is a CUDA kernel on a pod; nothing runs the model
  on the laptop beyond unit tests of the ops themselves.

## Next

1. TEST FITNESS on rung 0 (#2029).
2. The margin-objective prep PR (this implementation + its CUDA kernel), with the pod compile receipt.
3. Rung 0 on one pod; then rung 0b's kernels (rank-read CUDA + score-gradient stop + address credit).
