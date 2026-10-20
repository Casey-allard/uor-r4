# Native VSA retraining test: plan (pre-registered 9 October 2026)

Owner-approved test of whether the VSA term helps the native prose learner once
it is trained into the model, rather than swapped into a frozen artifact. The
review is in [vsa-review-2026-10-09](../vsa-review-2026-10-09/README.md), and the
pre-registration is on M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).
Results will be recorded in `README.md` beside this plan.

## Learner and data
- **Learner:** `train-native-prose` (the Card P3 native prose learner, which produces
  the `native_geometric_prose_model.rgm` scorer used by the P7 measurements).
- **Corpus:** `tinystories_train.u16`, 555,385,505 tokens, byte-BPE vocabulary 4,096.
  The private HF store holds it at `data/tinystories_train.u16`.
- **Training window:** offset 200,000,000, with an equal token budget for every arm.
  This keeps both held-out slices out of training.
- **Held-out slices (unchanged from the P7 ablation):** slice A at offset 0 and slice B at
  offset 100,000,000. Each arm is scored on 65,536 positions per slice with
  `ablate-prose --model <arm>.rgm --holdout-offset <0|100000000> --positions 65536`.

## Arms (2 seeds each: 2026_09_17 and 2026_10_09)

| Arm | Flags | Meaning |
|---|---|---|
| 1 | `--no-vsa` | VSA term off: scale held at 0, no update, exports 0 |
| 2 | `--vsa-codes fixed` | Fixed token-id hash codes; the scale is trained; serving uses the same fixed codes |
| 3 | `--vsa-codes root` | Codes derived from the learned icosian-root assignment, refreshed every 1,000 steps during training; serving uses the same codes |
| 4 | `--vsa-codes root --no-engram` | Arm 3 with the exact engram n-gram table removed. P7 has no Copy head; the engram table is its exact-lookup term |

All arms share `--offset 200000000 --tokens <budget> --lanes 4 --batch-size 256 --seq-len 64`.
The budget is fixed after a timed smoke run, so that 8 runs fit within about one RTX 5090-day.

## Metric and decision
- **Metric:** held-out bits per byte on each slice, from the exported, served artifact.
- **Interval:** paired bootstrap over positions, comparing each arm with arm 1 on the same
  slice and seed, 2,000 resamples, 95% interval.
- **KEEP** VSA if any arm improves held-out BPB by at least 0.01 against arm 1, with the
  interval excluding zero on both slices and both seeds.
- **Otherwise** VSA is not retired (owner, 9 October). The next step makes it more
  native: codes trained end to end (straight-through on the icosian root assignment),
  and the VSA term moved from a post-hoc scorer into the read or recurrence.
- **Also reported:** each arm's served tokens per second (the codebook is now built once
  per session) and its served `vsa_scale_q15`.
