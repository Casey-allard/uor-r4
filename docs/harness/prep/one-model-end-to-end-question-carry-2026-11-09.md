# Prep sheet: the question carry (one-model-end-to-end, claude-335, 9 Nov 2026)

The code under review is `95e4c6f0f` (claude-335's code commit, on PR 3888), with this sheet beside it.
- **claude-335 FIT round 1, sheet slice, at `60ddeeab5`:** FIT, no blocker, no major. Seven minors and a nit, all applied:
  - the question table's steps and seed-2 numbers, with B2-aqH's 16,384 line added;
  - branch 3c compares the question row at the same step (4,096), and names a choice that fades with length;
  - the `both` pull's `λ/2` split, stated and named in branch 1;
  - the bar's steps by row;
  - B2-aqC decides branch 3;
  - seed 3's arm list;
  - 3c read whatever the scored row reads.

- **claude-335 FIT round 1, mechanism slice, at `60ddeeab5`:** FIT, no blocker, no major. It checked the carried sum's rows and windows, the pull's flat row indexing, the cut, the refusals, save and load, the default path (bit-identical by reading), and each CUDA mutation against its test. Applied:
  - **Minor:** no test pinned that the forward carries `point + a` rather than the point alone. I-C2 now sets `a[x_t]` nonzero and checks a brute-force sum, with the gate `c-carry-no-a`.
  - **Nit:** the sheet's place for the snapshot export's refusal is corrected.
  - **Nit, not applied:** `with_carry` accepts any `vocab`. The load refuses a mismatch, and the run passes `SHEET_VOCAB` for both, so it cannot arise here.

TEST FITNESS: FIT (round 1, both slices), with every finding applied or answered above.

## The question

[claude-334](../../loop/tasks/one-model-end-to-end/claude-334/REPORT.md) closed the hold for `h = 8`: a held depth-2 stream write does not carry the chosen value to B2's scored row. On both seeds, at 4,096 and at 16,384 steps, every B2 arm answers at most 2 of 512 queries in a far bucket.

Its sealed traces show where the read loses the question. These are the readout read (depth 6), ask heads (`copy == false`), range over heads and buckets:

| Arm (seed 1 / seed 2), step | `kept_ask` at `query` (the `Q` row), of 512 | `kept_ask` at `scored`, of 512 |
|---|---|---|
| B1-a (answers 512 / 512), 4,096 | 512 / 479–512 | 512 / 479–512 |
| B2-aq, 4,096 | 410–512 / 479–512 | 0 / 0 |
| B2-aqH, 4,096 | 512 / 475–512 | 0 / 0 |
| B2-aqH, 16,384 | 343–464 / 355–391 | 0 / 0 |

The question-row choice weakens with length in B2-aqH, which branch 3c below accounts for.

`kept_ask` counts the queries where the head keeps the asked value's position among its picks, with weight above 0 (`count_trace`, `in_space_long.rs:12689`).

**Why B1 passes and B2 cannot:**
- B1 is scored at the question's own row (`RecallQuery::scored_offset`). There the readout read's ask heads ask with the question's point, `point(Q) + a[Q]`, and keep the value's position.
- B2 is scored at `H4`, four rows after `Q` (each tail slot is `Q H1 H2 H3 H4 v`). There the ask heads ask with `point(H4) + a[H4]`. `H4` is the same token in every slot, so no learned table row can name this episode's key, and the scored row keeps nothing.

So the readout read chooses at `Q` and cannot ask that question at `H4`. This unit carries the question's ask point from `Q` to the scored row. It is not a value carry, which claude-334 closed.

## The mechanism: the carried question

Under `--hamming-carry-question h` (it requires `--hamming-ask 1`), each ask head of the **readout read alone** asks at row `t` with:

`asked_t = point(x_t) + a[x_t] + Σ_{s=1..h} c[x_{t−s}] · (point(x_{t−s}) + a[x_{t−s}])`

- **The sum** runs over rows `t − s` of the same window only.
- **`c` is a new learned table:** one scalar per token, starting at 0, saved as `d6.carry`.
- **The code** (at `95e4c6f0f`):
  - `HammingRead::carried` (`flock_read_hamming.rs:1646–1658`) builds the sum with the hold's own shifted sum, `held_term` (`:657–684`).
  - `asked` (`:1632–1640`) adds it to the row's own question.
  - `with_carry` (`:1058–1071`) puts it on `reads[1]` only.
- **At `c = 0` the read is unchanged,** forward and gradient (I-C1).
- **What `c` can reach.** With `c[Q] = 1`, `c[H1..H4] = 0` and `a[H4] = −point(H4)`, the scored row asks exactly `Q`'s question. Then B1's working read applies at `H4`.
- **It starts with a gradient,** unlike claude-334's hold:
  - A loss on the asked point reaches `c[x_{t−s}]` through `point(x_{t−s}) + a[x_{t−s}]`.
  - That point is nonzero at the start, because the token points are.
  - The hold's gradient waited on `w_y = 0` to move.
- **The pull asks the same question.** The found-copy pull's question point is built the same way (`pull_bits`, `:1692–1706`), so the pull trains `c` and the carried `a` (I-C3).
- **No dense matrix multiplication:** a per-token scalar table, a shifted causal sum and an add.
- **No squashing:** `c` is a raw scalar, with no bound and no normalization.
- **Not served in this unit:**
  - The decode step refuses a carried read by name (`:1836–1840`). So does the decode cache, which refuses it in `geometric_stack_native.rs`.
  - Both export paths refuse it by name in `fbs_export.rs`: `round_hamming_as_exported` before any other clause, and `fbs_hamming_snapshot` after its depth and width check and before every per-depth clause.

**`--hamming-attract-at both`.** The pull sits at the question rows and the scored rows together (`attract_plan`, `in_space_long.rs:3107`, gives `[header, 0]`; `attract_rows_at`, `:3072`). `scored` and `question` are unchanged. The pull's loss is the mean over all its terms (`flock_read_hamming.rs:1113–1122`), so under `both` each placement carries about `λ/2`, while B2-aC's scored rows carry the whole `λ`. The logged pull distance under `both` is the pooled mean of the two placements.

**The cut `−C`.** An evaluation-time cut that drops the carried term alone (`set_carry_off`; `Ablation::CarryOff`, `in_space_long.rs:12454`, pushed for carried arms at `:12910`). It leaves `c`, `a` and every other read untouched (I-C4). A pass is attributed to the carry only if the arm's `−C` recall fails the bar.

**`h` is pre-registered:** `h = 4` reaches exactly `Q` from `H4`, and `h = 8` also covers the previous slot's `H*` and `v`, so `c` must learn to ignore them. No other `h` is run.

## Steps: the arms

**Common to every arm**, as claude-334's sheet:
- `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --lr 0.001 --lr-end 0 --recall-eval-windows 256 --recall-readings 1`;
- context 192;
- seeds 1 and 2, with seed 3 if the two seeds split. Seed 3 runs the whole arm list below, behind the same probe ladder.

The ask arms take `--hamming-read 4 --hamming-ask 1 --hamming-attract 1`.

| Arm | Form | Read | Steps, eval every |
|---|---|---|---|
| **B1-a** | B1 | ask (instrument: I-R1, I-T) | 4,096, 512 |
| **B2-aq** | B2 | ask, `--hamming-attract-at question` (instrument: I-R2, I-T0) | 4,096, 512 |
| **B2-n** | B2 | none (the no-read twin, L-010) | 16,384, 2,048 |
| **B2-aqC** | B2 | ask, `--hamming-attract-at both --hamming-carry-question 4` | 16,384, 2,048 |
| **B2-aqC8** | B2 | ask, `--hamming-attract-at both --hamming-carry-question 8` | 16,384, 2,048 |
| **B2-aC** | B2 | ask, pull at the scored rows (the default), `--hamming-carry-question 4` | 16,384, 2,048 |

- **The carried arms run 16,384 steps from the start.** claude-334's held arms needed the extension to be read, and each extension was a second run. Here there is no extension, so no arm is read early.
- **B0 is dropped.** claude-334 passed it at both seeds (B0-a 1.0, B0-n ≤ 0.005). It checks the exact copy, which this unit does not touch with the carry off. I-1 checks those default paths bit for bit, and B1-a checks the ask instrument the map reads.
- **Readings**, as claude-334's, plus:
  - on the carried arms, `R^{−C}`, the recall under the cut. The trace is taken on the plain forward only (`in_space_long.rs:12967–12993`), so no branch reads a trace under a cut;
  - `c` per checkpoint for the 16 question tokens, the 4 header tokens and the 512 value tokens (each as mean, minimum and maximum), and the mean of the rest (`RecallReading::hamming_carry_gain`).
- claude-334's stream ranks are recorded where the code writes them, and are descriptive only: **no stream rank decides a branch** (claude-334's lesson).

