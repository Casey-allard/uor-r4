# Starling read with a persistent base-3 tree (HoloBox/FBS): retention fixed, recall flat across distance at 6–36 candidates per query; readout now limits (row 3) (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Murmuration-read ladder, step 3.
**Line:** starling read · the tree's own mechanism target (hit rate ≥ 0.85 on ≥ 2 seeds) is met, so under D22 the tree is **kept as a component** and the count resets; recall is not met · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6117320792](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6117320792), with the amendment after review. **Review:** CODE OK; fitness NOT FIT on four items, all fixed before compute. **Sources:** the HoloBox persistent path-copy runtime and the SpiralCore FBS base-3 carry ([W33 overview](../../research/w33-overview-2026-10-12.md)).

## Question

[Step 2](../starling-context-2026-10-12/README.md) found that the starling read fails at 1536 because each 600-cell vertex keeps only its last M = 4 keys. Does a persistent base-3 tree per vertex, retaining by learned salience, keep old sources readable at logarithmic cost?

## Change (`read_route=starling:…:tree`)

- **Per vertex:** the recent ring stays. Each key evicted from it joins a pending list, and every M evictions form a level-0 block.
- **Levels:** a level holds at most 2 blocks. A third block merges all three: the **M most salient** keys survive (ties go to the later position) and carry up one level. This is a base-3 odometer.
- **Read:** a query reads every stored key of its home vertex and its 12 neighbours, then takes top-K by address dot and the rank-table readout as before. No softmax is added at runtime.
- **Salience** (`read.starling.salience`, `[heads, width]`) is a linear map of the detached stream, used only through the sort.
  - Its training-only regression targets the log of the dense teacher's mean attention into j per later query, centred per row.
  - It is active with the distillation KL, from step 1800.
- **Cost:** a counter of candidates per query is measured, across threads, on the final panel.
- **Tests:**
  - tree selection equals an independently written batch reference (merges exercised, noisy keys);
  - the tree equals the ring without evictions;
  - logits are unchanged by salience without merges;
  - the most salient key is never dropped;
  - the tie rule is pinned, with a negative gate: removing the tie-break fails the test;
  - the salience gradient reaches its map;
  - the route name round-trips through the parser.

## Runs

- **Arm WT:** the W03L configuration plus `:tree` and `starling_salience=1.0`. Context 1536, batch 8, 3,600 steps, seeds 1–3.
- **Hardware:** one RTX 4090 (pod 4jcq70bvxx4lq1), binary `f57af1c2b`, all on the GPU. About 31 min.
- **Exact-head checks** on the pod's CPU: starling lib tests 15/15, mqar-bench tests 18/18, and rustfmt clean on this PR's hunks.
- **Completion:** every report shows 3,600 steps with no early stop.
- **Outputs:** `claude-s3-20261012` in the cloud store (md5 eb050b33…).
- **Bases:** step 2 at the same settings.

## Result

| Arm | Recall, seeds 1–3 | Mean | Hit rate, step 3600 | d1000 recall | Recall with the source forced in | Candidates per query, mean (max) |
|---|---|---:|---|---|---|---|
| L, softmax | 1.000, 0.997, 1.000 | 0.999 | — | 1.00 | — | 1536 (every earlier position) |
| W03L, ring only | 0.185, 0.529, 0.009 | 0.241 | 0.56, 0.61, 0.00 | 0.064, 0.137, 0.006 | 0.31, 0.83, 0.01 | ≤ 52 |
| **WT, tree** | 0.031, 0.615, 0.340 | **0.329** | **0.99, 0.68, 0.90** | **0.035, 0.584, 0.316** | 0.03, 0.84, 0.37 | **15.5 (190), 36.1 (182), 5.6 (171)** |

**Decision: row 3.**
- The hit rate is ≥ 0.85 on 2 of 3 seeds, so retention is fixed.
- Recall (0.329) is below the 0.54 bar.
- Next, per the pre-registration: the readout, plus seed stability.

**What the tree did:**
- **Recall no longer decays with distance.** Within each seed, d1000 recall is close to d16 (seed 2: 0.58 against 0.68; seed 3: 0.32 against 0.37). Without the tree, d1000 was 4–20× below d16.
- **The read stays small:** 6–36 candidates per query on average, at most 190, against L's 1536.
- **Sources survive:** the hit rate reached 0.99 on seed 1.

**What it did not do:** turn retained sources into answers.
- **Seed 1:** hit rate 0.99 and the source ranked first in 67% of kept sets, yet recall is 0.03, both naturally and with the source forced in. The readout is the failure point: the rank table and value path do not map the source to the answer.
- **Seed 3:** forced-in recall is 0.37.
- **Seed 2:** the best seed (0.615), but its hit rate dropped from 0.80 at step 2700 to 0.68 at the end, with the gradient norm reaching 12. Late training is unstable.
- **The gap to softmax** at equal budget is still large: 0.329 against 0.999.

## Next

**The readout step at 1536, with the tree on.**
- The data say the remaining loss is between "source kept" and "answer produced".
- Candidates for that step: distil the rank-table weights from the teacher's attention over the kept set, and stabilise the late phase (the grad-norm spikes).
- The persistent tree is kept as the retention layer: it removes distance decay at logarithmic read cost.
