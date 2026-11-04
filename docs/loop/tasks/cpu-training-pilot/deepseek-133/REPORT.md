# REPORT: generated-prefix writer training, deepseek-133

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `75112a053`.

## What did not work

- **The generated-state eligibility gate stopped before training on both
  seeds.** Seed 1 found 30 eligible true-label decisions and 33 shuffled-label
  decisions; seed 2 found 32 and 32. The registered thresholds required 640
  eligible decisions per arm.
- **Same-cell control coverage was also far below the bar.** Seed 1 had 11
  true and 12 shuffled pairs; seed 2 had 12 and 11. The threshold was 64.
  Prompt and cell coverage reached 29 prompts / 11-12 cells, so the limiting
  quantity is admitted writer targets, not prompt diversity.
- **No writer update, held-out evaluation or seed-1 repeat ran.** This is an
  insufficiency result for the current public generated states, not a negative
  for generated-prefix writer training. The classifier cannot repair cells
  that writer admission never selects.

## What changed

Added a CPU-only `self_prefix` research module and driver:

1. it feeds the model's own greedy tokens through the complete CPU FBS,
   Hamming, opening-memory and reply-memory path;
2. it records the live Q12 query at each reply offset and aligns it with the
   reference content token at that same offset;
3. it builds fixed-horizon target maps independent of which rows a generated
   trajectory happened to reach;
4. it reuses the existing K=1 target/rival writer update and reparses every
   candidate artifact;
5. it supports reference, static-generated and dynamic-generated feedback plus
   same-cell shuffled targets; and
6. it fails closed on missing parity, vacuous controls, panel drift,
   non-determinism or a fixed-horizon regression caused by extra early BOS.

The public run stopped at the registered preflight before the six-arm training
comparison.

## Measurements

The fixed document-disjoint split reproduced **98 training** and **103
evaluation** samples on both seeds.

| Seed | Arm | Eligible | Prompts | Cells | Shuffle pairs |
|---:|---|---:|---:|---:|---:|
| 1 | true labels | 30 | 29 | 11 | 11 |
| 1 | same-cell shuffled | 33 | 29 | 12 | 12 |
| 2 | true labels | 32 | 25 | 12 | 11 |
| 2 | same-cell shuffled | 32 | 25 | 11 | 12 |

The fresh release build took **18 s**, peaked at **1,204,616 KiB** and occupied
**262,784,458 bytes**. The driver SHA-256 was
`332399b07045706829fae23660dd26e5082249d56775c752d5fdec5ab55718ae`.
The two 32-worker selection processes each took **1.00
s**; peak RSS was **221,740 / 202,712 KiB**. The retained pipeline is
**14,420 bytes**.

Selection evidence hashes:

- seed 1:
  `d8fcc11670f2d0b9aa3fd818c360cb2a015e7d485c926b137af0ee37448d60a2`;
- seed 2:
  `9e20643a7ffba77b676b6274ff5d103febe8f43ed7c529286fa51aae95d32484`;
- pipeline `SHA256SUMS`:
  `c037085bd8658a902570f875c60374bdc719c8da8f4e9320d9c7bb2eb9a50af8`.

No trained artifact, model-quality reading or parity aggregate exists because
no writer-training attempt ran.

## Checks run at the head

- FIT review attempts found and repaired missing target-map coverage, weak
  control handling, incomplete parity participation, missing reference
  continuation, and missing reply-memory evidence. Two fresh NOT_FIT reviews
  preceded FIT.
- Fresh-context TEST FITNESS: **FIT** at `efd9c9781`.
- Fresh-context pre-run reviews returned NOT_GO twice for an incomplete recipe;
  after adding exact commands and resource/repeat/checksum gates, the third
  returned **GO** at `e5e093af5`.
- Full `uor-r4-cpu-train` release suite: **223 library and 54 binary tests
  passed, 0 failed, 4 expected ignores**.
- Focused self-prefix tests: **11/11**; driver tests: **2/2**.
- Strict no-deps all-target CPU Clippy with `-D warnings`, package formatting,
  prep shape, `git diff --check` and `loop pr-check --local`: passed.
- All five public input hashes matched. The implementation tree matches pin
  `77524f3e47422a4c86578b2798bc98cd6d6d83b1`.
- Both fresh selection processes exited 0 and the retained recursive checksum
  verification passed.

## Skills used

No system-provided Codex skill applied. Read-only subagents performed the
mechanism/experiment audits, three FIT reviews and three pre-run reviews.

## Still open

- A generated-state admission or query mechanism must expose at least 640
  admitted writer targets on each public seed before writer ranking can be
  tested meaningfully.
- Complete replies, fresh phrasing, content correctness, integer-serving
  integration and GPU qualification remain open.
- Exact-head delivery review, protected merge and fresh-main verification.

## Plain summary

The generated-prefix training path was implemented and preregistered, but the
public models reached only 30-33 admitted repair rows per seed instead of the
required 640. Training stopped before any update. This is an insufficiency
result that redirects the next CPU move to generated-state cell admission,
rather than retuning writer rank or termination.
