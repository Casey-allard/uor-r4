# Prep sheet: why the writer's admission fails at seed 2 (writer-admission, claude-289, 5 Nov 2026)

Lab: claude. Item: [writer-admission](../../roadmap/items/writer-admission.md), done-when 1. Order: [claude-289](../../loop/tasks/writer-admission/claude-289/ORDER.md). Code: branch `claude/writer-admission`. The admission reading is commit `ca843995d820302739ad43f9f6d38a8825396b02`, and the pod builds the pushed head the job log names.

TEST FITNESS: FIT - research, #3394, two fresh-context rounds (claude-289)
- **Round 1, at `33d3483d11d268151839a69f42fa9d9051c7da44`: NOT FIT.** It found no rule for when checkpoints disagree, plus five notes.
- **Round 2, at `58090081d2e3f15cd4a8159b384029053689a18d`: FIT.** The rule closes the blocker. Its three wording notes and its negative gate (I-3b) are folded in at the head the job log names.

## The question

In claude-150's `voronoi` run, the writer learned at seed 1 and never learned at seed 2. Seed 2's target cell was among the 8 kept cells on only 208–714 of the 14,287 DEV rows from step 2,048 on, against 7,759–9,563 at seed 1. Whole-table nearest learned on the same seed-2 model (6,909 at step 32,768).

**What state are seed 2's two query ports in, from which step, and how does seed 1 differ?** Each port is the first or last 8 query coordinates, scored against the 16 SpiralCore roots. The answer picks done-when 2's change to admission.

One lead is logged. From step 4,096 on, seed 2's DEV address hinge, the mean of `max(0, rival − target + 1)`, reads `1.0000` to the fourth decimal. So the best kept rival scores the same as the target's cell. Seed 1 reads 0.87–0.98 over the same span.

## Steps: what runs

One RTX 5090 pod, no training. `train-in-space --data-root ROOT --inspect RUN --retain-rates 1 --seed S`, for S = 1 and 2, where RUN is claude-150's `s{S}-074d54b2/run`. Its config has no `fbs_arm` (so one-atom) and no served forward; the other flags are left at their defaults, as the run had them.

