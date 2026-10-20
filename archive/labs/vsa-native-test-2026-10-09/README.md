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

## Result 2: mode 2 (root codes + per-token readout residual), laptop CPU

Pre-registered on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) before the runs. KEEP required mode 2 to beat fixed codes on both slices for both seeds, with its own VSA ablation Δ above 0.

- **What ran:** `train-native-prose` / `ablate-prose` at main `c7fce45b1` on the owner's laptop (8 threads, `--samples 0`). Same corpus, window (offset 200,000,000, 6,795,016 tokens), batch, sequence length, lanes, seeds, slices and ablations as Result 1. Arm 2 (fixed codes) was rerun on the laptop for a same-machine control. Arm 5 trained with `--vsa-codes readout` and was scored with `--vsa-code-mode learned` (4,096 distinct codes).
- **Reproducibility:** the laptop rerun of arm 2 matches the pod run of Result 1 to every printed digit (1.8366 / 1.8214 and 1.8582 / 1.8427), so laptop and pod results here are directly comparable.
- **Cost:** about 50 minutes of laptop CPU: training 54–91 s per run, scoring 6–8 minutes per slice. No pod.

| Arm | Seed | Slice A | Slice B | VSA ablation Δ (95% CI), A / B | Engram ablation Δ, A / B |
| --- | --- | --- | --- | --- | --- |
| 2: fixed codes | 20260917 | **1.8366** | **1.8214** | +0.0133 [0.0128, 0.0139] / +0.0151 [0.0145, 0.0157] | +0.1943 / +0.1851 |
| 5: root + readout residual | 20260917 | 1.8629 | 1.8491 | −0.0050 [−0.0062, −0.0038] / −0.0043 [−0.0056, −0.0031] | +0.1884 / +0.1796 |
| 2: fixed codes | 20261009 | **1.8582** | **1.8427** | +0.0139 [0.0134, 0.0145] / +0.0157 [0.0151, 0.0163] | +0.1948 / +0.1857 |
| 5: root + readout residual | 20261009 | 1.8707 | 1.8566 | −0.0029 [−0.0040, −0.0017] / −0.0021 [−0.0033, −0.0009] | +0.1941 / +0.1852 |

**Decision: not KEEP.** Mode 2 is worse than fixed codes in all four cells, by +0.026 / +0.028 (seed 20260917) and +0.013 / +0.014 (seed 20261009). Its own VSA term is slightly harmful: removing it lowers BPB by 0.002–0.005, with every CI below 0. Fixed codes stay the default, and the mode-2 code stays in place.

**Reading (hypothesis, not tested):** in this mode the codebook is rebuilt from the readout every 1,000 steps while training runs (`--vsa-code-refresh`, default 1000, which these runs used), so the heads may chase moving codes. The fixed codebook gives each token a code that never changes. A test would freeze the mode-2 codebook after a warm-up and train on.

## Next
- **Mode 2: done, not KEEP** (Result 2 above). The freeze-after-warm-up test is the one follow-up the result justifies; it is not scheduled.
- Fixed-code VSA stays the default for the native learner. The M4 softmax-free reads come next.
