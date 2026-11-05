# REPORT: writer-admission, claude-290 (anchored writer ports: the table's port blocks held at each token's cell roots)

Lab: claude. [ORDER](./ORDER.md). PR #3727, item #3394. Plan step: P6 of Casey's 4 Nov plan (writer admission), its second unit; done-when 2, the change. Base `4ea097661`.

- **The change:** `train-in-space --writer-anchor 1` (`LongConfig::writer_anchor`, off by default).
  - The shared vector table's two port blocks, columns `0..8` and `W-8..W`, hold exactly the two SpiralCore roots of each token's own writer cell. The other `W - 16` columns stay learned.
  - They are set once when the model is built, and again after every main optimizer step, in both training loops. The port columns' gradient is masked to 0 before each step.
- **The tests:** four CUDA tests and three host tests. They ran on one RTX 5090 pod, with two negative gates. A 32-step smoke run ran through `run_long` with the option on.
- **The pod:** Vast `vast-59271113`, up from about 13:39Z to 14:00Z at $0.975/h. Two jobs, about $0.35 in all.

**In one paragraph.**
- **What the anchor does to the writer:** a token's own anchored vector, read as a query, puts its own cell first in the writer's `candidates`, at address score exactly 0. This holds for all six tokens tried, from both of a 512-token table's two cell rounds.
  - So with anchored ports, the writer's address distance to a cell is proportional to the ports' part of nearest's mean absolute distance to that cell's tokens (a factor of 8 / W; `nearest_scores`, `in_space_long.rs:1031–1046`).
  - The recipe's own token objective (`cell` or `nearest`) is then what would train admission: argued from the code, not yet tested in training. No separate address hinge is left to lose, as `voronoi`'s could.
- **What the tests pin:**
  - after a main step, the 16 port columns equal the roots bit for bit, and a learned column moved;
  - the gradient the update sees is exactly 0 on the port columns;
  - for an informed row whose ports start near the centre, leaning 1% toward the target's roots, the gradient moves its ports, and coordinate by coordinate never away from the target's roots (three tokens, one table).
- **The negative gates bite:**
  - with the projection's call removed, the step test fails: token 0's first port coordinate reads `1.9998` (bits `1073740146`) against the root's `2` (`1073741824`), since weight decay scaled it by `1 − 1e-3 · 0.1`;
  - with the mask's call removed, the gradient test fails at its zero check.