## Bar (pre-registered), both far buckets, both seeds

The instrument rows are read on the 4,096-step arms at step 4,096; the B2 and attribution rows at step 16,384.

| Check | Passes when |
|---|---|
| **I-R1 (instrument)** | B1-a passes claude-330's B1 rule at both seeds: `R_ask ≥ 0.50`, `R_ask ≥ R_oth + 0.25`, `R_ask ≥ R_ask(B1-n) + 0.10`. B1-n is claude-330's sealed line at the same seed, as in claude-334 |
| **I-R2 (instrument)** | B2-aq's step-4,096 line, against claude-330's sealed B2-aq line at the same seed: the same `rows`; each bucket's `hits` within `0.02 · rows`; and every `position == "query"`, `copy == false` trace entry's `kept_ask` within `0.05 · rows` of the sealed entry at the same `(depth, head, bucket)`. This is claude-334's check, unchanged |
| **I-T (instrument)** | B1-a's step-4,096 trace: every `position == "scored"`, `depth == 6`, `copy == false` entry has `kept_ask ≥ 0.5 · rows` (claude-334 read 512 of 512) |
| **I-T0 (instrument)** | B2-aq's step-4,096 trace: every `position == "scored"`, `depth == 6`, `copy == false` entry has `kept_ask ≤ 0.05 · rows` (claude-334 read 0) |
| **B2 (the carry)** | the B1 rule for B2-aqC, B2-aqC8 and B2-aC, each separately against B2-n at the same seed |
| **Attribution** | a carried arm that passes is credited to the carry only if its `R^{−C}` fails the B1 rule against B2-n |

