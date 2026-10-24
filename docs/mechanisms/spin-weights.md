c=1
# Spin-native weights (quaternion and icosian maps)

**Status: READY (owner ruling, 24 Oct 2026, D74).** The quaternion weights, snapped to a residual pair of icosians with a 2-bit scale, met their bar on a small test (1.5148 bits per byte: not distinguishable from a dense model with the same total parameters, and 0.107 behind a dense model of the same width; a single-icosian snap lost badly), but only by running a dense Hamilton-block multiplication, which the owner's rules ban. The code that ran them now refuses to run. The mechanism stays open.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

A network's weight matrix is usually a grid of independent numbers. Here each 4-by-4 block of the grid is replaced by one quaternion, a set of four numbers that describes a turn and a scale in four dimensions. That makes the grid four times smaller. After training, each quaternion is rounded to one of 120 exact rotations of the icosahedron (the 20-sided solid), plus a small scale. A product of two such rounded blocks then becomes a table lookup and a few shifts and adds, with no multiplication. The size and quality results were good. The speed benefit was never built or measured.

## What it replaces in a transformer

The weight matrices of the projections: the grids that turn a word's vector into queries, keys, values, recurrence inputs and feed-forward layers.

![Left: a sphere carrying the 120 icosian directions as dots. A trained quaternion is drawn as an arrow, snapped to its nearest dot about 15 degrees away, and then a second residual icosian pulls the arrow to within 4.7 degrees. Middle: the 120 by 120 product table as a grid, where choosing a weight icosian and an input icosian lights one cell that reads out the product icosian, then a shift and one or two adds. Right: a bar chart of held-out bits per byte for the dense, quaternion and snapped variants, with the equal-weight-bytes pair highlighted.](../figures/mechanisms/spin-weights.svg)

*Round a quaternion to the nearest of 120 rotations, then correct with a second small one; the product becomes a table read. The bars show what each step cost in quality.*

## How it works

**Compact form (Definition).** Each projection map is stored as an array $Q[\text{out}/4,\ \text{in}/4,\ 4]$, one quaternion $(a,b,c,d)$ per 4×4 block, so there are four times fewer weights. A block acts as the Hamilton left-multiplication matrix
$$L(a,b,c,d)=\begin{pmatrix}a&-b&-c&-d\\ b&a&-d&c\\ c&d&a&-b\\ d&-c&b&a\end{pmatrix}.$$
The output quaternion is $y_j=\sum_k Q_{jk}\otimes x_k$, with the input read as $\text{in}/4$ quaternions. Each block costs 16 multiply-adds, so by itself the compact form saves weights, not arithmetic (**measurement**: width 256 uses the same 5,537,792 multiply-adds per token as the dense width-256 model).

**Snap (Definition).** Each trained quaternion becomes
$$m\cdot 2^{e}\cdot\Big(g_1+\tfrac14\,g_2\Big),$$
where $g_1$ is the nearest of the 120 unit icosians (the binary icosahedral group, exact over $\mathbb Z[\varphi]$), $g_2$ is the icosian nearest the leftover residual, $m\in\{1,\,1.25,\,1.5,\,1.75\}$ is a 2-bit mantissa and $e$ is a power of two. That is about 19 to 20 bits per block instead of 128 for four 32-bit floats. Activations can be snapped to the same set after training.

**Table product (Definition, never built).** The 120 icosians multiply by a fixed 120×120 table. The records describe a snapped product as a table read plus shifts and at most two adds. The group table itself was verified: all 14,400 products (120×120) matched in the 2026-09-20 prefix-recovery check (**measurement**, [record](../../archive/integration/readout-result-review-2026-09-20.md)). A kernel that serves exact-set by exact-set block products was never written.

**Why one icosian is too coarse (measurement).** The 120 directions sit about 15 degrees apart. Adding the residual icosian cut the mean angle error from 14.75 to 4.73 degrees, about 9 times less direction error energy.

**Rule check.**
- *D11 (integer serving):* the weight side is multiplier-free by design. The activation side and the table kernel were never built, and export of quaternion-weight models is refused in the code.
- *D23 (no softmax):* the mechanism has none; snaps pick the nearest point.
- *D25/D30 (no dense matrix multiplication):* **fails as run.** The forward expanded each map to a dense matrix and called a library matrix multiply, and the snap itself was a matmul against the 120 icosians. The fused Hamilton kernel (OP2) is a handwritten dense block contraction, which D25 does not accept as a replacement. Both execution paths now refuse. The ban retired the contraction; it was not retired by a negative result.
- *D32/D37:* the historical form is float. D32 froze it and D37 lets it back only as a new offline float sheet with a fresh review ([decisions](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)).

