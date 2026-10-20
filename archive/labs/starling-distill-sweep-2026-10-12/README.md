# Starling distillation weight and start sweep: a heavier KL raises the hit rate but breaks the readout; a lighter KL gives the best starling recall so far, 0.380 (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Murmuration-read ladder, step 1b.
**Line:** starling read · count 5 (owner-directed continuation) · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6116308030](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116308030), with the cross-arm precedence amendment.
**Test fitness:** round 1 found one blocker (the cross-arm decision was undefined); round 2 found one wording blocker; both were fixed before compute.

## Question

[SDw](../starling-distill-2026-10-12/README.md) lifted the starling hit rate to 0.61–0.68, short of 0.70 on every seed. Do the KL weight or its start point move it further?

## Runs

- **Arms:** each is SDw with one change, seeds 1–3. Context 512, 1,800 steps, `starling:4:7:content:shared:ranked`, surrogate on, `starling_anneal=0.5`.
  - **W3:** KL weight 3.0.
  - **W03:** KL weight 0.3.
  - **S33:** KL from step 594, so the teacher-read helper runs over steps 594–899.
- **Code:** binary built at `dca29a80b`. Existing bench options only.
- **Hardware:** one RTX 4090 (pod aoq49jic2vhg1a). 9 runs of about 100 s each, all on the GPU (`device=cuda`); about $0.35 with the build.
- **Outputs:** `claude-sweep-20261012` in the cloud store (md5 fd686ee2…).

## Result

| Arm | Hit rate, step 1800 (curve) | Mean | Recall per seed | Mean | Recall with the source forced in |
|---|---|---:|---|---:|---|
| SDw (base) | 0.61, 0.68, 0.66 | 0.65 | 0.079, 0.401, 0.085 | 0.188 | 0.12, 0.62, 0.13 |
| **W3** | 0.97, 0.68, 0.64 | **0.765** | 0.022, 0.350, 0.012 | 0.128 | 0.02, 0.48, 0.01 |
| **W03** | 0.60, 0.66, 0.77 | 0.678 | **0.141, 0.561, 0.439** | **0.380** | 0.22, 0.79, 0.55 |
| S33 | 0.21, 0.71, 0.03 | 0.316 | 0.010, 0.608, 0.005 | 0.208 | 0.01, 0.89, 0.01 |
| L, the L2 softmax read (rung 1) | — | — | 0.669, 0.373, 0.809 | 0.617 | — |

**Decision: row 3, best arm W3, by the pre-registered bars.**
- W3's mean hit rate of 0.765 is above SDw + 0.05 = 0.70, but it is not ≥ 0.70 on every seed (0.68 and 0.64).
- No arm reaches row 1 or 2.
- W03's mean hit rate of 0.678 is below 0.70, so it does not reach row 3 either.

**What the data adds beyond the bars (reported, not used to re-score):**
- **The hit rate stopped tracking recall.** W3 keeps the source on 97% of queries on seed 1, yet recall is 0.022. Even with the source forced into every kept set, recall is only 0.02. A heavy KL pushes the addresses toward the teacher's map and breaks the readout: the source is ranked first when kept on 0.39–0.63 of queries, against SDw's 0.50–0.68.
- **A light KL gives the best starling recall so far.** W03's recall is 0.141, 0.561 and 0.439 (mean 0.380, two seeds above 0.36). That is 62% of the L2 softmax read's 0.617, from a read that serves without a softmax. Every seed beats SDw. Recall falls with distance (seed 2: d16 0.71, d400 0.27).
- **The teacher-read helper with the KL on collapses selection.** S33 collapses on 2 of 3 seeds, with a hit rate of 0.000 at step 900 on seed 3. The remaining seed reaches 0.608. This agrees with SD: from a teacher that has not yet trained, the KL is destructive.
- **Noise:** seed-to-seed spread is large in every arm; recall SD is 0.2–0.33. Any decisive run here needs either more seeds or a mechanism that removes the per-seed instability.

**Correction to the [SDw record](../starling-distill-2026-10-12/README.md):** SDw never ran the teacher-read helper, because the helper weight is exactly 0 at step 900. Its recall gain over SGS comes from the KL plus the step-900 switch, not from a change of helper.

## Next

The row-3 next step is "that lever further". The data say the lever is not "more weight": the hit rate rose under W3 while recall fell. The next pre-registration therefore scores **recall** directly. Its base is W03 (0.380), with the L2 read (0.617) as the target, and it keeps the hit rate as a diagnostic. It takes the weight ≤ 0.3 direction with more training steps, and aims to remove the per-seed instability. Ladder step 2 (distance and context sweep) follows a recall result; d400 recall is already the weakest bucket.
