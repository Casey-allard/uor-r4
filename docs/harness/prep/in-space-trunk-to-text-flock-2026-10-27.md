# Prep sheet: a Flock read on the in-space trunk, against no read (in-space-trunk-to-text, claude-144, 27 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-144](../../loop/tasks/in-space-trunk-to-text/claude-144/ORDER.md). Code: branch `claude/flock-read-trunk` at `e7918f9a9914f73e777885f9442ddf730e54e65f`. The probe ladder ran `d47e2c1a`. Since then the code changed in two ways: `--inspect` reports the read's weight, and a pick that is not real points at its window's start. The run's first step re-runs the CUDA tests on its own build.

TEST FITNESS: FIT (a fresh non-author reviewer, 27 Oct 2026, posted on #2028; D67 pre-run review PASS)

## The question

The trunk's recurrence learns short memory. claude-142's best checkpoint reads about 8–12 tokens back (`R16 = 0.011`), and 220 of 1,226 reply openings are right, against 161 for always naming the commonest opening. **Does a Flock read lift reply openings and reach together?** The read keeps each position's 7 nearest earlier positions in a learned geometric key space, and moves the point toward the table points of the tokens that followed them.

## Steps: the arms (identical except as stated)

Common to every arm:
- `train-in-space --objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0`;
- the FBS sheet trunk, `build_fbs_model(OneAtomActive)`;
- one RTX 5090 per pod, two arms side by side.

| Arm | Read | Seed | Pod |
|---|---|---|---|
| `read-s1` | `--flock-read 7` | 1 | A |
| `none-s1` | `--flock-read 0` | 1 | A |
| `read-s2` | `--flock-read 7` | 2 | B |
| `none-s2` | `--flock-read 0` | 2 | B |

All four arms run one binary, built at the code commit. Each read arm's twin is the no-read arm with the same seed, which starts from the identical model. The read's weight starts at 0, so the step-0 readings must match exactly (I-2).

## Bar (pre-registered): the decision table, read arm against its twin, both at step 16,384

Definitions, per arm:
- **`O`** = `nearest.opening_hits`.
- **`R16`** = `shift(16) / shift(0)`, from `--reach`.
- **`T`** = `nearest.hits`.

The two tests:
- **Openings pass** when `O_read ≥ max(197, O_none + 36)`. 36 is three binomial standard deviations of one arm's count at the commonest-token rate, `√(1,226 · 0.131 · 0.869) = 11.8`. That is about 2.2 standard deviations of the difference between two independent arms; the threshold is fixed.
- **Reach pass** when `R16_read ≥ max(0.05, 2 · R16_none)`.

| | Reach pass | Reach fails |
|---|---|---|
| **Openings pass** | **A.** The read lets openings use the prompt. If `T_read ≥ 7,002` too, the item's done-when (1) and (2) are met. Unlocks long runs and the reply text on the read. | **B.** Openings improve through context nearer than 16 tokens. Unlocks a unit that reads which neighbours the read keeps. |
| **Openings fail** | **C.** The read reaches far, but its payload does not help openings. Unlocks a unit on the payload (the earlier point itself, or a carried rotation). | **D.** The read is not used. Unlocks a unit on its credit and its start (I-3 tells which). |

- **Flag F**, reported beside the cell: `T_read < T_none − 500`, meaning the read costs fit.
- Both seeds must land in the same cell for the reading to count. If they split, a third seed (3) is run on both arms, and the cell two of the three seeds share is the reading.
- A missing number gives no cell: the step is rerun, never read as a fail.
- `R16` averages over openings with at least 16 tokens before them, and `shift(0)` over all openings. The bias is the same for every arm.

## Instrument checks

- **I-1:** the crate's CUDA library tests pass on the run's build, including the three read tests:
  - the choice: nearest, causal, never itself;
  - zero weight leaves the points unchanged, and unit weight adds the payload mean;
  - one AdamW step matches the hand-computed step.

  The probe ladder's build passed 94, failed 0.
- **I-2:** each read arm's step-0 DEV reading equals its twin's exactly: nearest hits and openings, which go through the read. The printed dev loss is the writer's panel loss, which does not go through the read, so its match is trivial.
- **I-3:** the read's weight moves off 0. The mean of `|w|` in the step-16,384 `flock_read.safetensors` is reported. If it stays under `1e-3`, cell D's cause is the start, not the read.
- **I-4:** no non-finite loss; the runner stops on one.

## Probe ladder (done before this sheet, not a reading)

On `vast-57953920`, at `d47e2c1a`, seed 1, lr 1e-3 → 0 over 4,096 steps, read (K = 7) and no read side by side. Output is in `/workspace/flock/probe`, to be stored with the run.

| At step 4,096 | Read | No read |
|---|---|---|
| Nearest top-1 | `7,028` | `7,309` |
| Openings | `202` (`211` at 2,048) | `179` (`168` at 2,048) |
| `R16` | `0.0215` | `0.0069` |

- The read trains, with no stall. The step-0 readings are identical.
- Throughput was 0.074 s per step with one arm alone, from the 256-step probe.
- This is too short to read by the bars above; the run decides.

## Data split (pinned in `hamilton_sheet.rs`, checked at load)

- **Training** reads only TRAIN, `mw-balp/train`: 68,218 usable windows.
- **Every reading** is on DEV, `ft/dev`: 14,287 targets, 400 documents, 428 windows, which no arm trains on.

The files are under `caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`, with their sha256 prefixes:

| File | sha256 |
|---|---|
| `tokenizer.json` | `d36d3e87…` |
| `mw-balp/train/tokens.u16` | `3dee858f…` |
| `mw-balp/train/response_mask.u8` | `b5e7352d…` |
| `ft/dev/tokens.u16` | `0919e90f…` |
| `ft/dev/response_mask.u8` | `a838bb1a…` |

## Base artifact

No trained checkpoint. Every arm starts from `build_fbs_model(OneAtomActive, seed)`. The read, when on, starts with identity rotations and a zero weight. The binary is built at the code commit, and its sha256 is recorded in the job log.

## Minimum test power

Two seeds per arm, and both must agree. The openings threshold is three binomial standard deviations above the larger of 161 and the twin.

## Cost

Two pods at `$0.563/h` each, two arms per pod. The probe ladder's throughput is 0.074 s per step with the read and one arm alone; two side by side take about 0.15 s per step. That is about 45 minutes per pod, about `$0.85` in all. Pods together cost `$1.13/h` of the `$8/h` cap.

## Stop rule

- Stop on a non-finite loss or a crash; a healthy run is never cut short.
- Results go to `caseyallard/uor-r4-store:results/claude/flock-read-trunk-2026-10-27/` by `uor-store put`, read back with `uor-store ls`, before either pod comes down.