## What has been tried

All training rows below are small: width 256, a recurrence-heavy stack, 2,000 steps on a chat corpus, 2–3 seeds. The model around the weights still used softmax (the quaternion-weights and icosian-mantissa records both state "softmax at runtime: yes"), so these runs isolate the weight representation, not a lawful model. Scores are held-out bits per byte (BPB), lower is better.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 19 Sep | Exact group-table composition replacing the 16-multiply Hamilton product on the serving path | Exact table and multiplier-free path verified. The replaced site was about 1,000 multiplies per generated token. No chat ability, training or energy measurement | positive | [handoff](../../archive/integration/handoff-2026-09-19.md) |
| 20 Sep | Prefix recovery with the multiplication table bound through a signed-root bijection (PR #1306) | 14,400 products checked; the old learned function was preserved | positive | [record](../../archive/integration/readout-result-review-2026-09-20.md) |
| 11 Oct | Quaternion weights on every map: dense width 256 (D256), dense width 152 with equal total parameters (D152), dense width 128 with equal map weights (D128), quaternion width 256 (Q256) | D256 1.4078, Q256 1.4959, D152 1.5136, D128 1.5520. Q256 beats D152 by 0.0177 (bar 0.0059, noise 0.0029) and costs 0.088 against D256. Weight bytes 8.55 MB against 8.62 MB | positive (KEEP) | [record](../../archive/labs/quaternion-weights-2026-10-11/README.md) |
| 11 Oct | Single-icosian snap: after training, then trained in with a straight-through gradient | After training: +0.141 BPB. Trained in: +0.103 to +0.140. Mean cosine to the original 0.965 | negative | [record](../../archive/labs/icosian-qat-2026-10-11/README.md) |
| 11 Oct | Residual icosian pair, trained in | Direction error 14.75 to 4.73 degrees. Full-precision norm +0.016; power-of-two norm +0.071. Training was 11 times slower on the host snap | inconclusive | [record](../../archive/labs/icosian-pair-2026-10-11/README.md) |
| 11 Oct | Pair with a 1- or 2-bit mantissa on the norm (QM2), 3 seeds, one RTX 4090 | QM2 1.5148, +0.0012 against D152 (about one noise unit), +0.020 against the full-precision quaternion model. GPU snap 7.8 times faster than the host loop. 6 runs, 8.5 minutes, about $0.15 | positive (KEEP) | [record](../../archive/labs/icosian-mantissa-2026-10-11/README.md) |
| 11 Oct | Snap every map input to the same exact set, after training and trained in | After training +0.0196 BPB (1.5343); trained in +0.0231 (1.5379). A configuration result | negative | [record](../../archive/labs/activation-snap-2026-10-11/README.md) |
| 13 Oct | Fused Hamilton projection kernel (OP2) replacing the expanded dense multiply | Implemented and checked locally. 949 library tests pass; agreement with the dense expansion below $10^{-5}$. No run, no device receipt, no model numbers. Superseded within days by [SpiralCore/FBS](./spiralcore-fbs.md) | blocked by rule | [record](../../archive/labs/hamilton-linear-2026-10-13/README.md) |

**Reading these honestly.** "Level with dense" means level with a dense model of the same total parameter count (width 152), not the same width. At equal width the quaternion model is 0.088 BPB worse. The 0.0012 gap is about one noise unit, so the claim is "not distinguishable at this scale", not "better". No fresh-transfer measurement is recorded for either KEEP, which decision D22 asks for. The trained-in activation snap lost by about twice the noise, with one gradient bridge at one scale; the pre-registered soft-to-hard schedule was never tried. Nothing was measured in tokens per second or energy.

## What has never been tested

- **A table-lookup product kernel** for exact-set by exact-set blocks, in any form.
- **Tokens per second, wall-clock or energy** of any quaternion or icosian map.
- **A soft-to-hard schedule** for activation snapping, or a finer set of directions (a third icosian term, or the 2,160-point second E8 shell).
- **Any width above 256 or run longer than 2,000 steps.** The +0.088 BPB cost of four-times-fewer weights was measured only there.
- **Device parity for the fused kernel.** The tests exist but were skipped.
- **Fresh transfer** for the quaternion and icosian KEEPs.
- **Quaternion maps inside a routed or selected structure** instead of an all-input sum.

## A fair test needs

*From the earlier brief, [archive/mechanisms/spin-native-weights.md](../../archive/mechanisms/spin-native-weights.md).*

- Equal total parameters **and** equal weight bytes per token against dense.
- A panel that first shows it separates model sizes.
- At least 2 seeds, with paired per-block noise.
- For the table-product rung: the snap trained in, not only applied post hoc, before any negative counts.

## What would reopen it

1. A table or Cayley-product path (120×120 read, shifts, adds) with no dense all-input contraction, for example applying the block product only to blocks chosen by routing, as the SpiralCore/FBS projection (PR #2279) does. It must be a new D37 sheet: fresh test-fitness review, owner release, a separately scanned float module, and an export gap.
2. An explicit architecture decision for a Hamilton-block trunk. The 18 October architect audit says the active discrete model has no dense projections to replace, so this is a new architecture. "Spinor/quaternion adapters" are already listed as a conditional item, to be considered only at a measured ceiling.
3. A soft-to-hard activation-snap schedule, blending $\alpha\cdot\text{snap}+(1-\alpha)\,x$ with $\alpha$ rising to 1, compared with the +0.0196 BPB post-hoc cost, before any negative on activation snapping counts.
4. A served-form table kernel measured in tokens per second on the same CUDA harness, with weight bytes per token held equal to the dense control (8.55 MB against 8.62 MB in the rung-0 study).
5. At least 2 seeds with paired per-block analysis (noise 0.0012 to 0.0041 BPB in these records) and fresh transfer for any KEEP.

Open questions: how much of the activation-snap cost is direction and how much is scale; whether to snap during training or build directly in served form; whether a finer exact group (the 600-cell plus a scale codebook, or the second E8 shell) is better; and whether the equal-weight-bytes gain survives at scale. These are **hypotheses**.

## Also known as

Quaternion weights (`quaternion_weights=all`); Hamilton-block projections; `HamiltonLinear` (OP2); residual icosian pairs; icosian Cayley-table products; icosian-mantissa weights; spin-snap.

## Code

- [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs): `QuaternionWeights`, `QuaternionSnap` (including the pair and mantissa forms), the straight-through snap and `ActivationSnap`. The dense weight expansion and the fused Hamilton projection are now stubs that refuse to run.
- [`examples/quaternion-snap.rs`](../../crates/uor-r4-training/examples/quaternion-snap.rs): post-hoc snap probe.
- [`stack_snap_parity.rs`](../../crates/uor-r4-training/src/stack_snap_parity.rs): parity report of the exported multiplier-free artifact against two float comparators.
- [`learner/group_table.rs`](../../crates/uor-r4-core/src/native_geometric/learner/group_table.rs): exact icosian composition by table lookup.
- [`learner/hamilton_transport.rs`](../../crates/uor-r4-core/src/native_geometric/learner/hamilton_transport.rs): exact signed-permutation left actions on integer 4-vectors.
- Restorable dense source, preserved inactive: `archive/history/branch-archive/dense-quaternion-projection-inactive-20261012.patch`, indexed in `archive/history/branch-archive/INDEX.md`.
- Existing art: [`docs/figures/geometry/600-cell-icosians.svg`](../figures/geometry/600-cell-icosians.svg).

Audit source: `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/spin-weights.json`.

## Related mechanisms

[spiralcore-fbs](./spiralcore-fbs.md): replaced the dense quaternion sum with one selected, transported 8D block per output block. [hamilton-gate](./hamilton-gate.md): the same Hamilton product used as a gate on 4-blocks. [recurrence-gates](./recurrence-gates.md): the quaternion scan carries state by the same product, and identity transport cost 0.07 nats. [ternary-xor](./ternary-xor.md): the competing multiplier-free weight alphabet. [discrete-learning](./discrete-learning.md): the active model uses 8D signed-axis transport, not quaternion maps. [e8-orbit-memory](./e8-orbit-memory.md): exact E8 addresses, the other half of the spin-native design. [hopf-observation](./hopf-observation.md): quaternions reach the 3-sphere to 2-sphere shadow.
