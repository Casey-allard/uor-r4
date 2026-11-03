# REPORT: repeated-key memory-weight comparison, deepseek-95

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Sheet:
[cpu-training-pilot-memory-heldout-2026-11-03.md](../../../harness/prep/cpu-training-pilot-memory-heldout-2026-11-03.md).
Base: `5e5c2590e567dd6d080b990e5977caccc5147a78`.

## What did not work

- **The three-sigma ranking bar failed on both seeds.** Trained top-1 gained
  13 rows on seed 1 and 5 on seed 2, against thresholds of 48 and 44 over 2,431
  held-out decisions. This is not a capability or language positive.
- **The first serial cost probe exceeded the practical budget.** One seed-1
  update had run for 9m47s on one core without completing the candidate sweep.
  It was stopped, retained as a cost negative, and the independent candidate
  evaluations were parallelized across all 16 physical cores.
- **Evaluation keys overlap training keys by design.** The comparison tests
  whether learned movement weights transfer across different examples and
  documents with the same exact key. It does not test transfer to an unseen
  opening key.
- **The public source artifacts have no memory.** The comparison adds
  deterministic corpus-derived tables before training. It is not an artifact
  published by the existing public model.
- **The historical finite full-path helper had an off-by-one reply row.** Its
  test-only table derivation started at reply offset 1, omitting the first
  reply-table row. The new production derivation starts at offset 0, and the
  new attach test pins both reply rows. The historical artifact remains
  unchanged so its recorded hash keeps its original meaning.

## What changed

The CPU corpus reader now recovers contiguous masked assistant responses as
teacher-forced prompt/reply examples, with current-turn boundaries and
context-safe truncation.

The memory trainer now:

- derives opening/reply tables from examples with deterministic commonest-target
  tie breaking;
- appends those tables to a UORLUT artifact without changing existing bytes,
  then reads every table back;
- evaluates arbitrary held-out samples with memory on or off;
- supports table replacement for a matched random control;
- parallelizes independent candidate artifacts across Rayon workers, with an
  ordered chunk reduction that preserves the serial tie rule.

`cpu-memory-panel-driver` selects repeated keys from corpus documents, attaches
one memory image to each public artifact seed, and retains frozen, trained,
random-move and memory-off readings.

## Setup

- Eight opening keys are present in at least four even training and four odd
  evaluation documents.
- The source-order cap is eight examples per key: **58 training samples** and
  **56 evaluation samples**.
- The derived memory has eight opening rows and 492 reply rows.
- Each public artifact is width 256 and context 192.
- Each seed starts with zero movement weights and runs eight Q24 updates at
  `0.0625`, limit `1.0`.
- Candidate search uses 16 Rayon threads.

The exact inputs, selected keys and bar are fixed in the FIT sheet. The
comparison record is
[comparison.json](evidence/comparison.json); the complete per-seed reports are
[seed-1.json](evidence/seed-1.json) and [seed-2.json](evidence/seed-2.json).

## Measurements

| Seed | Arm | Top-1 / 2,431 | Target distance | Wall | Peak RSS |
|---:|---|---:|---:|---:|---:|
| 1 | frozen | 278 | 2,821,353,534 | — | — |
| 1 | trained | **291** | **2,820,387,931** | 415.370 s | 365,048 KiB |
| 1 | random | 278 | 2,821,404,885 | — | — |
| 1 | memory-off | 278 | 2,821,353,534 | — | — |
| 2 | frozen | 238 | 2,898,309,913 | — | — |
| 2 | trained | **243** | **2,896,327,331** | 414.121 s | 364,360 KiB |
| 2 | random | 238 | 2,898,001,645 | — | — |
| 2 | memory-off | 238 | 2,898,309,913 | — | — |

Seed 1 reduced held-out distance by **965,603** and gained **13** top-1 rows.
Seed 2 reduced it by **1,982,582** and gained **5**. The trained arm beat the
random control's distance and top-1 on both seeds.

The full candidate sweep used about 12–14.5 effective cores. One-step wall time
fell from over 9 minutes serial to **62.307 s** parallel. The two completed
eight-step runs took **6m55s** and **6m54s**.

All five arms on both seeds had exact parity over 56 samples, 2,487 positions,
636,672 Q12 coordinates, 2,487 token decisions and 19,896 kept cells. Every
mismatch counter is zero.

## Reading

The **dense held-out mechanism result is positive and replicated on both model
seeds**: full-path training lowers target distance, raises top-1 without a
fall, and beats a matched random move. The gain disappears at the predeclared
three-sigma ranking threshold, so this does not support a useful-ranking or
language-capability claim.

This is materially different from the failed rank-margin configurations: the
training objective is target distance through the complete exported query after
memory rows fire, and the random control holds the number of moves fixed.

## Checks run at the head

- Both registered seed runs completed with exact CPU/serving parity and all
  eight updates accepted.
- Fresh-target release crate suite: **146 passed, 0 failed, 4 release
  benchmarks ignored**.
- Driver suites: **6, 1 and 5 tests passed** for the CPU corpus,
  counterfactual-corpus and discrete-corpus drivers.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- Package formatting, evidence JSON checks and `git diff --check`: passed.
- Evidence `SHA256SUMS`: both per-seed reports passed.

## Skills used

No available Codex skill applied. The corpus reader, artifact layout, memory
trainer and full-path audit were inspected directly.

## Still open

- A three-sigma held-out ranking gain.
- Transfer to unseen keys rather than repeated exact keys.
- A public full-path artifact with corpus-trained tables and weights.
- Complete replies, fresh phrasing and useful local chat.
- GPU qualification of this mechanism family.

## Plain summary

Two CPU model seeds trained eight memory-weight steps through the complete
served path on a repeated-key public panel. Both improved held-out target
distance and top-1 and beat a random-move control, with exact CPU/serving
parity. The improvements were smaller than the registered three-sigma ranking
bar, so the mechanism is promising but the capability result is still negative.