For each of the 17 checkpoints (step 0, then every 2,048 to 32,768), `--inspect` prints the DEV panel and nearest readings it always printed, plus the new `admission` block. That block is read on all of DEV's scored rows, on the panel's own points, with the panel's writer (8 kept cells):
- **The target's cell:** its rank among the 256, in bands (0, 1–7, 8–15, 16–63, 64–255) and as a mean. Also the rows whose rank equals the cell's own id, which is what a tie of all 256 gives.
- **The kept cells:** how many distinct cells are kept first and kept at all, and how many rows sit on the commonest first cell.
- **The cell scores' spread** (best minus worst of a row's 256): the rows under `1e-6`, `1e-4`, `1e-2` and `1e-1`, and the mean.
- **The cut gap** (the last kept cell minus the best cell left out), and the miss gap (the last kept cell minus the target's, over the rows not admitted).
- **Per port:**
  - the rows scoring each root best (distinct roots, and the commonest root's rows);
  - the rank of the target's own root (cell `c` needs root `c / 16` in the first port and `c % 16` in the last);
  - the best-minus-second root gap;
  - each coordinate's mean, the coordinates' spread across rows, and their mean absolute value.

## Bar: the decision table (pre-registered)

This is a diagnosis, so it has no pass bar. Each class below is decided at a checkpoint over its R = 14,287 rows, in this order:

| Class | Holds when | Means | Points done-when 2 to |
|---|---|---|---|
| **F, flat** | the 256 cell scores span under `1e-4` on at least `0.9 R` rows | Both ports sit at or near a point where all 16 roots tie. The zero point is one: every root of the anchor orbit is a signed placement of (2, 2, 0, 0, 0, 0, 0, 0), 0.5 from it in mean L1. An exact tie keeps cells 0–7 on every row, so the admitted count would not move between checkpoints. The logged count does move (601, 208, 714 at steps 2,048, 10,240 and 32,768), so an F reading is a near-tie, where residual differences pick the kept cells. `rank_is_cell_rows` tells the two apart. | A change the tie point cannot satisfy, from the roots' own geometry. For example: cells kept by the target's own ports in training, or a port-to-target-root term that reads 0 only on the root. A margin alone does not leave the tie. |
| **N, near-flat** | not F, and the span is under `1e-2` on at least `0.9 R` rows | The ports have nearly reached a tie, but not inside the `1e-4` band | As F |
| **C, collapsed** | not F or N, and in a port the commonest best root covers at least `0.9 R` rows | Nearly every row keeps the same row or column of the 16 × 16 cell grid | Cells kept by the target's own ports, or a kept count chosen by the margin |
| **L, both ports** | none of the above, and both ports' mean target-root rank is at most 3.0 | Both ports point toward the target's roots. A cell needs both, so admission then fails on the pair: 8 kept cells are too few for two near-misses. Seed 1, which learned, is expected here. | A kept count chosen by the margin, or cells kept by the target's own ports |
| **P, one port** | none of the above, and one port's mean target-root rank is at most 3.0 while the other's is at least 6.0 | One port learned and the other did not | A change to the failing port's rule |
| **U, uninformed** | none of the above, and both ports' mean target-root rank is at least 6.0 (chance is 7.5) | The ports vary, but not toward the target's roots | First check that the address term's gradient reaches the trunk (instrument). Then cells kept by the target's own ports |
| **O, other** | none of the above, for example a port between 3.0 and 6.0 | Not explained | No change claimed; the REPORT names the next reading |

**How it is read:**
- **Each checkpoint gets one class,** the first row above that holds there.
- **A seed's class** is the one that holds at at least 12 of its 15 checkpoints from step 4,096 to 32,768, step 32,768 among them. Step 4,096 is where seed 2's hinge first reads `1.0000`.
  - Otherwise the seed reads **mixed**, which counts as O, and the REPORT lists the class at each checkpoint.
  - Seed 2's class picks done-when 2's change. Seed 1's class, by the same rule, is the contrast.
  - If seed 2 reads the same class as seed 1, that class cannot be what made seed 2 fail, so seed 2 counts as O.
- **Steps 0 and 2,048** are reported for both seeds. So is the first checkpoint at which the seed's class holds, for each seed that is not mixed.
- If both seeds read alike at step 0 and seed 2 enters its class during training, the cause lies in training (the objective or admission), not the initialization.
- **Two seeds, one failing run:** the class describes this run's failure. It is not a rate over seeds, and the REPORT says so.

## Instrument checks

- **I-1, in code.** `--inspect` stops at a checkpoint where any of these fails:
  - the admission reading's admitted count and row count equal the panel's `admitted_target_hits` and `scored_rows`;
  - the port sums reproduce the writer's 256 cell scores exactly;
  - the ranks under the tie rule admit the writer's own kept cells.

  I-1 checks the reading against the panel on the same build: the forward is deterministic, the gate state is the panel's, and the port sums follow the writer's formula. Only I-2 checks against the logs.
- **I-2, reproduction.** At all 34 checkpoints, the panel's `top1_hits` and `admitted_hits`, and the nearest reading's `hits`, equal claude-150's logged `dev_top1_hits`, `dev_admitted_hits` and `dev_nearest.hits` at that step, exactly.
  - The crate moved by about 31,000 lines since `074d54b2`, though not the writer. So this tests whether today's forward is the trained one.
  - A mismatch is an instrument fault, and nothing is read from that build.
- **I-3.** The crate's CUDA library tests pass on the pod's build. Two of them pin this reading:
  - `admission_counts_read_known_ports_on_cuda`: ports placed on roots 3 and 5, and the zero query, which ties all 256 cells;
  - `admission_counts_read_a_near_tie_on_cuda`: a port at `e = 5e-5` times root 0. Its cell scores' span lies between `e / 2` and `e`, inside the F band.
- **I-3b, negative.** With `ADMISSION_SPREAD_BOUNDS[1]` set to `1e-5` in the pod's source copy, the near-tie test fails. The bound is then restored. This runs after every reading, so it cannot touch them.
- **I-4.** 14,287 rows at every checkpoint.

## Probe ladder

1. **The build and the CUDA tests** at the pinned head (I-3).
2. **Seed 2's step 32,768 alone,** from a run folder holding only its `config.json` and that checkpoint. It reads I-1 and I-2 there, and times one checkpoint to project the wall time. The job stops here if I-2 fails.
3. **All 17 checkpoints of both seeds,** one process per seed on the one GPU.

## Data split

The pinned DEV split, `ft/dev` from `caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`. Its sha256s are checked at load (`hamilton_sheet.rs`): tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. That gives 14,287 scored rows. TRAIN is fetched as claude-150's job fetched it, but no reading uses it.

## Base artifact

claude-150's sealed checkpoints, `caseyallard/uor-r4-store:results/claude/one-decoder-2026-10-27/s{1,2}-074d54b2/run/`. Each seed has 17 `models/step-*/model.safetensors`, its run `config.json` and its `progress.jsonl`.
- **The seals:** `SHA256SUMS-claude-150-s1` (`eb689fe9b532ad0c13b36a4ca248e45bfc39fb811ebd158def603d62b49a2494`) and `-s2` (`6e63f250db2f987f978f20f7aaaaf63510158c6766da06547d1e52775582ff31`). On the pod, `sha256sum -c` checks all 38 `run/` files each seal lists for its seed, before any reading: 17 checkpoints of two files each, plus the run's config, progress and two sample files.
- **The writer:**
  - Each checkpoint's `config.json` pins the writer's source sha256, `8507a7ae337bfdd5f96d030bac33c1423f782cc5600c5988c8ee645324f6307c`. That is today's `native_vector_writer.rs`, and `StackModel::load` refuses a checkpoint otherwise.
  - So this unit leaves that file alone, and the reading reads through its public accessors.

## Minimum test power

- **No sampling enters:** every reading is a count or sum over the same 14,287 rows of a fixed checkpoint, and I-2 requires the forward to reproduce the logged counts exactly.
- **The thresholds are wide bands:** 90% of rows, and mean ranks of 3.0 and 6.0 against chance's 7.5. They are set wide so that the class cannot hang on a few rows. The logged contrast they are read against is 1.5–5.0% admitted at seed 2 against 54–67% at seed 1.
- **The power limit is the design:** one failing seed against one working seed. The class names this run's failure, not how often it happens.

## Cost

One RTX 5090 at about `$0.5/h`.
- **Build and CUDA tests:** about 8 minutes.
- **Each checkpoint:** three forwards over DEV's windows (panel, admission, nearest), in batches of 16 × 192 tokens. That is about a minute, an estimate the probe measures.
- **In all:** 34 checkpoints is about 35 minutes, two processes overlapping. Projected under an hour and under `$1`; the probe checks it.
- **At launch,** the #3394 comment records `uor-pod status` across all labs, this lab's lease, and the completion watcher (L-004).

## Stop rule

- **Stop on:** a failed CUDA test, an I-1 refusal, an I-2 mismatch at the probe or in the run, or a crash.
  - On an I-2 mismatch, build the reading on `074d54b2` plus this unit's instrument commits (an evidence branch) and probe again. `hamilton_sheet.rs` and `in_space_long.rs` have moved since then, so the commits may need porting by hand.
  - If that build also fails I-2, stop. The REPORT records an instrument fault and claims no diagnosis.
  - A healthy run is never cut short.
- **Re-plan past `$3` or 2 hours** (the ORDER).
- **Results** go to `caseyallard/uor-r4-store:results/claude/writer-admission-diagnosis-2026-11-05/`, sealed with a SHA256SUMS, and are read back before the pod comes down.

## Outcome (5 Nov 2026)

- **Run 1, at `dae2911be0d6acd72f729813d08f6064516b72a6`: an instrument fault, before any reading.** The CUDA tests passed and both seals checked, then `StackModel::load` refused seed 2's step 32,768 at the probe: the projection fingerprint pins `spiralcore_fbs.cu`, whose backward changed in `bcf2f57f7` (29 Oct). That is a refusal to load, not an I-2 mismatch, but it stops the same way, so the stop rule's fallback ran (#3394, 6657976950).
- **Run 2, on the evidence branch `claude/evidence/writer-admission-074d54b2` at `8cb2ff9813515463665ab8c2c6269a0f976222c0`** (`074d54b2` plus this reading): I-1 to I-4 hold, I-2 exactly at all 34 checkpoints, and I-3b fails the near-tie test as it must.
- **Seed 2 reads F** (14 of 15 checkpoints from step 4,096, N at 14,336). **Seed 1 reads L** (15 of 15). Both read U at step 0. At step 2,048 seed 1 reads O and seed 2 U.
- The numbers, the store folder and the seal are in [claude-289's REPORT](../../loop/tasks/writer-admission/claude-289/REPORT.md).
- **One line of the table is an inference, not tested here:** the F row's "A margin alone does not leave the tie" follows from the near-tie reading. This run tried no margin change. The table stays as pre-registered.
