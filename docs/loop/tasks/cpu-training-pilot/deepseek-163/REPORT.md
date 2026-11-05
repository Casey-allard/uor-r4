# REPORT: cpu-training-pilot, deepseek-163

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `7fe55fdbf`.

## Status

Decision: **REJECT** for the longest-32 exact-copy extension. The deeper arm
selected depth 32 on most decisions and beat every shallower or randomized
control except its nested longest-4 control, but it produced exactly the same
correct body positions as nested
longest-4: **409/4,661 train** and **300/4,758 fresh dev** on both artifact
seeds. The deeper history carried no additional content decision.

This closes the tested depth-8-through-32 exact-suffix extension. It does not
reject exact copy, longer approximate matches or a trained vector-space read.

## What did not work

- Fresh-dev longest-32 and longest-4 selected byte-identical token
  trajectories. Their paired body-position comparison was 0 candidate-only
  and 0 control-only on each seed.
- On the training diagnostic, 101 selected tokens differed after the reference
  body on each seed; every scored body position still matched.
- The fresh-dev bar failed despite adequate power: 458 reachable replies,
  29,312 decisions, and non-vacuous random-32 changes on 1,612 decisions and
  33 rows per seed.
- No arm emitted BOS or an exact reference. This was expected because the
  experiment changes content selection only.

## What changed

- Preserved the historical five-arm `CopyKeyReplyArm::ALL` API.
- Added a separate seven-arm evaluation set with `longest_32_to_1` and a
  distinct `random_available_depth_32`; the legacy random arm remains 1..4.
- Extended only the complete-reply suffix search to depths 1..32.
- Made the copy oracle accept both historical five-arm and new seven-arm
  reports, with source serializer names checked.
- Added `--allow-underpowered` so an underpowered report can be sealed before
  classification rather than disappearing behind a driver error.

No writer, memory, point-key, stopping, serving, CUDA or float-training path
changed.

## Measurements

Fresh panel `cell=dev,dev`, seed 2752, 128 conversations and 458 reachable
replies per seed:

| Artifact seed | Longest-32 body | Longest-4 body | `k2>k1` | Depth 1 | Random-32 | Legacy random | Writer |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | **300** | **300** | 248 | 148 | 201 | 182 | 58 |
| 2 | **300** | **300** | 248 | 148 | 201 | 182 | 65 |

Training diagnostic was also identical: **409** correct body positions for
both longest-32 and longest-4 on each seed.

Longest-32 selected depth 32 on 15,114/29,312 fresh-dev decisions and selected
each depth one through four 458 times. The remaining 12,366 decisions selected
depths five through 31. The arm therefore exercised deep matches rather than
falling back to longest-4. It still chose the same token as longest-4 at every
fresh-dev decision.

Longest-32 beat `k2>k1`, depth 1, random-32 and writer by the paired two-sigma
rule. The required gain over nested longest-4 was absent.

Both fresh-dev repeats matched after timing fields were removed. Premature BOS
stops and BOS-complete replies were zero for every arm.

Resources:

- release build **19.05 s**, peak **1,205,920 KiB**;
- generator build **9.94 s**, peak **1,280,868 KiB**;
- panel generation **0.61 s**, peak **7,796 KiB**;
- each report run **3.08-3.17 s**, peak **439,536 KiB**;
- driver **2,141,984 bytes**, SHA-256
  `8477c8588a04978c74f92b67ad378decac32ff679f017e032c291970ab5951aa`;
- fresh target **7,441,615,364 bytes**; free space after run
  **438,262,300,672 bytes**;
- published depth-32 evidence **8,585,858 bytes**.

Local run manifest SHA-256:
`1d8b2af139899ecba803e5fc42b96c466cf7fd4034a47676c0536824468bf3da`.
Published depth-32 raw manifest SHA-256:
`f1114cd15dd8ca6e7750b161f26cde49af5caeac7a47b5fd66f98ff18d5c04f8`.
Published evidence manifest SHA-256:
`d33e500e59ff95739a3f17e1ae0681f5804b46da9e3991fdf1ccf665e33fa5d1`.

## Checks run at the head

- Fresh FIT and independent GO preceded panel generation.
- Full CPU library suite: **288 passed**, 4 expected ignores; all 82 binary
  tests and 13 focused copy-reply tests passed.
- Scoped all-target Clippy with `--no-deps -D warnings`, package rustfmt,
  D23/D25/D32, claim wording and `git diff --check` passed.
- Source, artifact, tokenizer and training-panel hashes matched.
- Fresh panel identity, both reachability floors, random non-vacuity,
  depth-above-four use, ungated token identity, paired comparisons, repeats
  and output-size cap passed.
- Historical five-arm oracle compatibility was exercised against the sealed
  deepseek-153 report.
- No serving, integer runtime, CUDA, float-training or Claude-owned source
  changed.

## Size

Against base `7fe55fdbf`, this unit changes four CPU-train files
(**+121 / -23**), nine task, preregistration, notebook, roadmap, state and
lab-record files (**+643 / -0**), and 30 published evidence files
(**+380 / -0**): **43 files, +1,144 / -23**. No corpus or binary is tracked.

## Skills used

No system-provided Codex skill applies. Project CPU FIT, GO, evidence and
delivery procedures are used.

## Still open

- Test non-exact geometric copy/read representations rather than deeper exact
  suffixes.
- Train/export a copy read selected by a positive complete-reply mechanism.
- Add independent termination and terminal-token reconstruction.
- Complete replies, fresh phrasing and useful local chat remain open.

## Plain summary

Looking 32 tokens back instead of four changed which depth was selected. On
fresh dev it did not change the copied token; on train it changed tokens only
after the reference. Scored body accuracy tied its nested control exactly, so
the deeper arm fails its bar.
