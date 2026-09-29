# G-binding: masking the text tag/trigger losses recovers held-out role binding — result

September 29, 2026. References #973 and #820.
- **Lab:** Lab 2 (OpenCode); code in `stack_aerm.rs` / `aerm-probe.rs` (merged in #1489, `e0eefd37`).
- **Pre-registration:** [#973 comment 5883172773](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5883172773) (Lab 1), with the names-list constraint recorded at [#973 comment 5883489559](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5883489559) before the run.
- **Evidence:** [g-binding-2026-09-29.json](../evidence/g-binding-2026-09-29.json); receipt `opencode-g1-binding/binding.supervision.json`.

**Status.** Complete; both arms sealed with manifests and no errors. **The pre-registered primary rule selects "adopt masking"** (+0.809 held-out Updated over the baseline); **the text-NLL guard fails** and is reported, not hidden. Nothing is promoted by Lab 2.

## What was tested

The G-decomposition showed the held-out failure is role binding on the query's entity (77.7 %), caused by ordinary #1017 prose teaching held-out names as "other" (Emma 9,573 ×, Leo 7,717 ×, Molly 7,230 ×, Rose 2,801 × as other, never an entity). The pre-registration's two arms start from G v1's `aerm-s1` (no retrain of that baseline):

- **A — mask the tag and trigger auxiliary losses on ordinary-text batches** (weight 0); everything else unchanged, including the LM loss.
- **B — A plus name-diverse episodes** from the committed 83-name list (`crates/uor-r4-training/data/g-binding-names.txt`).

Both: seed 1, same stack/world/schedule/evaluation as G v1, 2 threads each, checkpoints saved.

## Result

| Arm | Held-out Updated | Held-out First | Recency trap | In-distribution Updated | Text NLL | Δ text vs baseline |
|---|---:|---:|---:|---:|---:|---:|
| **A: mask only** | **0.967** (264/273) | 141/144 | 24/24 | 1.000 | 2.4926 | **+0.1188** |
| B: mask + names | 0.645 (185/287) | 116/134 | 28/34 | 1.000 | 2.4337 | +0.0599 |
| Baseline `aerm-s1` | 0.158 (43/273) | 31/144 | 7/24 | 1.000 | 2.3738 | — |

Arm A, held out: Reasserted 31/31, Previous 164/195, PreviousAbsent 146/157, Absent 377/378; free-running 32/32. Fresh (in-distribution) classes are perfect in both arms.

## Decision (pre-registered rule)

The pre-registered rule was applied to the **aerm-probe's own held-out Updated metric** (the numbers below); Lab 1's `prime-route-eval` sieve-register evaluation of these checkpoints is a separate, still-pending result and is not claimed here.

1. **A − baseline = +0.809 ≥ 0.30 → the supervision artifact is a major cause; adopt masking in memory training, including the S2 integration.** This is the strongest held-out Updated measured on this world; 9 of 273 Updated queries fail.
2. **B − A = −0.322, so B does not beat A by 0.20.** Name diversity in the cast is **not indicated** at this scale and is recorded as a negative. B's world also has different denominators (287 Updated, 134 First), because extending the name pool also extends the `friend` relation's values and changes episode composition.
3. **Guards.** In-distribution ≥ 0.98 in every class: **PASS** (1.000 everywhere in both arms). Text NLL within 0.05 of the baseline: **FAIL for both** (A +0.119, B +0.060). Masking removes auxiliary text supervision and costs language NLL; whether that price is acceptable for adoption is Lab 1's decision, not this record's.

## Mechanism check

- **Tag accuracy rises from 0.974–0.977 to 0.996** (A) / 0.998 (B).
- **`Unavailable` (model-vs-gold store divergence) falls from 460 / 484 / 519 to 0** in Arm A; `read_events` rises to 7,460.
- Arm A's residual held-out failures are `QueryRelationTag` 42, `Other` 12, `QueryEntityTag` 1 (trace `Emission` 12, `NotSelected` 43) — the seam has moved from the entity slot to the relation slot and shrunk by an order of magnitude.
- Reading: removing the "these names are other" supervision lets the tag head bind by slot rather than by lexical identity, which is exactly the pre-registered hypothesis.

## Artifacts and consumer

- Checkpoints (loadable through the merged `AermModel::load`, #1482):
  - Arm A `opencode-g1-binding/checkpoints-a/aerm-s1`; Arm B `checkpoints-b/aerm-s1`.
- Consumer: **Lab 1** runs `prime-route-eval` on these checkpoints (the pre-registration assigns the evaluation), then the S2 + branch integration (pre-registered separately) now has `AermModel::from_stack` (PR #1491) to start from S2's weights.

## Scope and limits

- One seed per arm, one parent, one read policy, the synthetic relation world; the baseline is G v1's `aerm-s1` under identical conditions.
- The text-NLL cost is measured, not explained; the guard comparison uses the baseline's own `text.dev_nll`.
- Arm B's denominators differ from A's, so the A/B contrast is not denominator-matched; its negative is recorded at that scope.
- No serving, dialogue or general-language claim.

## Resources

- 2026-09-29T05:04:50Z → 06:23:52Z, **4,742 s wall**; 2 processes × 2 Rayon threads; training 4,679.8 s / 4,682.7 s; peak sampled RSS **2.80 / 2.78 GiB** (3 GiB per-process guard); checkpoints 5.5 MB each; no paid compute. The slot was claimed after resolving a stale S4 claim (its PID was not running and no model job was active), noted on #973 before claiming.
