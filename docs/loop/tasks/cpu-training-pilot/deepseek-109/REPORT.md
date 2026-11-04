# REPORT: cpu-training-pilot, deepseek-109 (expanded memory-weight panel)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Prep:
[expanded memory panel](../../../../harness/prep/cpu-training-pilot-memory-expanded-2026-11-03.md).
Base: `9bd3c5773`.

## What did not work, first

- **The first FIT review rejected four contract errors.** It required an
  admission metric the reading does not contain, changed the historical
  three-sigma formula, retained only selection counts and omitted the
  established cost stop. The final sheet uses the existing formula, records
  all 16 keys and membership counts, records source/binary/host identity and
  stages seed 2 behind a 30-minute total stop.
- **The registered ranking signal still fails on both seeds.** Trained top-1
  gains over the stronger controls were 19 rows on seed 1 and 43 on seed 2,
  below thresholds of 64 and 60.
- **Repeated keys are not generalization.** Training and evaluation share the
  16 opening keys. The document split prevents table reuse, but this remains
  internal development evidence.
- **The seed-2 random control falls below frozen.** Random scored 431 versus
  frozen 441 top-1. The trained arm still clears random by 53 rows, but the
  control perturbation is not neutral.

## What changed

No model source changed. The existing exact cached memory-weight trainer ran a
larger fixed selection:

- 16 keys instead of 8;
- 102 training and 114 evaluation examples instead of 60 and 56;
- 32 accepted movement-coordinate updates instead of 8;
- 32 Rayon workers per seed;
- seed 1 followed by seed 2 under the registered cost stop.

The controls remain frozen, equal-accepted-move random and memory-off. Every
arm is checked through the exact CPU/serving full path.

## Measurements

Source code was `9bd3c5773`; release binary SHA-256 was
`c3bc4164efd0c02b74455381ee8f5e37eb98cea52c35b18015a755725f9e6bae`.
Host was the Ryzen 9 9950X, Linux 7.0.0-42-generic, Rust 1.97.1.

| Seed | Top-1 frozen / random / trained | Distance frozen / random / trained | Gain vs controls | Threshold | Accepted |
|---|---|---:|---:|---:|---:|
| 1 | `507 / 507 / 526` | `4,912,575,151 / 4,911,362,319 / 4,896,257,709` | `+19 / +19` | 64 | 32/32 |
| 2 | `441 / 431 / 484` | `5,075,557,655 / 5,072,947,107 / 5,054,667,878` | `+43 / +53` | 60 | 32/32 |

Training objective movement:

| Seed | Training top-1 | Training distance |
|---|---:|---:|
| 1 | `419 → 450` | `4,785,844,128 → 4,757,344,232` |
| 2 | `359 → 404` | `4,985,524,617 → 4,954,724,242` |

Both seeds completed in about 36.1 s with peak RSS 402,636-408,256 KiB. The
projected total was far below the 30-minute stop.

Every arm checked 114 samples, 4,333 positions, 1,109,248 Q12 coordinates,
4,333 token decisions and 34,664 kept cells with zero query, token or kept-cell
mismatches. Frozen and memory-off readings were identical. The trained
artifact with memory disabled reproduced the same frozen/memory-off readings.

The full record is [comparison.json](evidence/comparison.json).

## Reading

The dense training objective and held-out ranking both moved in the right
direction on both seeds. The expanded panel produced larger raw top-1 gains
than deepseek-95's +13/+5 (+19/+43 here), but the key set, example count and
step count all changed, so this is not a matched growth comparison. The
registered three-sigma threshold still failed. This is a positive mechanism
result and a negative ranking-signal result for the tested budget. It does not
justify a capability claim or a larger run without a different hypothesis.

## Checks run at the head

- Selection reproduced: 16 keys, 102 training, 114 evaluation.
- FIT: fresh-context non-author review before public compute.
- Both public seeds completed with exact five-arm parity.
- Full CPU release tests: 163 library tests passed with 4 release benchmarks
  ignored; every binary suite passed.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- Formatting, diff whitespace, FIT schema and evidence checksum: passed.
- Exact-head review, protected merge and fresh-main verification are run after
  this report head is frozen.

## Skills used

No available Codex skill applied.

## Still open

- The ranking gain misses the registered threshold.
- Unseen-key transfer, complete replies, fresh phrasing and useful chat remain
  unproven.
- A materially different credit assignment or larger external panel is needed
  before extending this configuration.

## Plain summary

The earlier memory-weight run improved held-out next-token correctness by 13
and 5 rows. This run made the panel substantially larger and trained 32
coordinates. The gains became 19 and 43 rows, and target distance fell on both
seeds, but the required statistical signal was 64 and 60 rows.

So the geometric memory weights are doing something repeatable, but not enough
to call the current configuration useful. The next step should change the
training hypothesis or panel, rather than simply spending more CPU steps on
this one.