I-T and I-T0 together show that the map's instrument reads success where B1 succeeds and failure where B2-aq fails.

## Outcome map (read in order, first match wins)

The branches read the **scored-row trace** of the readout read: `position == "scored"`, `depth == 6`, `copy == false`. "Kept" means `kept_ask ≥ 0.5 · rows` in every such entry; "lost" means below that in every entry. They also read the same arm's question-row trace (`position == "query"`) and `c` by group. All of these are read at step 16,384, at both seeds.

0. **I-R1, I-R2, I-T or I-T0 fails:** an instrument fault. Repair it and rerun; nothing else is read.
1. **A carried arm passes, and its `−C` fails the bar:** the carried question reaches the scored row and answers there.
   - **If B2-aC passes:** the scored loss alone trains the carry.
   - **If only the `both` arms pass:** the carry answers once the question is also trained at its own row. The `both` arms also put half the pull's weight on the scored rows, so the REPORT names the placement and the scored-row weight together as the difference, not the placement alone.
   - Either way the REPORT reads `c` (expected largest on the question tokens) and the plain scored-row trace (expected kept).
   - **Next:** serve the carry at the decode step and in the export. It needs only the last `h` rows' ids. Then a language run with the carry, under its own sheet.
2. **A carried arm passes, and its `−C` passes too:** the answer does not come through the carry.
   - The arm differs from claude-334's failing B2-aq in the pull's placement and in its length.
   - **Next:** B2-aq with `--hamming-attract-at both` at 16,384 and no carry, under its own sheet. No credit goes to the carry.
