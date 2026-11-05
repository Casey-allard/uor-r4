# REPORT: writer-admission, claude-289 (why the writer's admission failed at seed 2: the ports went flat)

Lab: claude. [ORDER](./ORDER.md). PR #3722, item #3394. Plan step: P6 of Casey's 4 Nov plan (writer admission), done-when 1, the diagnosis. Base `ef620df0a`.

- **The sheet:** [the prep sheet](../../../../harness/prep/writer-admission-diagnosis-2026-11-05.md) ran as pre-registered. It reached FIT in round 2 and passed a D67 pre-run review (#3394, 6657781139).
- **Run 1,** at the head `dae2911be`, stopped on an instrument fault before any reading.
- **Run 2** ran the sheet's fallback: the same reading on claude-150's code `074d54b2`, on the evidence branch `claude/evidence/writer-admission-074d54b2` at `8cb2ff9813515463665ab8c2c6269a0f976222c0`.
- **The pod:** one RTX 5090, Vast `vast-59260630`, at $0.56/h, up from about 12:14Z to 12:33Z. Both runs cost about $0.19.

**In one paragraph.**
- **The classes:** seed 2 reads **F, flat**, at 14 of its 15 checkpoints from step 4,096, and N at the 15th (14,336). Seed 1 reads **L** at all 15. The classes differ, so seed 2's F stands (the sheet's seed rule).
- **What F is here:** from step 4,096, seed 2's two query ports sit at the centre of the 16 roots, where every root scores alike.
  - **The ports shrink:** their coordinates' mean absolute value falls from 2.2e-5 and 2.9e-5 at step 4,096 to 5.4e-7 and 8.3e-7 at 32,768. Over the same steps the whole query's grows from 1.06 to 3.97.
  - **The scores tie:** at step 32,768 every one of the 14,287 DEV rows has its 256 cell scores within 1e-4 of each other, and 5,625 rows within 1e-6.
- **It is a near-tie, not an exact one.** An exact tie would keep cells 0–7 on every row. Instead:
  - only 30–84 rows rank the target's cell at its own id. That is near the 56 (14,287 / 256) a rank unrelated to the id gives by coincidence, though not a fit to it (56 ± 7.5 at one checkpoint), and far from the 14,287 an exact tie gives;
  - 110–246 distinct cells are kept first;
  - residual differences pick the kept cells. Admission is then a lottery: 208–714 rows a checkpoint, a mean of 442 over the 15, against chance's 446 (8 of 256 cells).
- **The ports carry no target information the mean rank can see:** the target root's mean rank is 6.47–8.10 against chance's 7.5.
- **The trunk learned anyway:** nearest's hits rose from 3,702 at step 4,096 to 6,909 at 32,768. The failure lies in the 16 port coordinates, not the trunk.
- **By the sheet's rule, the split arises in training:** both seeds read U alike at step 0.
  - That rule does not exclude a difference in the two seeds' initializations or data order, which a step-0 class cannot see.
  - Still open, item 2's rerun would time the split, not separate those causes. That would take runs that change the initialization with the data order held, and the reverse.
  - By step 2,048 they have split. Seed 1's ports point at the target's roots (mean ranks 2.68 and 3.32, class O).
  - Seed 2's ports have shrunk to 0.02, with mean root ranks near chance (7.08 and 7.28, U), and its nearest reads 76 hits, against seed 1's 2,630.
