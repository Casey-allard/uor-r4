# Starling readout self-distillation with the tree: one seed reaches softmax-level recall at about 69 candidates per query, two never learn the task (row 4) (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Murmuration-read ladder, step 3b.
**Line:** starling read · count 1/3 · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6117824256](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6117824256), with the amendment. **Review:** CODE OK; fitness NOT FIT on three items, all fixed before compute. In particular, the auxiliary term trains the readout only, never q·k.

## Question

With the tree ([#2240](https://github.com/UOR-Foundation/uor-r4/pull/2240)), sources are retained, but recall at 1536 was 0.329. The code suggested the shared readout dilutes the source: slots past rank K share one table entry. Does training the served readout weights (rank table × gate) toward the softmax of the content score over the kept slots lift recall?

## Change (training only, never saved or served)

- **`set_starling_rank_distill(weight)`:** in the distillation's training forwards, add weight × mean Σ_slots (w − p)².
  - w is the readout weight, with its gate recomputed from the **detached** content score.
  - p is the softmax of the content score q·k/√d over the live slots.
  - The forward value is unchanged; the served read stays softmax-free.
- **Measured:** the readout's effective slot count, the participation ratio of the weights, averaged over the read layer's positions. This is descriptive only.
- **`mqar-bench starling_rank_distill=W`.**
- **Test:** logits are unchanged; the term is ≥ the KL alone; its gradient reaches the rank table, β and γ; the measurement lies in [1, slots]; an address route refuses it.

## Runs

- **Arm WTR:** WT + `starling_rank_distill=1.0`. Context 1536, batch 8, 3,600 steps, seeds 1–3.
- **Hardware:** one RTX 4090 (pod b5slecujwwaoef), binary `56500ce93`, all on the GPU. About 28 min.
- **Exact-head checks** on the pod's CPU: starling lib tests 16/16, mqar-bench tests 18/18.
- **Completion:** every report shows 3,600 steps with no early stop.
- **Outputs:** `claude-s3b-20261012` in the cloud store (md5 925b7843…).

## Result

| Seed | WT recall | **WTR recall** | WTR forced-in | WTR hit rate | WTR effective slots | WTR candidates per query, mean (max) | WTR train NLL, step 3600 |
|---|---:|---:|---:|---:|---:|---|---:|
| 1 | 0.031 | 0.063 | 0.063 | 1.00 | 1.64 | 19.0 (189) | 3.87 |
| 2 | 0.615 | **0.964** | 0.970 | 0.99 | 2.62 | 68.9 (228) | 0.37 |
| 3 | 0.340 | 0.104 | 0.131 | 0.68 | 2.36 | 16.5 (171) | 3.97 |
| **Mean** | 0.329 | **0.377** | 0.388 | | | | |

The L2 softmax read at the same settings ([step 2](../starling-context-2026-10-12/README.md)) reaches 0.999 (seed 2: 0.997).

**Decision: row 4.**
- Mean recall of 0.377 misses row 2 (≥ 0.63).
- Forced-in recall did not rise against WT's (0.413 → 0.388), so row 3 is not met either.
- The readout term does not move mean recall.

**What the seeds show (reported, not used to re-score):**
- **When the model learns the task, it matches softmax.**
  - Seed 2 reaches recall **0.964**, flat across distance (d1000 **0.967**), with held-out class 0.955. Softmax L on the same seed reaches 0.997.
  - It reads **68.9 candidates per query on average, at most 228**, against softmax's 1536, with no softmax at runtime.
  - The same seed without the readout term (WT) reached 0.615.
- **When it does not, nothing downstream helps.**
  - Seeds 1 and 3 end with train NLL 3.87 and 3.97 (chance is about 5.3).
  - Recall with the true source forced into the kept set is 0.06–0.13.
  - On seed 1 the source is retained 100% of the time, ranked first 75%, and the readout concentrates on 1.6 slots. The model reads the right position, but the value there never learned to produce the answer.
- **Dilution is not the general cause.** Effective slot counts are 1.6–2.6 on all seeds, so the readout concentrates whether or not the seed learns.
- **The limit is learning reliability.**
  - Both failing seeds are still near chance at step 1800, when the dense helper has annealed to zero (train NLL about 5.0).
  - Seed 2 had already started learning by then (NLL 4.1, accuracy 0.15).
  - After 1800, the sparse read's gradient alone does not rescue them.
  - The same pattern held in step 3: of WT's seeds, the one that had begun learning by step 1800 did best.

## Next

The pre-registered row-4 next step was readout capacity. The seed evidence points instead at **learning dynamics before the helper anneals**:
- keep the dense helper on until the model has learned (a longer anneal, or one gated on train NLL);
- or start the distillation and readout terms earlier.

Seed 2 shows the mechanism itself reaches softmax-level recall at about 4.5% of the reads. The next pre-registration targets the helper schedule, scored on recall with 3 seeds and a base of WTR.