- **What this does not show:** that admission holds in training. Claude-289's seed 2 failed on the query side, where its two ports fell to the roots' centre, and the anchor fixes only the table side.
  - For a row whose trunk predicts the target's roots, the informed-row test shows the pull toward the target's roots near the centre, from ports already leaning 1% that way. It does not compute the gradient at the centre itself.
  - For an uninformed trunk it is still a fixed point in expectation (the ORDER's "What it does not claim").
  - Done-when 3's pre-registered run decides it, and must read the query ports before step 2,048.

## What did not work

1. **Nothing here measures admission under training.** The smoke run is 32 steps at batch 16, 0.0075 of an epoch.
   - It reads `479` of DEV's 14,287 rows admitted at step 32, about chance's 446 (8 of 256 cells), and `466` at step 0.
   - That is expected: the table's ports are anchored from step 0, but the query's ports come from an untrained trunk.
   - It shows the option runs end to end and is recorded. It is not a result about the writer.
2. **The design review's first reading of the collapse was wrong, and the ORDER was corrected** (`685aa65b3`).
   - The review inferred that weight decay could explain the ports' fall.
   - Over claude-150's schedule (lr from 0.0005 to 0, linearly, over 32,768 steps) decay alone scales a decayed weight by about e^(−0.1 × 6.27) ≈ 0.53 between steps 4,096 and 32,768, about 1.9×.
   - The ports fell about 40× over the same steps. So most of the fall came from the gradient, and excluding the port-producing weights from decay is held in reserve, not expected to fix it.
3. **The builder subagent reached its 200k context mark with no gate verified.** I ran the host gates, the stub check and the full CPU suite myself, and committed the code (`15abfc258`).
4. **The informed-row test does not start where the ORDER's wording puts it.** The ORDER says the ports start at 0. The test starts them at `0.01 · root + 0.001`, for two reasons.
   - candle's gradient of `|x|` at exactly 0 is +1, so a port coordinate at 0 whose target root is also 0 would be pushed off its root by the kink alone.
   - The `+ 0.001` puts every coordinate on one side of every kink. It is the same for every rival, so it does not favour the target.
   - The `0.01 · root` term does favour the target. Where a target root is −2, the start is −0.019, which lies between the target and every rival whose root there is 0, so that coordinate is pulled. From the exact centre, with candle's +1 at 0, it would get no pull from such a rival, while a coordinate whose target root is +2 would. That split is the ORDER's "about half the time".
   - So the test shows the pull near the centre, from a start already leaning toward the target, not at the centre itself.
   - The code review checked this. It also found that the test's first form asserted only that the summed L1 distance to the roots falls, which one coordinate moving away could hide. The test now asserts each of the 16 coordinates (What changed).
5. **No CPU path:** the anchor refuses a non-CUDA device before any arithmetic, as the writer does. So the projection and the mask are pinned only by the pod's tests, and the laptop runs only the three host tests.
6. **Job 2's first attempt stalled, an instrument fault.** Its `git fetch --depth 1` of `b6e214bbf` from GitHub stopped at 16 MB and grew no further for about three minutes; git sets no timeout.
   - I stopped that one git process by its pid (5284). The job then failed its source pin (EXIT 1) and released the GPU.
   - The source was rebuilt on the pod from job 1's checkout and a git bundle of the one new commit (`685aa65b3..b6e214bbf`). Its `HEAD` is `b6e214bbf` and its status is clean, which job 2's own source pin checks again.
   - The failed attempt's folder is kept as `b6e214bb-try1/` in the seal.

## What worked

- **The pod's first job passed every step, at `685aa65b3`:**
  - the float crate's CUDA lib suite: 377 passed, 0 failed, 12 ignored, all seven anchor tests among the passes;
  - both negative gates (each a one-line change, checked by `git diff --numstat` at 1 and 1);
  - the restored file (a clean `git status`), whose seven anchor tests then passed;
  - the binary build, the input fetch and the smoke run.
- **The second job, at `b6e214bbf`** after the review's test change: the suite again, 377 passed, 0 failed, 12 ignored, with all seven anchor tests among the passes.
- **Off is a no-op:** with the option off, `anchored_update(None, …)` runs the update alone (read from the code), and `config.json` carries no `writer_anchor` key (a host test). The existing tests, all run in both suites, pass unchanged.
- **The writer's file is untouched.** `native_vector_writer.rs`'s `binding()` carries its own `source_sha256`, so editing it would have invalidated every existing writer checkpoint.

## What changed

- **`in_space_long.rs` (+360, −6):**
  - `LongConfig::writer_anchor`, serialized only when true.
  - `writer_anchor_cell_roots` (cell `c = t mod 256`, roots `(c / 16, c mod 16)`), `writer_anchor_check` (width ≥ 16 and the writer's own `WriterSpec::validate`) and `writer_anchor_layout`. The layout builds each token's roots and the column keep-mask on the host, from the writer's own `ProjectionPlan::new(8, 8)` roots at scale 1.
  - `WriterAnchor`:
    - `new` refuses a model without `embedding.weight`, a non-CUDA device and a non-F32 table;
    - `project` sets the table to `table · keep + values` through `Var::set`, which keeps the variable the optimizer holds;
    - `mask` replaces the table's gradient with `grad · keep`.
  - `anchored_update`: mask, the main update, project. Both training loops call it, `run_recall` and `run_long`, after building and projecting the anchor once at model build, before the step-0 checkpoint.
  - Under train-as-served, `run_long` restores the float values before the step, so the projection writes float roots. The export rounds the table on a fixed Q12 grid, `round(v · 4096) / 4096` (`crates/uor-r4-integer/src/fbs_snapshot.rs:80–87`, called by `fbs_export.rs`'s `exported_roundings`). So 0 and ±2 round to themselves, and the rounded and exported tables keep the roots. No test checks this; the arithmetic is cited.
  - `nearest_loss` now calls `nearest_loss_on(table, …)`, with the same arithmetic, so the tests can score a hand-built table.
  - Tests:
    - host: `writer_anchor_layout_holds_each_cells_roots`, `writer_anchor_refuses_what_it_cannot_hold` and `writer_anchor_config_json_only_when_on`;
    - CUDA: `writer_anchor_holds_the_roots_after_a_step`, `writer_anchor_masks_the_port_gradient`, `writer_anchor_cell_first_with_score_zero` and `writer_anchor_informed_row_pulls_ports_to_the_target`.
- **`train-in-space.rs` (+14, −1):** `--writer-anchor 0|1` in the usage line and its own paragraph. It refuses the option outside training with `--out`, and allows it in both training loops.
- **After the exact-head review,** test only (`b6e214bbf`): the informed-row test asserts, for each of the 16 port coordinates, that `grad · (q − root) ≥ 0`, so a descent step never moves a port away from its root.
- **Docs:** the ORDER, this REPORT, one notebook row and the item's "What is missing".
- **The builder's choices in the tests,** each checked by the code review:
  - the step test updates only `embedding.weight`, at lr `1e-3`, so without the projection weight decay alone moves the roots' bits;
  - the mask test reads the gradient that `anchored_update` hands the update, not the store before it;
  - the informed-row test uses a 256-token table, one token per cell, so the nearest rival always lies in another cell and its roots differ (What did not work, item 4).
- **Size:** `in_space_long.rs` adds 155 lines of code and 205 of tests, and `train-in-space.rs` adds 14, docs included.

## Measurements

**The pod:** Vast `vast-59271113`, one RTX 5090, CUDA 12.8, bootstrapped at `685aa65b3` in 269 s (D29 parity PASS).

**Job 1, `job.sh` at `685aa65b3fa5070e29afabb214fcaf4b0be3bf4a`** (13:43:52Z to 13:48:23Z):

| step | what | result |
|---|---|---|
| tests | `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1` | 377 passed, 0 failed, 12 ignored, 113.6 s; all 7 anchor tests `... ok` |
| neg-project | `        a.project(model)?;` → `        let _ = a;` (numstat 1, 1), then `writer_anchor_holds_the_roots_after_a_step` | FAILED as required, at `in_space_long.rs:12915`: token 0 column 0, `1073740146` against `1073741824` |
| neg-mask | `        a.mask(model, grads)?;` → `        let _ = a;` (numstat 1, 1), then `writer_anchor_masks_the_port_gradient` | FAILED as required, at `in_space_long.rs:12940`, the zero check |
| restored | `git checkout` of the file, a clean status, then the seven `writer_anchor` tests | 7 passed, 0 failed |
| build | `train-in-space` at the pin | `train-in-space-685aa65b`, SHA-256 `d5d2cc6254ab612eab640cc2564c4121c837a014a7e6b613f2cc606cc85ea545` |
| smoke | `--objective cell --width 256 --retain-rates 1 --decay-mult 1 --batch 16 --lr 0.0005 --lr-end 0 --steps 32 --eval-every 32 --seed 1 --writer-anchor 1`, on `w2-inputs/2026-10-25/driver-data` | exit 0; `config.json` holds `"writer_anchor": true`; `progress.jsonl` has rows at steps 0 and 32 |

**The smoke run's two DEV readings** (14,287 scored rows; a smoke, not a result):

| step | nearest | writer | admitted | dev loss | train s |
|---|---|---|---|---|---|
| 0 | 2 | 2 | 466 | 2.7616 | 0 |
| 32 | 3 | 3 | 479 | 2.7443 | 1.59 |

**Job 2, `job2.sh` at `b6e214bbf56cb783a746d088d22908d988397492`** (13:56:11Z to 13:58:40Z, after the source rebuild in What did not work, item 6): its source pin passed, then the same suite, 377 passed, 0 failed, 12 ignored, 101.6 s, with all seven anchor tests `... ok`.

## Checks run at the head

- **The head** adds this REPORT, the notebook row and the item's lines to `b6e214bbf`, the commit job 2 tested.
- **The laptop gates:**
  - the float crate's CPU suite, `--no-fail-fast`, at `b6e214bbf`: 309 passed and 0 failed over 9 test targets, the three host anchor tests among them;
  - the stub-nvcc `--features cuda --all-targets` check passes at `b6e214bbf`;
  - no added line passes 100 characters, except the usage line, which was already 620 characters on main and grows by 22, to 642.
- **The seal:** `SHA256SUMS-claude-290`, SHA-256 `8b02f8a64a5adf375a34e8eae42a4cbf6a02f58b262f8691cb0c9e7410f5be04`, over 69 files.
  - A read-back from the store gives a byte-identical seal, and it checks 69 of 69 files.
  - **The store:** `caseyallard/uor-r4-store:results/claude/writer-anchor-tests-2026-11-05/` holds the seal, both job scripts (`workspace/job.sh`, SHA-256 `5d3541b2…`, and `workspace/job2.sh`, `276ea4f0…`, each equal to the laptop's copy), job 1's `c290/685aa65b/`, job 2's `c290/b6e214bb/`, the stalled attempt's `c290/b6e214bb-try1/` and the binary.
- **The pod** was released and deleted after the read-back. `uor-pod status` then showed 0 running pods.
- **`uor-harness check`, `loop gates --full`, `loop report --check` and `loop pr-check --local`** were run at the head before the review record.
- **Reviews, each in a fresh context:**
  - **the design review,** before any code: SOUND WITH CHANGES. All six of its findings went into the ORDER, among them the restated claim, the query-side decay and the requirement that done-when 3 read the ports before step 2,048. One inference in it was wrong (What did not work, item 2).
  - **the exact-head code review,** at `685aa65b3`: APPROVE, with two minors and three nits.
    - Train-as-served's rounding: answered by the Q12 citation in What changed.
    - The informed-row test: strengthened (`b6e214bbf`).
    - The host layout test checks against its own helper, so the agreement with the writer's address order is pinned by the CUDA test `writer_anchor_cell_first_with_score_zero`.
    - The step test's "a learned column moved" passes from weight decay alone. The mask test pins the masking.
    - The usage line was already over 100 characters.
    - The reviewer confirmed that nothing else writes `embedding.weight` after the projection. Train-as-served rounds, then restores before the step; the checkpoint closure rounds, saves and restores; no optimizer other than `opt_main` steps the table.
  - **a claims check of this REPORT,** the notebook row and the item's lines, at `f0a7ba9df`: FIX CLAIMS, two minors and three nits, all fixed in the next commit.
    - "No training run" was wrong, since the 32-step smoke trains; it now reads "no training run beyond a 32-step smoke".
    - The informed-row test was cited as showing that the centre is not stationary. It starts the ports leaning 1% toward the target's roots, so it now shows the pull near the centre, not at it (What did not work, item 4).
    - "Trains admission" is now "would train admission", argued from the code. "Off runs the update alone" is marked as read from the code, not tested. The writer's distance is now "proportional to" nearest's port part, a mean, not an L1 sum.
    - It reproduced the seal, both suites' counts, the restored count, both negative gates' bits and their arithmetic, the CPU suite, the usage line's lengths, the diff's split, the smoke readings, the decay arithmetic, the 40× fall and the cost.
  - It is recorded by `loop review` at the final head.

## Skills used

None applied.

## Still open

1. **Done-when 3, the pre-registered run:** claude-152's (a) recipe with `--writer-anchor 1`, two seeds, the writer and whole-table nearest read on the same checkpoints, with the cost per token. Its sheet should carry:
   - **the query ports' mean |x| before step 2,048,** with a stop rule. Seed 2's ports had fallen to 0.02 by then, and the centre stays a fixed point in expectation while the trunk is uninformed;
   - **what the anchor costs nearest.** The table is also the input embedding. Its learned columns start at standard deviation 0.02 (`INITIAL_STD`), against roots at 0 and ±2, so at first the 16 tokens of a cell share one input code;
   - **claude-289's instrument** (`--inspect`'s `admission` block), run again. Its limits, listed in claude-289's Still open, item 5, land with that unit, not this one, which ran no reading.
2. **Held in reserve:**
   - excluding the port-producing trunk weights from decay (What did not work, item 2);
   - a kept count chosen by the margin, which needs the writer's file, and which `validate()` caps at 16 of 256 cells.
3. **No test of the rounded table's ports:** the Q12 argument is arithmetic, cited, not tested.
4. **The evidence branch** `claude/evidence/writer-admission-074d54b2` is still kept, as claude-289's Still open, item 6, records.

## Plain summary

The model predicts a point, and a cheap "writer" picks which of 256 boxes (cells) the answer is in, by comparing 16 of the point's numbers (two "ports" of 8) with 16 fixed reference points (roots). In the failed training run, those 16 numbers shrank to the centre of the reference points, where every box looks the same.

This unit adds an option that pins the matching 16 numbers of every word's stored vector to its own box's reference points, and keeps them there after every training step. So a word's own vector now points straight at its own box, and the model's ordinary training goal should teach the box choice too. The next unit's run tests whether it does.

Tests on a rented GPU show the pinned numbers stay exact after a step, that no training signal is wasted on them, and that a word's own vector picks its own box first. They also show that removing either half of the mechanism makes its test fail. What this does not yet show is whether the writer now works in a real training run. That is the next unit's pre-registered run, which watches the predicted point's 16 numbers early in training, where the failure began. The GPU time cost about $0.35.

References #3394.
