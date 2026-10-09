# Native VSA retraining test: result (9 October 2026)

**Decision: KEEP.** Once the VSA term is trained into the native prose learner, it improves
held-out bits per byte by at least 0.014 on both slices and both seeds, with fixed token-hash
codes. Icosian-root codes did not help. Next: root codes bound with a per-token residual
(mode 2), which keeps both geometric structure and token identity.

Plan and pre-registration: [PLAN.md](PLAN.md); M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).
Review that motivated it: [vsa-review-2026-10-09](../vsa-review-2026-10-09/README.md). Code: #2077.

## What ran
- **Learner:** `train-native-prose` at origin/main `86134d98`. Corpus `tinystories_train.u16` (md5 `87dc182a…`).
- **Training window:** offset 200,000,000, 6,795,016 tokens per run (budget set by a timed smoke run).
  Batch 256, sequence length 64, 4 lanes.
- **Seeds:** 20260917 and 20261009.
- **Scoring:** each run's exported, served artifact was scored with `ablate-prose` on held-out slice A
  (offset 0) and slice B (offset 100,000,000), 64,575 positions each. Each arm was scored with its own
  `--vsa-code-mode`, plus within-model ablations of the VSA term and the engram table.
- **Compute:** one 2× RTX PRO 6000 pod for about 2.3 h (about $11). The learner used about one core
  per process and no GPU. That job should have run on the laptop (owner rule, #2095).

## Results (held-out BPB; lower is better)

| Arm | Seed | Slice A | Slice B | VSA ablation Δ (95% CI), A / B | Engram ablation Δ, A / B |
| --- | --- | --- | --- | --- | --- |
| 1: VSA off | 20260917 | 1.8574 | 1.8440 | 0 / 0 | +0.2015 / +0.1923 |
| 1: VSA off | 20261009 | 1.8726 | 1.8590 | 0 / 0 | +0.2006 / +0.1913 |
| 2: fixed codes | 20260917 | **1.8366** | **1.8214** | +0.0133 [0.0128, 0.0139] / +0.0151 [0.0145, 0.0157] | +0.1943 / +0.1851 |
| 2: fixed codes | 20261009 | **1.8582** | **1.8427** | +0.0139 [0.0134, 0.0145] / +0.0157 [0.0151, 0.0163] | +0.1948 / +0.1857 |
| 3: root codes | 20260917 | 1.8577 | 1.8441 | +0.0025 / +0.0025 | +0.2006 / +0.1916 |
| 3: root codes | 20261009 | 1.8673 | 1.8536 | +0.0023 / +0.0023 | +0.1986 / +0.1893 |
| 4: root, engram off | 20260917 | 2.0583 | 2.0357 | +0.0059 / +0.0058 | 0 / 0 |
| 4: root, engram off | 20261009 | 2.0660 | 2.0429 | +0.0050 / +0.0049 | 0 / 0 |

**Arm 2 − arm 1, same seed and slice:**

| Seed | Slice A | Slice B |
| --- | --- | --- |
| 20260917 | −0.0208 | −0.0226 |
| 20261009 | −0.0144 | −0.0163 |

All four clear the 0.01 threshold. **Arm 3 − arm 1:** +0.0003 and +0.0001 on seed 1, −0.0053 and −0.0054 on seed 2.

## Reading
- **The VSA term is not inert when trained.** Arm 2 improves on VSA-off by 0.014–0.023 BPB in every
  slice and seed pair. Inside each trained model, the VSA term's own contribution (+0.013 to +0.016)
  has a 95% interval that excludes zero. DeepSeek's 8 October null measured codes swapped into a
  frozen artifact, which never learned to use them.
- **Icosian-root codes lost to the fixed hash.** In mode 1 every token assigned to the same icosian
  root gets an identical code: there are only 120 distinct codes for a 4,096-token vocabulary. The VSA
  term then cannot separate those tokens, so token identity matters more than root geometry here.
- **The engram table carries most of the exact memory.** Removing it costs about 0.19–0.20 BPB, and
  the VSA term does not compensate for it (arm 4).

## Limits
- **The interval differs from the pre-registered one.** The plan called for a paired bootstrap between
  arms, but the scoring logs do not keep per-position losses for cross-model pairing. The intervals
  shown are within-model ablations. Cross-arm differences are point estimates, consistent in sign and
  above threshold in all four slice and seed pairs.
- **Seed spread is about the size of the effect.** Arm 1 varies by about 0.015 BPB between seeds, so
  more seeds would tighten the estimate.
- **Scale.** These are small runs (6.8M training tokens per run) of one learner on TinyStories.

## Next
- **Mode 2:** root codes bound with a per-token readout residual, already built in #2077. It keeps the
  icosian structure and gives every token its own code.
- **Run order:** after the native learner is made truly multi-core, run mode 2 on the owner's laptop
  (CPU-only), with the same arms, slices and seeds plus per-position losses for the cross-arm interval.