3. **No carried arm passes.** B2-aqC's readings decide the branch. B2-aqC8's and B2-aC's are reported beside it, each with the branch it would read, and change nothing:
   - **3a. Kept at the scored row, and the recall fails:** the carried question names the key at `H4`, and the readout combine does not turn the kept value into the answer.
     - The REPORT reads `weight_ask` against `weight_oth`.
     - **Next:** the readout combine at a carried question, tested on B2 first.
   - **3b. Lost at the scored row, and kept at the question row:** the question is chosen at `Q`, and the carry does not deliver it to `H4`. Read `c` on the 16 question tokens:
     - If its upper bound (`max(|min|, |max|)`) is below 0.01, **the carry never grew**: a configuration result for this `lr` and budget. **Next:** the carry's start (a nonzero start or a learning rate of its own), tested on B2 first.
     - Otherwise, **it grew and does not deliver**: a configuration result for this form. **Next:** the carry's form, a carried question that replaces the row's own rather than adding to it, tested on B2 first.
     - If `|mean| < 0.01 ≤` the upper bound, the REPORT names the case as undecided, and takes the form branch.
   - **3c. Lost at the question row** (whatever the scored row reads): the readout read's ask heads do not choose at `Q`, so there is nothing to carry. Read the arm's own question-row trace at step 4,096 against B2-aq's step-4,096 line, at the same step:
     - If B2-aq's question row is also below `0.5 · rows` at 4,096, it is an instrument fault (I-R2 should have caught it). Repair it and rerun.
     - If the carried arm's question row is kept at 4,096 and lost at 16,384, the choice fades with length, as B2-aqH's did in claude-334 (512 / 475–512 at 4,096, 343–464 / 355–391 at 16,384, with no carry). It is a configuration result of the length, not of the carry. **Next:** the question-row choice over a long run (its margin or the pull's weight there), tested on B2 first.
     - Otherwise (lost at 4,096 where B2-aq keeps it): a configuration result, in which the carried pull disturbs the question's own choice. **Next:** the pull's placement weights (the question rows against the scored rows), tested on B2 first.
   - **3d. Any other pattern** (entries that are neither kept nor lost, or seeds that disagree on the branch after seed 3): named from the readings and recorded as a configuration result.
     - **Next:** a diagnostic sheet, with no lever chosen in advance.

**Undertrained.** A carried arm that misses the bar while its `R_ask` still rises by more than 0.05 between its last two checkpoints (14,336 and 16,384) is reported as undertrained, beside the branch it reads. It is not extended in this unit.

**Seeds:** two seeds that split run a third, and at least 2 of 3 must agree. Every cell, trace entry, `c` group and ablation is reported either way.

## Instrument checks

- **I-1 (default paths):** the three printed digests in `flock_read_hamming.rs`'s `i1_digest` module (`:5138`) are equal at the base `734450768` and at the head. They are `ask_default_forward_digest`, `ask_no_hold_forward_digest` and `no_read_forward_digest`. The new flags are left out of `config.json` when off (`skip_serializing_if`), and a host test pins that.
- **I-C0 (host, load and refusals):** `the_carry_starts_at_zero_loads_back_and_is_refused_where_it_is_not_served`.
  - The carry needs the ask heads and at least one row.
  - `c` starts at zero, one per token, at the readout read alone.
  - It saves as `d6.carry` with its row count, and loads back.
  - A save with half of it, a carry at the stream depth, or a carry with no ask heads is refused on load.
  - The decode step and both export paths refuse a carried read, each naming the carry.
  - Also `the_carry_cut_keeps_the_decode_cache_off`, and the flag tests in `in_space_long.rs`: the default, the refusal without ask heads, the checkpoint row-count match, the `both` placement's order and its refusals.
- **I-C1 (CUDA, the carry starts off):** `the_carry_starts_off_in_the_readout_forward_and_the_gradient`. At `c = 0`, the forward and the gradient of every shared parameter equal the uncarried read's.
- **I-C2 (CUDA, the carry's rows):** `a_carried_question_reaches_exactly_the_next_rows_of_its_window`. On a hand-built input with one nonzero question point `u` at row `t`, the carried term is `c[x_t] · u` at rows `t + 1` to `t + h`. It is zero at row `t`, at `t + h + 1`, before `t`, and in every other window. Under `−C`, only row `t`'s own question remains.
- **I-C3 (CUDA, the pull):** `the_carried_pull_asks_the_question_rows_question_and_trains_the_gain`.
  - **The setup:** with `c[5] = 1` and `a[1] = −point(1)` at depth 6, the pull's distance from the scored row (10) to the candidate (7) equals the distance from the question row (6), and differs under `−C`.
  - **The gradient:** `c`'s gradient is nonzero at the tokens inside the carry's reach (5, 13, 14, 15) and zero elsewhere, and `a[5]` gets one.
  - **Under `−C`:** `c` gets no gradient.
- **I-C4 (CUDA, the cut):** `the_carry_cut_drops_the_carried_question_alone`.
- **Negative gates:**
  - **13 host mutations** (`local/claude/claude-335/mut-host.sh`), each failing its named test on the laptop: 13 of 13 (`mut-host.out`).
  - **8 CUDA mutations** (`mutate-335.sh`), each type-checked under the stub nvcc (`stub-gates.out`: all 8 build), and each run on the test pod against its named test:
    - the gain starting at one;
    - the forward dropping the carry;
    - a row carrying its own question;
    - the carry reaching `h + 1` rows;
    - the windows read as one;
    - the pull asking without the carry;
    - the carried term detached in the pull;
    - `−C` ignored.
  - **A ninth, from FIT round 1** (`mutate-335b.sh c-carry-no-a`): the forward carries the token point alone, not `point + a`. I-C2 now sets `a[x_t]` nonzero and checks a brute-force sum, so this mutation must fail it. The test pod t1 ran at `310e7d031`, before the change; the run pods run the strengthened I-C2 and this gate at the run pin before any arm trains, and stop if either misses.
- **I-R1, I-R2, I-T, I-T0:** as in the bar.

## Probe ladder

B1-a and B2-aq first on each pod. If any instrument check misses, stop and repair. The carried arms and B2-n do not run on a failed instrument.

## Data split

As claude-330's sheet. Every window is generated from `(seed, index)`, and held-out windows use `seed ^ RECALL_EVAL_SEED`.

## Base artifact

None. Every arm trains from step 0.

## Minimum test power

As claude-330's sheet, with 512 held-out queries per bucket: the 0.10 margin against the no-read twin is about 3.2 standard deviations near 0.5, and the paired 0.25 margin about 6.8.

## Cost

These are claude-334's measured times on one RTX 5090:
- about 6 minutes per 4,096 steps for an ask arm;
- about 2.5 minutes for a no-read arm;
- its three 16,384-step extensions took about 72 minutes per pod.

Per seed here:
- B1-a and B2-aq, about 15 minutes;
- three carried arms and B2-n at 16,384 steps, about 85 minutes with the `−C` passes;
- a build of about 10 minutes.

That is about 1.9 h per pod. Two pods in parallel, one per seed, at the $0.70/h ceiling: about $2.7 at most, inside the caps. A third seed adds one pod of the same.

## Results and stop rule

- **Results:** written to `results/claude/one-model-end-to-end-question-carry-2026-11-09/` in the HF store with a `SHA256SUMS`, and read back before any pod is brought down.
- **Stop** at an I-R1, I-R2, I-T or I-T0 failure, or at any non-finite loss.
- **Watchers:** one is armed on every pod (L-004), with the lease renewed by it.
- **Scripts:** pinned by SHA-256 in a D67 pre-run review section added before any pod.
- **Builds:** each checkout builds into its own cargo target, and the job checks the "Compiling … (<checkout>/crates/uor-r4-float-train)" line before trusting a test verdict (claude-330's fix, with claude-334's exemption for the pin's own checkout after I-0).

## D67 pre-run review (claude-335, 9 Nov 2026)

**Verdict: PASS-WITH-MINORS (launch)**, from a fresh read-only reviewer that read this sheet at the run pin, the six run scripts, their claude-334 templates, and the code. It found no blocker and no major. Both minors are applied:
- the B2 bar, the `−C` row and the undertrained test are recorded for every carried arm that trained, whatever stopped the run;
- a bar line requires the arm to have trained, not just to have exited.

The two field shapes it left unverified are settled:
- `hamming_pull_distance` has one entry per ask depth whatever the placement (`in_space_long.rs:13171`);
- `hamming_carry_gain` is written on every reading of a carried read (`:13231`).

t1's carried smoke shows both on every line, and the job's `trained_ok` accepts it.

- **The run pin:** `0ca41675a7eafe9d36c9213c5da9bbc2f8ebed25`. The head after it changes this sheet only.
- **The test pod t1** (vast-59852185, at `310e7d031`, $0.17):
  - EXIT 0;
  - I-1 MATCH at the base and the pin;
  - all 8 CUDA gates FAILS_AS_IT_MUST;
  - the B2-aqC smoke ok, with 8 ablations (`−C` last), ranks `plain`, `−aS`, and `c` on the question tokens rising from 0 to 0.012 in 32 steps.

  Sealed at `results/claude/one-model-end-to-end-question-carry-2026-11-09/tests/t1`:
  - `question-carry-tests.tar.gz`: `aa6fb48538a4d3037785fce85e6b908a0eacc2f1f99f4682a83ab6b025eaa309`
  - `question-carry-tests.list`: `4478db7372771b8681bdfaf367f40a72e938e289ed5bd61f719a3f50b396d6b9`
- **Before any arm trains, the run pods:**
  - run the strengthened I-C2 in I-0;
  - run its five gates at the run pin (`c-carry-no-a`, `-forward`, `-own-row`, `-far`, `-window`), all applying once (1+1) at the pin;
  - stop with exit 5 on any gate that does not fail as it must.

  `c-carry-own-row` now matches only `held_term`'s loop. The strengthened test added a second `for s in 1..=rows.min`.
- **Scripts** (`local/claude/claude-335/`, SHA-256):
  - `rjob-335.sh` `6ec70d39a4f4f1cd7c263fd295a83d64427e227e2d37a97b9ec13888758bb263`
  - `rchain-335.sh` `c8c384734b5e881608d59ca3063f3174649d3f82eaa91eef02ed838684400361`
  - `rseal-335.sh` `256c131905a9ead3e414f5619bf9b3539db38c55a172fd75fb39abe42e892961`
  - `mutate-335b.sh` `063838d685bb1916cde07d0f293f8cb1dfa8e2fa1b450c42db2b965be326cdf7`
  - `rseed-335.sh` `1fbc91bfd552c314ac11afdef57a90703c48185ee273dd9308d6740e00589208`
  - `rwatch-335.sh` `604a5f4350b7e96d0b3143464cf6274c6fed58a9406458da1199bc232865203a`
- **Cost:** the reviewer puts it at 2.3–2.9 h per pod with the gate rebuilds. That is inside the 3 h lease, which the watcher renews hourly, and the 4 h notice. Two pods at ≤ $0.70/h come to at most about $4.2, inside the caps.
- **What a pass and a fail unlock:**
  - A carried arm that passes with `−C` failing (branch 1) unlocks serving the carry at decode and export, then a language run.
  - Any other branch names its next sheet in the outcome map above.
  - An instrument fault (branch 0) repairs the instrument and reruns.
