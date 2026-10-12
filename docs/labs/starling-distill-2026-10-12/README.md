# Starling selector distillation from a dense content teacher: from step 0 it stops learning; started half-way it lifts the hit rate to 0.61–0.68 (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Murmuration-read ladder, step 1 (design [6115874382](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115874382)).
**Line:** starling read · count 4 (owner-directed continuation after 3/3; "do not shelve, refine") · headline "softmax at runtime: yes" → yes.
**Pre-registrations:**
- SD: [6115892682](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115892682), with the amendment [6115920234](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115920234) and [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115928460).
- SDh: diagnostic rows [6116074156](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116074156).
- SDw: [6116131206](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116131206). Its review was NOT FIT on one blocker, a test target that did not compile, which was fixed before compute.

## Question

In the best earlier arm, SGS ([record](../starling-surrogate-2026-10-12/README.md)), the true source is in a head's kept set only 0.39–0.45 of the time. Does a dense-teacher KL on the 4-d addresses that drive the 600-cell selection lift that hit rate? This is the DSA / SeerAttention recipe for training a sparse selector.

## Change (training only; never saved or served)

- **`set_starling_distill(weight)`** (`geometric_stack.rs`). Each starling read in a training forward adds weight × KL(teacher ‖ softmax(16·⟨a,b⟩)) to a thread-local auxiliary loss.
  - The teacher is softmax(q·k/√d) over earlier positions, detached.
  - With the dense helper on, the helper becomes that teacher read.
  - Evaluation forwards never build the teacher, because a training-forward flag gates it (this fixed an OOM).
- **`mqar-bench`:**
  - `starling_distill=WEIGHT`;
  - `starling_distill_start=FRACTION`, which keeps the KL off before that point;
  - the KL is logged on its own, and the reported train NLL stays language-only.
- **Bench test target.** It had not compiled on main, because its test literals lacked `select`, `attn_reads` and other fields. Both literals are now complete.
- **Runtime unchanged.** The served read has no softmax, so acceptance item 0 is untouched.

## Runs

- **Bench:** as in rungs 1–3 (context 512, 1,800 steps, `pattern=rarr key_shift=true`). The route is `starling:4:7:content:shared:ranked` with `starling_surrogate=true` and `starling_anneal=0.5`, seeds 1–3.
- **Hardware:** one RTX 4090 (pod h007uvngyqng5k); 9 runs of about 75 s each, roughly $0.45 including builds.
- **Checks:** exact-head checks ran on the pod's CPU:
  - starling lib tests: 11/11;
  - mqar-bench tests: 16/17. The failure is a frozen-parameter-count assertion in the transformer-control test, which never compiled on main and is filed separately.
- **Outputs:** `claude-sd2-20261012` (SD, SDh; md5 63412dc9…) and `claude-sdw-20261012` (md5 028e094b…) in the cloud store.

## Result

| Arm | KL | Recall per seed | Mean | Hit rate, step 1800 | Train NLL, step 1800 |
|---|---|---|---:|---|---|
| SGS ([#2222](https://github.com/UOR-Foundation/uor-r4/pull/2222)) | none | 0.044, 0.288, 0.091 | 0.141 | 0.39–0.45 | — |
| **SD** | weight 1.0 from step 0 | 0.006, 0.007, 0.006 | 0.006 | 0.03, 0.00, 0.10 | 5.30, 5.37, 5.34 |
| SDh (diagnostic) | weight 1e-6 (teacher-read helper only) | 0.018, 0.007, 0.109 | 0.045 | 0.42, 0.00, 0.38 (natural) | 4.80, 5.11, 4.34 |
| **SDw** | weight 1.0 from step 900 | **0.079, 0.401, 0.085** | **0.188** | **0.61, 0.68, 0.66** | 4.20, 3.24, 4.08 |

**SD: row 4a by the letter, but the arm did not train.**
- The KL fell from 2.3 to between 0.004 and 0.04, while the hit rate fell and recall stayed at chance.
- SDh's row D2 places the cause: NLL fell below 5.0 on 2 seeds once the KL was removed. So the KL from step 0 was the problem.
- Mechanism (inferred): the untrained teacher is near-uniform, and copying it collapses the addresses into a few cells, where per-cell eviction drops the source.

**SDw: row 3 (partial), by the pre-registered bars.**
- The mean hit rate of 0.65 clears the 0.55 bar, which is the binding one since SGS + 2·SD ≤ 0.49.
- It is not ≥ 0.70 on every seed.
- On every seed the hit rate **rose** after the KL switched on: from 0.53, 0.46 and 0.60 at step 900 to 0.61, 0.68 and 0.66. So a KL from a trained teacher moves the addresses the right way.

**Beyond the bars (reported, not used to re-score):**
- **Recall rises** to a mean of 0.188 (against 0.141). Seed 2 reaches 0.401, above the row-1 recall bar of 0.36 and two-thirds of the L2 read's 0.617. The recall gain is not attributable to the KL alone, because the step-900 switch also changes the helper.
- **Readout now limits two seeds.** With the source forced into every kept set, recall is only 0.12 and 0.13 on seeds 1 and 3, against 0.62 on seed 2. When kept, the source is ranked first 0.50–0.68 of the time.

## Next

Pre-registered next step for row 3: **sweep the distillation weight and temperature**, which targets ≥ 0.70 on every seed. The readout evidence above argues for including the rank-table readout in the same step (row 2's next). Ladder steps 2–4 (the distance and context sweep, the FBS binder layer, language) wait on step 1.