- **What it points done-when 2 to (the sheet's F row):** a change the tie point cannot satisfy, built from the roots' own geometry. Examples: cells kept by the target's own ports in training, or a port-to-target-root term that reads 0 only on the root.
  - A margin alone is not expected to leave the tie. That is an inference from the near-tie, not tested here.
  - Still open, item 1, adds one caution.
- **Two seeds, one failing run:** the class names this run's failure. It is not a rate.

## What did not work

1. **Run 1 stopped at the probe on an instrument fault, before any reading.** At `dae2911be` the CUDA library suite passed (370 passed, 0 failed, 12 ignored, both admission tests among them), and both seals checked 38 of 38 `run/` files.
   - **The refusal:** `StackModel::load` refused seed 2's step 32,768 with `native projection compiler/operator/FBS identity is missing or stale`.
   - **Why:** each checkpoint pins its projection plan's fingerprint, which includes the sha256 of `spiralcore_fbs.cu`. claude-179's `bcf2f57f7` (#3367, 29 Oct) changed that file's backward, putting fixed-order reductions in place of atomicAdd. So main refuses every checkpoint saved before it, claude-150's included, though the forward did not change.
   - **The fallback:** the sheet's stop rule for an I-2 failure, applied to this refusal. The reading was built on `074d54b2` (the evidence branch above) as one commit.
     - `hamilton_sheet.rs`'s instrument block and its three tests went in byte for byte (+614).
     - `in_space_long.rs`'s `admission` field and its I-1 check were ported by hand (+16).
   - Run 2 passed I-2 at all 34 checkpoints. Posted on #3394 (6657976950).
   - **What this leaves:** main cannot load claude-150's checkpoints at all (Still open, item 3).
2. **FIT round 1 was NOT FIT,** at `33d3483d1`. The sheet had no rule for when a seed's checkpoints disagree. Round 2, at `58090081d`, added the 12-of-15 rule, the rule against a class shared with seed 1, and the negative gate I-3b, and read FIT.
3. **The class does not say why seed 2 went flat and seed 1 did not.** Both read U at step 0 and have split by step 2,048. The checkpoints are 2,048 steps apart, so neither the moment of the split nor its trigger is seen. Seed 1 reads O at 2,048, not L.
4. **The cost estimate was far off, harmlessly.** The sheet projected about 35 minutes for the 34 checkpoints. Run 2's two processes read them all in 59 s, and the whole job took 1 min 41 s after its fetch.
5. **The laptop's first CPU suite at the head failed to compile,** a fault in the build cache, not the code. This worktree and the evidence worktree had shared one `CARGO_TARGET_DIR`.
   - Cargo's dep-info records paths relative to the workspace root, so the head's build took the evidence build's `uor-r4-integer` (074d54b2's source, built later) as fresh, and found none of today's snapshot types.
   - After `cargo clean -p` of the four path crates, the suite ran (Checks run at the head).

## What worked

- **The instrument checks held** (Measurements):
  - I-1 at all 34 checkpoints, since both inspections exited 0;
  - I-2 exactly at all 34;
  - I-3 and I-3b on the pod's build;
  - I-4: 14,287 rows everywhere.
- **The pre-registered classes separated the seeds cleanly:** no checkpoint from step 4,096 on reads O at either seed.
- **The diagnosis is cheap to repeat:** under two minutes of GPU time for both seeds, once the inputs are fetched.

## What changed

- **`hamilton_sheet.rs` (+614):** the admission reading.
  - `AdmissionCounts` and `PortCounts`, mergeable across batches, with `AdmissionReading` and `PortReading`.
  - `admission_counts` (one batch of queries against the writer) and `admission_dev_panel` (all of DEV, on the panel's own points).
  - `ADMISSION_SPREAD_BOUNDS = [1e-6, 1e-4, 1e-2, 1e-1]`, the sheet's bands.
  - Three tests:
    - `admission_counts_merge_and_read` (host);
    - `admission_counts_read_known_ports_on_cuda`: ports on roots 3 and 5, and the zero query, which ties all 256 cells;
    - `admission_counts_read_a_near_tie_on_cuda`: a port at `5e-5` times root 0, whose span must lie inside the F band.
- **`in_space_long.rs` (+26):** `--inspect` prints the `admission` block at every checkpoint, and fails the inspection when its admitted or row count disagrees with the DEV panel (I-1).
- **Docs:** the sheet and its outcome, the ORDER, this REPORT, one notebook row, and the item's "What is missing".
- **No change** to the writer, the trainer, any objective or any served path. `native_vector_writer.rs` is untouched, since every checkpoint pins its sha256.
- **The evidence branch** (+630 on `074d54b2`) carries the same instrument for run 2 and is not merged.
  - Its worktree is removed and the pushed branch kept.
  - Its one commit is also stored as `evidence-8cb2ff98.patch` beside the results (SHA-256 `b4745e5cf0db6115b56dbae84a215ae9ece25c1719cee9f904a0e76a5fbbfc58`).
  - `uor-harness check` warns that the branch has no PR. It is kept rather than archived, because this REPORT and the store's `job.sh` name its commit (Still open, item 6).
- **After the exact-head review,** test only: `admission_counts_merge_and_read` also asserts `rank_is_cell_rows`, the second port's `mean_abs` and `target_root` after a merge, each carried on both sides.
  - Each assert fails with its part of the merge set aside:
    - dropping the `rank_is_cell_rows` line reads 1 against 2;
    - merging only `ports[0]` (the port loop cut to one) reads 0.03125 against 0.125;
    - dropping the `target_root` line reads (3, 0) against (3, 1).
  - The test passes again after the restore and a rebuild.
  - The instrument block itself is unchanged, and stays byte-identical to the evidence branch's.
- **Size:** the ORDER gave no line estimate. The PR adds 647 lines of code (621 in `hamilton_sheet.rs`, 26 in `in_space_long.rs`), about a quarter of them tests, and removes none.

## Measurements

**The command,** per seed S, on the pod: `train-in-space-8cb2ff98 --data-root /workspace/tis-in/w2-inputs/2026-10-25/driver-data --retain-rates 1 --inspect RUN --seed S`, where RUN is claude-150's `sS-074d54b2/run`.
- **The classes** come from `jq -s -f classify.jq inspect-sS.out`.
  - It encodes the sheet's decision table and the per-seed rule (12 of 15, step 32,768 among them).
  - The rule against a class shared with seed 1 is applied in this REPORT, by hand.
  - It is stored beside the seal, SHA-256 `0d8b50e926e42056a32275cfa15b5a449fae39120a5a4512cfe9f76c76b86d73`.
- **The rows** are DEV's 14,287 scored rows at every checkpoint.
- **The columns:**
  - **admitted:** rows whose target cell is among the 8 kept;
  - **< 1e-4, < 1e-6:** rows whose 256 cell scores span under that bound;
  - **spread:** the span's mean;
  - **rank = id:** rows ranking the target's cell at its own id, as a tie of all 256 gives;
  - **first cells:** distinct cells kept first;
  - **root ranks:** the target root's mean rank in each port (chance 7.5);
  - **port |x|:** each port's mean absolute coordinate.

**Seed 2, class F (14 of 15 from step 4,096; first at 4,096):**

| step | class | admitted | < 1e-4 | < 1e-6 | spread | rank = id | first cells | root ranks | port \|x\| |
|---|---|---|---|---|---|---|---|---|---|
| 0 | U | 526 | 0 | 0 | 1.18 | 73 | 256 | 7.22 / 7.41 | 0.794 / 0.766 |
| 2048 | U | 601 | 0 | 0 | 0.034 | 66 | 256 | 7.08 / 7.28 | 0.023 / 0.018 |
| 4096 | F | 608 | 14287 | 0 | 44e-6 | 50 | 202 | 7.74 / 6.95 | 22e-6 / 29e-6 |
| 6144 | F | 422 | 14287 | 0 | 34e-6 | 47 | 246 | 7.26 / 7.6 | 17e-6 / 21e-6 |
| 8192 | F | 306 | 14287 | 0 | 35e-6 | 45 | 150 | 7.28 / 7.73 | 15e-6 / 25e-6 |
| 10240 | F | 208 | 14287 | 0 | 32e-6 | 60 | 120 | 7.74 / 7.94 | 18e-6 / 19e-6 |
| 12288 | F | 333 | 14287 | 0 | 38e-6 | 49 | 196 | 6.59 / 7.85 | 15e-6 / 29e-6 |
| 14336 | N | 429 | 11060 | 0 | 78e-6 | 58 | 199 | 7.26 / 7.52 | 19e-6 / 71e-6 |
| 16384 | F | 461 | 14287 | 0 | 27e-6 | 55 | 110 | 7.5 / 7.27 | 9e-6 / 18e-6 |
| 18432 | F | 572 | 14287 | 0 | 23e-6 | 75 | 152 | 7.16 / 7.74 | 4e-6 / 23e-6 |
| 20480 | F | 335 | 14287 | 0 | 22e-6 | 31 | 129 | 6.78 / 7.8 | 10e-6 / 15e-6 |
| 22528 | F | 561 | 14287 | 0 | 11e-6 | 45 | 213 | 7.36 / 7.3 | 4e-6 / 8e-6 |
| 24576 | F | 402 | 14287 | 0 | 14e-6 | 30 | 170 | 7.02 / 7.71 | 4e-6 / 13e-6 |
| 26624 | F | 302 | 14287 | 0 | 11e-6 | 53 | 163 | 7.19 / 8.1 | 6e-6 / 7e-6 |
| 28672 | F | 271 | 14287 | 0 | 7e-6 | 80 | 162 | 7.15 / 7.77 | 3e-6 / 5e-6 |
| 30720 | F | 699 | 14287 | 0 | 4e-6 | 77 | 169 | 6.51 / 7.12 | 1e-6 / 3e-6 |
| 32768 | F | 714 | 14287 | 5625 | 1e-6 | 84 | 176 | 6.47 / 7.58 | 1e-6 / 1e-6 |

At step 14,336, all 14,287 rows lie under 1e-2, so it reads N.

**Seed 1, class L (15 of 15 from step 4,096; first at 4,096):**

| step | class | admitted | < 1e-4 | spread | rank = id | first cells | root ranks | port \|x\| |
|---|---|---|---|---|---|---|---|---|
| 0 | U | 430 | 0 | 1.191 | 53 | 256 | 7.41 / 7.46 | 0.795 / 0.769 |
| 2048 | O | 7759 | 0 | 0.326 | 87 | 256 | 2.68 / 3.32 | 0.165 / 0.157 |
| 4096 | L | 8558 | 0 | 0.629 | 29 | 254 | 2.44 / 2.67 | 0.29 / 0.313 |
| 8192 | L | 8966 | 0 | 0.769 | 29 | 255 | 2.09 / 2.59 | 0.392 / 0.359 |
| 16384 | L | 9451 | 0 | 0.828 | 33 | 256 | 1.99 / 2.4 | 0.439 / 0.4 |
| 22528 | L | 9563 | 0 | 0.824 | 37 | 255 | 1.89 / 2.4 | 0.447 / 0.406 |
| 32768 | L | 9431 | 0 | 0.817 | 22 | 255 | 1.97 / 2.56 | 0.453 / 0.423 |

- **Seed 1's other eight checkpoints** read L as well. Over 4,096–32,768 they span:
  - admitted 8,558–9,563;
  - root ranks 1.89–2.44 and 2.40–2.84;
  - port |x| 0.29–0.47;
  - spread 0.63–0.84.

  None lies under 1e-6.
- **The trunk at both seeds** (nearest's hits, and the whole query's mean |x|):
  - seed 1: 3,201 at step 4,096 and 6,762 at 32,768 (query 1.18 and 4.76);
  - seed 2: 3,702 and 6,909 (query 1.06 and 3.97).

  At step 2,048 nearest reads 2,630 at seed 1 and 76 at seed 2.
- **The writer's top-1** from step 4,096 to 32,768: seed 1 6,464 to 8,430, seed 2 413 to 493. These are claude-150's logged numbers, reproduced (I-2).
- **The DEV address hinge** (margin 1) from step 4,096 to 32,768: seed 2 1.000030 to 1.0000007, seed 1 0.926 to 0.882.
  - This is the DEV reading at the writer's margin of 1. The run trained with `--objective voronoi`: the token hinge plus an address hinge at half the nearest root separation, 0.25 (`voronoi_loss` in `in_space_long.rs`). The config's `margin_hinge rivals 8` field does not name the trained objective.
- **The target cell's mean rank** (chance 127.5):
  - seed 2 reads 114.7–134.4 from step 4,096;
  - seed 1 reads 30.4–38.2.
- **The instrument checks:**
  - **I-1:** both inspections exited 0, so at all 34 checkpoints:
    - the reading's admitted and row counts equalled the panel's;
    - every batch's port sums equalled the writer's 256 cell scores exactly;
    - the rows whose target cell ranks under 8 numbered the writer's admitted rows.

    That last leg compares a count, not each kept cell's rank (Still open, item 5).
  - **I-2:** `I2-s1.json` and `I2-s2.json` read 17 of 17 with no mismatch, and the probe 1 of 1. The panel's top-1 and admitted counts and nearest's hits equal claude-150's logged ones at every step.
  - **I-3:** the CUDA library suite on the evidence build passed (116 passed, 0 failed, 5 ignored), both admission tests among them.
  - **I-3b:** with `ADMISSION_SPREAD_BOUNDS[1]` at `1e-5`, the near-tie test fails, reading `[0, 0, 1, 1]` against `[0, 1, 1, 1]` (exit 101). The source was then restored.
  - **I-4:** 14,287 rows at every checkpoint.

## Checks run at the head

- **The head** adds this REPORT, the notebook row, the item's lines and the sheet's outcome to `dae2911be`, the commit the pod built at run 1. Its one code change since then is the merge test's three asserts, which touch test code only. Run 2 built the evidence branch.
- **The laptop gates at the head**, after the cache repair in What did not work, item 5:
  - the float crate's CPU suite, `--no-fail-fast`, passes: 306 passed and 0 failed over 9 test targets, `admission_counts_merge_and_read` among them;
  - the stub-nvcc `--features cuda --all-targets` check passes.
  - The two CUDA admission tests ran on the pod (I-3), not on the laptop.
- **The binary:** `train-in-space-8cb2ff98`, SHA-256 `f31f8cbed5c96262e2d892a99066ff8266f3914b6b4b2e644e5542fb4c56734d`, built on the pod at the evidence pin. Run 1's binary is `train-in-space-dae2911b`, SHA-256 `018de9b5bfbdfbcd0d10d84a31bbae73cf1be3b506272b6c8ad1701898b01dbc`.
- **The seal:** `SHA256SUMS-claude-289`, SHA-256 `c90671ea97d068a3674e9bab40c18f265c7bdd82856ab231f3e935dcc0f38a99`.
  - It checks 86 of 86 files on the laptop copy after a read-back from the store, and the read-back's `inspect-s2.out` is identical.
  - **The store:** `caseyallard/uor-r4-store:results/claude/writer-admission-diagnosis-2026-11-05/` holds the seal, `job.sh`, `dae2911b/` (run 1) and `8cb2ff98/` (run 2), with `classify.jq` and `evidence-8cb2ff98.patch` added after the seal.
- **The pod** was released and deleted after the read-back. `uor-pod status` then showed 0 running pods.
- **`loop report --check`, `loop pr-check --local` and `uor-harness check`** were run at the head before the review.
- **Reviews, each in a fresh context:**
  - the sheet's FIT: NOT FIT at round 1, then FIT at round 2 (#3394, 6657781139);
  - the D67 pre-run review: GO (6657781139);
  - the run 1 note (6657976950);
  - the exact-head review of this PR, at `94e513ac4`, in three fresh slices, each APPROVE WITH CHANGES with no blocker:
    - **the instrument code:** it confirms the reading's port scores, cell layout and tie rule match the writer's, and that the bands, means and merge are right. Its three minors are Still open, item 5, with the coincidence reading of `rank_is_cell_rows` now in "In one paragraph".
    - **the tests and wiring:** it confirms the evidence patch's `hamilton_sheet.rs` lines equal the head's, and its `in_space_long.rs` port is equivalent for a non-served inspection. Its merge-test gaps are fixed (What changed); its other points are Still open, item 5.
    - **a claims check of every number:** it reproduced both seeds' classes and every table value and range it checked. Its two majors were a causal sentence and an untested margin claim; both are reworded here, in the notebook row and in the item, and so are its two minors.
  - It is recorded by `loop review` at the final head.

## Skills used

None applied.

## Still open

1. **Done-when 2, the admission change,** guided by class F. One caution for its sheet, an untested reading:
   - **Why the centre may attract:** every root has 6 of its 8 coordinates at 0. So an L1 pull toward the target's root is smallest near the centre, when it is averaged over targets the port cannot yet tell apart.
   - **What fits it:** seed 2's ports shrank to 0.02 by step 2,048, while its nearest read 76 hits.
   - **What it means for the change:** a port-to-target-root term alone may share that centre as its expected minimum. The sheet should check that its change leaves the centre for a port that is still uninformed, not only that the term reads above 0 there.
2. **When and why the seeds split,** before step 2,048. A seed-2 rerun that saves checkpoints in its first 2,048 steps would show it. That is a training run, with its own sheet.
3. **Main cannot load checkpoints saved before `bcf2f57f7`** (29 Oct). The projection fingerprint includes the sha256 of `spiralcore_fbs.cu`, whose backward changed while its forward did not.
   - Any later reading of pre-29-Oct checkpoints on main needs either an evidence branch, as here, or a load that pins the forward alone.
   - The second is a change to the checkpoint identity rule outside this item's scope, so it is not taken here.
4. **Done-whens 3 and 4,** the pre-registered run and its REPORT, follow done-when 2.
5. **The instrument's limits** (the exact-head code review; none changes a reading here). They land with done-when 2's unit, which runs the reading again, so the merged instrument stays byte-identical to the block run 2 ran.
   - **I-1's rank leg** compares how many rows rank under 8 with the admitted count. A per-column check would rank each of the writer's 8 kept cells and require column k at rank k. The tie rule was checked against the writer's by reading the code.
   - **A port's per-coordinate spread** is E[x²] − E[x]², summed in f32 on the device. Away from 0 it can cancel when the mean is much larger than the spread. This REPORT cites no spread value, only `mean_abs`.
   - **A served inspection** (`--inspect` on a served run) skips the reading and prints no `admission` field, because its panel reads the Q12 writer while the reading reads the continuous one. A marker saying so would tell "skipped" from "missing".
   - **The near-tie CUDA test's band** is 0.45e to 1.05e, against the sheet's e/2 to e. I-3b shows it can fail.
6. **The evidence branch** `claude/evidence/writer-admission-074d54b2` stays on origin until an archive unit moves it to `archive/history/branch-archive/` with an INDEX row. That folder is outside this item's scope. Its patch is already in the store.

## Plain summary

The model predicts a point, and a cheap "writer" looks up which of 256 boxes (cells) the answer sits in. It does that by comparing 16 of the point's numbers (two "ports" of 8) with 16 fixed reference points (roots), and checks only the 8 best boxes. In claude-150's training run this worked at one random seed and failed at the other. Nobody knew why.

This unit opened the failed seed's saved checkpoints and measured the ports at each one. The ports had collapsed to almost exactly zero, the centre of the 16 reference points. There, every reference point looks equally good, so all 256 boxes score the same to within a millionth, and the writer's choice is a coin toss. The rest of the model still learned: a slower whole-table lookup on the same model kept getting better. The good seed's ports pointed at the right reference points instead. Both seeds started alike, so the collapse happened during training, in the first 2,048 steps.

So the next unit has to change how boxes are chosen, in a way that the "everything ties" point at the centre cannot satisfy. A bigger margin alone is not expected to help, though that was not tested. The first attempt to read the checkpoints failed because today's code refuses checkpoints saved before 29 October. The reading was rerun on the code they were trained with. Every check held, and the pod time cost about $0.19.

References #3394.
