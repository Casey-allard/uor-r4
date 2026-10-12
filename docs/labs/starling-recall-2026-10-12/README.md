# Starling read, light distillation trained twice as long: recall 0.780, stable on 5 of 5 seeds (KEEP, provisional) (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Murmuration-read ladder, step 1c.
**Line:** starling read · **KEEP**: the mechanism target is met, so the count resets (D22) · headline "softmax at runtime: yes" → yes. The served model is unchanged.
**Pre-registration:** [6116521393](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116521393), with its amendment. **Test fitness:** FIT, after one routing fix.

## Question

Step 1b ([record](../starling-distill-sweep-2026-10-12/README.md)) found that the light-KL arm W03 had the best starling recall, but it was unstable across seeds. Scored on recall directly, with 5 seeds per arm: does W03 need more training (W03L), or a lighter KL (W01)?

## Runs

- **Arms:** each is W03 with one change. The route is `starling:4:7:content:shared:ranked`, with the training-only softmax surrogate, the address helper annealed to zero at half-way, and the selector KL at weight 0.3 from half-way. Context 512.
  - **W03L:** `steps=3600` (the helper phase and the KL phase both double).
  - **W01:** KL weight 0.1.
  - **W03:** seeds 4–5 added to the existing 1–3, for a 5-seed base.
- **Code:** binary built at `6e64a8734`. The bench and model code are byte-identical to the step-1b binary.
- **Hardware:** one RTX 4090 (pod oanuajbo433oh3). 12 runs, all on the GPU (`device=cuda`); about 25 min, about $0.45.
- **Completion:** every report shows `steps_completed` equal to its target and `stopped_early_at_max_seconds: false`.
- **Outputs:** `claude-s1c-20261012` in the cloud store (md5 753ae6c7…).

## Result (recall = in-class fresh accuracy, 64 sequences per seed)

| Arm | Recall, seeds 1–5 | Mean | Seeds ≥ 0.30 | Row |
|---|---|---:|---:|---|
| W03, base (1,800 steps) | 0.141, 0.561, 0.439, 0.247, 0.015 | 0.281 | 2/5 | — |
| W01 (1,800 steps) | 0.188, 0.910, 0.731, 0.360, 0.009 | 0.440 | 3/5 | 2 |
| **W03L (3,600 steps)** | **0.938, 0.605, 0.950, 0.843, 0.566** | **0.780** | **5/5** | **1** |

**Decision: row 1, KEEP W03L.**
- The bar was max(0.50, W03 + 0.13 = 0.41) = 0.50, with ≥ 4/5 seeds ≥ 0.30. W03L's mean is 0.780, and all 5 seeds are ≥ 0.30.
- W03L beats the paired W03 seed on every seed: +0.80, +0.04, +0.51, +0.60, +0.55.
- **Provisional, per the amendment:** with 5 seeds the bar is about one SE, so ladder step 2 re-runs W03L at context 512 among its conditions.

**More detail on W03L:**
- **By distance (in-class):** d16 0.85–0.99, d64 0.69–0.99, d200 0.48–0.96, d400 0.24–0.93. The two weaker seeds lose most at d400.
- **Hit rate:** 0.60–1.00. **Source ranked first when kept:** 0.49–0.73.
- **Held-out pairing class:** 0.52–0.94, close to in-class, so the read generalises the pairing rule rather than memorising pairs.
- **Recall with the source forced in:** 0.74–0.99. On 4 of 5 seeds this ceiling is above 0.93, so the remaining loss is mostly selection at long distance.

**Against the L2 softmax read** (rung 1: 0.669, 0.373, 0.809, mean 0.617): W03L's mean is higher. **But the budgets are not equal.** L was trained for 1,800 steps and W03L for 3,600, so this is not a claim that the starling read beats softmax. Step 2 includes L at the same budget.

**W01** (a lighter KL at 1,800 steps) is better than W03 but unstable (row 2). Most of W03's shortfall was training length, not KL weight.

## What this establishes, scoped

On the synthetic multi-query associative recall bench (context 512, 4 heads, width 128), a read that serves **without a softmax** reaches recall 0.57–0.95 on 5 of 5 seeds. It selects about 7 neighbours by a 600-cell lookup and reads them through a rank table.
- **Training-only aids,** none saved or served: an annealed dense helper, a softmax surrogate gradient, and a dense-teacher KL on the 4-d addresses.
- **Not established:** language modelling, long context (> 512), equal-budget parity with softmax, and the served-kernel cost.

## Next

**Ladder step 2: distance and context sweep.**
- W03L and L at an equal budget, at contexts 512, 1536 and 4096, with M scaling.
- It tests both the provisional KEEP and the logarithmic-cost claim (the kept set stays about 7 while the context grows).
- Then step 3 (the FBS binder layer) and step 4 (language).
