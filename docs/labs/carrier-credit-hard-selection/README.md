# Carrier credit by hard selection: eight arms on the live discrete learner (#2819)

- **Lab:** claude, unit claude-103, PR #3203.
- **Item:** [`carrier-credit-hard-selection`](../../roadmap/items/carrier-credit-hard-selection.md). **Mechanism:** [carrier credit](../../mechanisms/carrier-credit.md).
- **Records:**
  - decision card [`DECISION.md`](../../loop/tasks/carrier-credit-hard-selection/claude-103/DECISION.md), D68, approved in round 4;
  - prep sheet [`carrier-credit-hard-selection-2026-10-25.md`](../../harness/prep/carrier-credit-hard-selection-2026-10-25.md), FIT in round 2 (#2819 `6395832768`);
  - D67 pre-run review (#2819 `6395854890`);
  - release [`release.json`](release.json), sha256 `695f4205…` over the file; each arm record's `release_sha256` is `3296f7a1…`, the sha256 of the release's compact JSON, which is what `run_arm` hashes;
  - plan [`plan.json`](plan.json);
  - fresh-slice manifest [`fresh-slice-manifest.json`](fresh-slice-manifest.json);
  - base config [`base-config.json`](base-config.json), the A8 dose config byte for byte.
- **Outputs:** in the HF store under `results/claude/carrier-credit-2026-10-25/`, in `podA/`, `podB/` and `podC/`, each with a SHA256SUMS. The fresh slice is in `fresh/`.

## What ran

| Schedule | Vector block tried on 7 of every 8 batches |
|---|---|
| ON `carrier_credit_v1` | the token with the largest summed integer hinge (target, plus the strongest rival when top-1 is wrong), on CUDA |
| CONTROL-U `carrier_uniform_target_v1` | a SplitMix64 draw over the batch's distinct targets |
| CONTROL-F `carrier_uniform_row_v1` | a SplitMix64 draw over the batch's rows |
| OFF `interleaved_read_every8_v1` | the fixed sweep, as in #2831 |

Every arm uses V 4,096, B 4, C 192, K 8, the Relation read, `rows_v1`, 64 rows per batch, a seeded-device codebook, seeds 1001 and 2001, and 16,404 batches. FRESH is 65,536 targets from TRAIN documents past the dose's last visited target, with 3,237 exact repeats dropped. DEV is `ft/dev`.

## Layout and timing

**Probes.**
- Run 1: four arms at once per GPU, on `vast-57642427` (2×5090). It projected 203 minutes per arm (86 targets/s) and stopped at the 90-minute rule.
- Solo probes: ON 318, controls 326, OFF 521–532 targets/s at batches 0–164.

**Full doses.** Per-batch cost grew as each arm trained:
- Over the full dose, ON averaged 62–70 batches/min (66 and 75 targets/s in the records) and OFF 112–116 (119 and 124 targets/s), with GPU memory flat and both GPUs at 100%.
- The A8 dose, from a pinned codebook, held 0.12 s per batch. The seeded-device start is the likeliest cause of the growth here; no run has tested it.

**Pods.** The arms were spread across three pods (owner log #2819 `6396426933`):
- `vast-57642427`: ON, both seeds;
- `vast-57648161` (2×5090): OFF, then CONTROL-U;
- `vast-57648503` (1×5090): CONTROL-F, both seeds in turn.

A second 2×5090 was refused by the $8/h cap guard.

**Launch faults, kept as records:**
- The faulted folders `seed-1001`/`seed-2001` are run 1, stopped by the probe.
- `seed-*-podB` and `seed-*-podC` are an empty first launch. `uor-pod run` dropped the quoting of a `bash -c` command, and a cache-hit bootstrap left no source tree.

## Results

Pending. The control arms finish about 18:30Z. A docs-only follow-up unit grades all eight with `carrier-arms grade` and records the result here and in the notebook.
