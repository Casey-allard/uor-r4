# REPORT: integer-nearest-readout, claude-254 (Q5's near-tie grader)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md), as amended by this unit's [decision card](DECISION.md). References #3462.

## Tokens per second

No decode speed is claimed, and no served path changes. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1.

The grader reads traces offline. On the laptop (Apple M1), with the release build of `fbs-toks` at `768c39ff3`, after review round 1:

| Reading on F's DEV s1 base trace (621 MB, 308 windows) | Wall | Peak memory footprint (macOS) |
|---|---|---|
| `--decision-deciles` | 4.2 s | 1.39 GB |
| `--decision-check` with F's table | 3.8 s | 1.01 GB |
| `--decision-grade`, base against base, all 10,840 nearests | 13.6 s | 1.38 GB |

The first readings, at `bc02a2f3e`, took 3.6 s, 3.6 s and 11.9 s at the same peak memory footprint (`/usr/bin/time -l`'s "peak memory footprint"; its maximum resident set size differs between the runs).

## What did not work, first

- **Q5 is not read, and nothing here moves the item's done-when 3.** This unit builds the grader. The replay driver, the sheet with its FIT and pre-run review, and the GPU run that reads Q5 come next, in that order. The item stays `ready`.
- **On real data, no near-tie rule was exercised.** Every pass graded on a real trace either found no difference and reached the nearest with equal picks, or was refused as an instrument fault. So no real first difference was judged near or clear: bucket, code or pick. Those rules are pinned on synthetic traces only (the tests and gates below). Real ones come with the replay driver's passes.
- **The first build refused a float pick that is not ahead in f64, and that rule was wrong.** Review round 1 found it (two reviewers, the library slices).
  - The float writer picks on the GPU's f32 scores and records f64 recomputations of them. At a near tie the f64 order can reverse, so a float pick may score below its runner-up, or below the other side's pick, in f64.
  - The grader refused both cases on every side. Now only R's are refused: R picks the best of its integer scores exactly. A float side's negative margin is judged by the card's rule (both margins at most G₁₀), and it is always within G₁₀.
  - On DEV s1 it changed nothing: no F nearest was behind in f64, since the first check, which refused that on F, passed on the same trace. Every reading re-ran equal.
  - Pinned by gates g26 and g27; g15 now pins the refusal on R.
- **A NaN rate score became an infinite bucket distance in G₁₀.** `f64::min` drops a NaN, so `bucket_distance(NaN)` folds to +∞ and slipped past the deciles' "not a number" check. The deciles now refuse a rate score outside [0, 1], as the grader already did (gate g31). No real trace held one: the first deciles and the new ones are equal.
- **The review's other round-1 findings, all fixed:**
  - a test that never checked the reference's margin;
  - a code test where Q's coordinate had P's size, so a grader reading Q's value would pass;
  - no bucket case with Q's entry below P's;
  - no lane-order case;
  - substring checks too loose to name the refusal;
  - a replay that forced one place twice with two values was not refused;
  - the trace check and the deciles took a step or layer of the wrong shape;
  - the deciles file did not name its trace;
  - the CLI took an `--at` twice, and its side and offset checks were loose. Now a side with no table is refused by name, and an offset past the window before grading.
- **claude-253's one-forced window could not be read as the ORDER first wrote it.** Done-when 5 expected "the first difference is the bucket forced at step 49, entries 1 and 9, not a near tie".
  - claude-253 forced entry 9 where both windows' rate score (f32 bits 1031913096, about 0.0634) selects entry 1. That tested the forcing path. It is not a replay value: a replay forces the reference's own value.
  - So done-when 2's refusals apply, and the grader stops at that bucket as an instrument fault instead of judging a near tie:
    - without Φ: "the other side's entry 9 is not the 1 its score selects, and Φ does not force it";
    - with claude-253's Φ: "Φ forces entry 9, not the reference's 1 (I-Force)".
  - The ORDER's bullet was amended during the work, with this reason, to "no difference before step 49, and the first difference is the bucket forced at step 49, entries 1 and 9". Both parts hold: steps 40 to 48 are equal, and the refusal names that bucket and those entries.
- **R's first table check refused R's own trace, and the fault was the grader's.**
  - At position 41, step 41, R's Q12 query sits exactly on its pick's row, so the L1 sum is 0. The grader negated it as an f64: `-(0 as f64)` is −0.0. R's writer records the integer 0, which converts to +0.0. The two compare equal, but their bits differ, and the check compares bits.
  - Fixed at `bc02a2f3e`: R's score is negated as an integer before the conversion. The float sides were already right, since the grader computes `-4096.0 * sum` exactly as the float writer does (`in_space_long.rs`, `l1_score_r`).
  - Pinned by the trace-check test, with an R query on its pick's row, and by negative gate g19.
  - The synthetic tests had missed it because none of their queries sat exactly on a row.
- **main's serving audit fails, and the cause is in this item, from claude-249.** In `m5_full_path_cost_controls`, two tests disassemble `uor-r4-stack`'s serving call graph. Both find `fmov` (a SIMD-to-general transfer) in `TypeHeads::mean_by`, in `stack/hamming.rs`.
  - `mean_by` came with claude-249 (#3499). Its points already pass through `black_box` for the audit (claude-194), but the generated code still holds the `fmov`.
  - The same two tests fail at origin/main `245f47089` with the same finding, so this unit did not cause it. It does not change `hamming.rs`.
  - It is the next unit's work, before the replay driver: the audit guards integer serving (D11), and Q5's own trace instrument broke it.
- **R's codes are not sign-checked.** R's trace records no coordinates, so R's check covers buckets and nearests only (codes: 0). The card judges a code near tie on P's values, and P is F or T, never R. So R's codes are only compared bit for bit.
- **T was not read on real data.** The CLI's T table loader (R's `.uorlut` table, divided by 4096) and a T trace never ran in this unit; the library's tests run T as a float side on synthetic traces. The replay driver's unit should check a T trace against its table before the sheet.
- **`--decision-grade` stops the whole run at its first refusal.** That fits an instrument fault, which voids the run. But it means the three refused readings wrote no file, so their messages are kept verbatim in the store (`reading/README.txt`, and `reading-r1/refusals.txt`). The replay driver grades one pass per call.
- **The CLI's refusals have no unit test.** `fbs-toks` is a binary without tests. Two of its new refusals, an `--at` named twice and an offset past the window, were each run once by hand with the binary built at `768c39ff3`: each refused with exit 1 and wrote no file (`local/claude/claude-254/cli-refusals-r1.log`). The third, a side with no table, was not run.
- **Review round 2 left six minor points open,** carried to "Still open" below, not to a third round. The sharpest: a float pick behind in f64 is accepted however far behind, while f32 rounding explains only a small gap.
- **The one-forced reading used claude-253's window extracts,** not the whole one-forced trace. The base extract is the base trace's window 41: `jq -c` of each hashes to `22b6430408b493bc…`.
- **The crate's clippy is not clean,** but none of its findings is in this unit's files. Clippy finds 51 warnings in the library (plus a few in other targets), and 0 in `fbs_snapshot.rs` or `bin/fbs-toks.rs` (`local/claude/claude-254/clippy-r1.log`).
- **No sheet, FIT or pre-run review was written.** The readings are laptop checks of this unit's own instrument, outside Q5's set (the ORDER's done-when 5), and no claim rests on them.
- **The disk stays below the 30 GiB floor** (26 GiB free). This unit's copies of the base trace (621 MB, and its 208 MB gzip) are deleted at cleanup. The rest of the usage belongs to other labs or to Casey and is not touched.

## What worked

- **The prefix decision** ([DECISION.md](DECISION.md), option B): p's causal prefix gains the readout read's key codes at steps 1 to p − 1, and the key at step 0 leaves every prefix and G₁₀'s values. A fresh decision review approved it with nits, all fixed, and it is logged on #3462.
- **The library, in `fbs_snapshot.rs`:**
  - `decision_deciles`: G₁₀(φ, P) per family, the k-th smallest value with k = max(1, ⌊N/10⌋), zeros counted, step-0 keys left out. A bucket's value is its distance to the nearest boundary k/16; a code's is |v| per coordinate.
  - `grade_pass`: it walks p's causal prefix in the forward's order and reports the first difference, judged by the card's near-tie rules, or else the pick at p's nearest.
    - Inside the prefix, Φ must hold the reference's own values. Every forced decision must lie in the prefix, before or at the first difference.
    - A kept set or a memory lookup that differs ends the pass, never a near tie.
    - The pick is a near tie when both sides rank the other's pick within the sealed g10.
  - Its refusals as instrument faults:
    - two traces that do not line up (I-Align);
    - a forced decision the other side did not take, or one not the reference's value (I-Force);
    - a float code whose coordinates do not sign to it;
    - an entry its score does not select, or a rate score outside its range;
    - a table that does not reproduce a side's recorded pick and runner-up scores, bit for bit;
    - on R, a pick that is not its best. A float pick behind in f64 is judged by the card instead.
  - `replay_accounts`: a finished replay accounts for its step when every forced decision and the pick are near ties. A pass that forces a place an earlier pass forced is refused, whatever value either forced.
  - `check_decision_trace`: every step's layer count, every layer's entries and scores, every entry against its score, every float code against its coordinates (step-0 keys included), every nearest against its own table (on R, the pick ahead).
  - The deciles refuse a trace with no layers or lanes, a step without the trace's layers, and a rate score outside [0, 1].
- **The CLI, in `fbs-toks`:**
  - `--decision-deciles TRACE OUT`, whose file names its trace by path and sha256;
  - `--decision-check TRACE --table TABLE --out OUT`;
  - `--decision-grade REFERENCE OTHER --deciles … --g10 G --reference-table … --other-table … [--force …] [--at P:O]… --out …`, which writes `uor-r4.decision-grade/1` new. Without `--at`, it grades every nearest of the other side's windows. The record copies the deciles' trace, which may be the whole cell's trace while REFERENCE is one window of it.
- **The readings of done-when 5** (below) hold, with the second bullet as amended, at both heads.
- **32 negative gates.** Each sets one rule aside, and the test that pins it fails. The restored tree passes, and the gates leave it clean.
- **The in-unit reviews** (`local/claude/claude-254/reviews.md`), each by a fresh reviewer:
  - round 1, five reviewers (the CLI, two library slices and two test slices): four APPROVE WITH NITS and one REQUEST CHANGES, every finding fixed at `768c39ff3`;
  - round 2, on the fix diff: APPROVE WITH NITS, six minor points left open;
  - claims against evidence, on this REPORT: APPROVE WITH NITS, no false claim, its wording and logging points fixed.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-integer/src/fbs_snapshot.rs` | `decision_deciles`, `bucket_distance`, `DecisionSource`, `NearestTable` (Q12 and f32, with the bit-exact score check), `GradeSide`, `PassOutcome`, the prefix walk, `grade_pass`, `replay_accounts`, `check_decision_trace`; `CodeRole` gains `PartialOrd` and `Ord`. 12 new tests |
| `crates/uor-r4-integer/src/bin/fbs-toks.rs` | `--decision-deciles`, `--decision-check` and `--decision-grade`; the table loaders for F (the snapshot's `embed`), R (the `.uorlut` table at Q12) and T (the same table divided by 4096), and a refusal for any other side |
| `docs/loop/tasks/integer-nearest-readout/claude-254/DECISION.md` | the prefix card |
| `docs/loop/tasks/integer-nearest-readout/claude-254/ORDER.md` | done-when 5's one-forced bullet, amended with its reason |
| `docs/roadmap/items/integer-nearest-readout.md` | the grader's bullet; the next step is the replay driver |
| `docs/research/notebook/m4-serving.md` | this unit's row |

## Measurements

All on traces outside Q5's set, on the laptop, at $0. They ran first with `fbs-toks` at `bc02a2f3e` (sha256 `2260599e…`), then again after review round 1 at `768c39ff3` (sha256 `c8db382a…`), by `local/claude/claude-254/readings-r1.sh`. Every count, G₁₀ and graded pass below is equal at both heads; the second deciles file also names its trace.

- **The inputs:**
  - F's DEV s1 base trace is claude-253's store file `run/base/r-drop-s1/decision-trace-dev.json.gz`. Gunzipped, it is 621,282,215 bytes, with the sha256 claude-253's `SHA256SUMS` lists (`9e7c9bf3…`, compared by `cmp`).
  - F's table is claude-243's `snapshot-s1.json`.
  - R's DEV s1 trace is claude-249's `smoke/trace-dev-s1.json` (`11e6b7d9…`). R's table is claude-243's `a7r-s1.uorlut` (`f86a7856…`).
- **Table checks:**
  - **F with F's table:** 308 windows, 36,541 steps, 14,031,744 buckets, 584,656 codes and 10,840 nearests. The codes are 36,541 steps × 2 reads × 4 heads × 2 roles. That is claude-253's 582,192 forced codes plus the 2,464 step-0 keys (308 × 2 reads × 4 heads), which no forcing names.
  - **R with R's table:** 308 windows, 17,491 steps, 6,716,544 buckets and 4,382 nearests.
- **G₁₀ on F's base trace**, 14 families:

  | Family | N | k | G₁₀ |
  |---|---|---|---|
  | code, depth 2, copy, query | 4,677,248 | 467,724 | 0.03210 |
  | code, depth 2, copy, key | 4,637,824 | 463,782 | 0.03208 |
  | code, depth 2, type, query | 4,677,248 | 467,724 | 0.3605 |
  | code, depth 2, type, key | 4,637,824 | 463,782 | 0.01648 |
  | code, depth 6, copy, query | 4,677,248 | 467,724 | 0.03102 |
  | code, depth 6, copy, key | 4,637,824 | 463,782 | 0.03085 |
  | code, depth 6, type, query | 4,677,248 | 467,724 | 0.1745 |
  | code, depth 6, type, key | 4,637,824 | 463,782 | 0.01548 |
  | bucket, layer 0 | 2,338,624 | 233,862 | 0.005525 |
  | bucket, layer 1 | 2,338,624 | 233,862 | 0.006144 |
  | bucket, layer 2 | 2,338,624 | 233,862 | 0.009363 |
  | bucket, layer 3 | 2,338,624 | 233,862 | 0.008864 |
  | bucket, layer 4 | 2,338,624 | 233,862 | 0.006779 |
  | bucket, layer 5 | 2,338,624 | 233,862 | 0.007550 |

  - A bucket family's N is 36,541 steps × 64 lanes. A query family's N is 36,541 steps × 2 heads × 64 coordinates.
  - Each key family is smaller by 39,424 = 308 windows × 2 heads × 64 coordinates: the step-0 keys, and nothing else.
- **F's base against itself** (g10 30,228, every nearest step): 10,840 passes, all reach the nearest with equal picks.
- **claude-253's one-forced window 41 against the base's:**
  - offsets 0 to 8 (steps 40 to 48), no Φ: 9 passes, all reach the nearest with equal picks;
  - offset 9 (step 49), no Φ: refused at step 49, layer 0, lane 0, "the other side's entry 9 is not the 1 its score selects, and Φ does not force it";
  - offset 9, with claude-253's `force-one.json`: refused at the same bucket, "Φ forces entry 9, not the reference's 1 (I-Force)";
  - the one-forced window's own table check: refused at the same bucket, "entry 9 is not the 1 its score selects".
- **Saved** in the store under `results/claude/integer-nearest-readout-grade-pass-2026-11-02/`:
  - `reading/`, the first readings: every output, the wrapped window traces, the timings, a `README.txt` with the refusals verbatim, and a `SHA256SUMS` of 12 files;
  - `reading-r1/`, the readings at `768c39ff3`: every output, the timings, `refusals.txt`, the script that ran them, a `README.txt` and a `SHA256SUMS` of 11 files.
  - Each store `SHA256SUMS` reads back equal to the laptop's. The base trace is not saved again; claude-253's store copy stands.

## Checks run at the head

The code's last change is `768c39ff3`, review round 1's fixes. After it, the branch adds documents (this REPORT, the ORDER's amendment, the item's bullet and the notebook row) and a merge of main's `454a90031`, which touches only DeepSeek's `uor-r4-cpu-train` crate, its documents and one `Cargo.lock` line, nothing in `uor-r4-integer`. The module tests, the gates, the host suite and the loop gates ran at `768c39ff3`, and clippy and fmt again at the merge `4edc6e841`. The same checks at `bc02a2f3e` gave the same results.

| Gate | Command | Result |
|---|---|---|
| Module tests | `cargo test -p uor-r4-integer --lib fbs_snapshot` | 20 passed, 0 failed (`gates/restored.log`) |
| Host suite | `cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast` | 629 passed, 38 failed, 4 ignored, in 51 targets (`local/claude/claude-254/host-suite-r1.log`). The 9 targets that hold the 38 were re-run at origin/main `245f47089`, in a detached worktree with its own target dir (`base-suite.log`; the worktree's commit was read from `git worktree list` before it was removed, and the log does not record it). There the same 38 tests fail: the sorted lists of failed tests are identical. 35 read local bundles or `tokenizer.json`, which this checkout does not hold. m7's one is its browser parity check, which fails while compiling to Wasm. m5's two are the serving audit, below |
| Clippy | `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets` | 0 findings in `fbs_snapshot.rs` or `bin/fbs-toks.rs` (`clippy-r1.log`) |
| Formatting | `cargo fmt -p uor-r4-integer -- --check` | exit 0: the whole crate formats clean (`fmt-r1.log`) |
| Loop gates | `uor-harness loop gates --full` | 11 green, 0 red (`loop-gates-full-r1.log`) |

### Negative gates

`local/claude/claude-254/gates.sh`, with each gate's patch and log beside it under `gates/`, at `768c39ff3`, with `fbs_snapshot.rs` clean. Each gate finds its line by the line's own text, sets one rule aside with `sed`, saves the patch, runs the one test that pins it, reverses the patch and checks the file is clean. Every gate failed inside its test, not at the build: at an `assert!`, or for g09 and g31 at the `expect_err` or `unwrap_err` that takes a refusal. The restored tree then passed all 20 module tests, and the tree was clean after the gates (`gates.out`).

| Gate | Rule set aside | Test | Result |
|---|---|---|---|
| g01 | Step-0 keys left out of G₁₀ | `deciles_take_the_kth_smallest_value_with_zeros_and_without_step_0_keys` | FAILED |
| g02 | k = ⌊N/10⌋ (set to N/5) | the same | FAILED |
| g03 | The table reproduces the recorded scores | `grade_pass_refuses_what_does_not_line_up_or_check` | FAILED |
| g04 | A code signs to its coordinates | the same | FAILED |
| g05 | Φ holds the reference's value | `phi_holds_the_references_values_inside_the_prefix` | FAILED |
| g06 | The other side's entry is the one its score selects | the same | FAILED |
| g07 | A bucket near tie needs adjacent entries | `a_bucket_is_a_near_tie_only_next_to_its_boundary` | FAILED |
| g08 | A code's value is its largest differing coordinate (set to the smallest) | `a_code_is_a_near_tie_only_when_every_differing_bit_is_near_zero` | FAILED |
| g09 | Every forced decision is met in the prefix | `phi_holds_the_references_values_inside_the_prefix` | FAILED |
| g10 | No key at step 0 in the prefix | `the_prefix_holds_the_readouts_earlier_keys_and_no_step_0_key` | FAILED |
| g11 | The readout's earlier keys are in the prefix | the same | FAILED |
| g12 | The readout's earlier queries are not | the same | FAILED |
| g13 | The readout's kept set at p is compared | the same | FAILED |
| g14 | A memory lookup is compared | `kept_sets_and_memory_lookups_end_the_pass` | FAILED |
| g15 | R's pick is its best | `grade_pass_refuses_what_does_not_line_up_or_check` | FAILED |
| g16 | The pick's near tie needs the other side's margin | `the_pick_is_a_near_tie_only_when_both_sides_rank_it_within_g10` | FAILED |
| g17 | A replay accounts only when every forced decision is a near tie | `replay_accounts_reads_a_finished_replay` | FAILED |
| g18 | A stream read's kept set is compared | `kept_sets_and_memory_lookups_end_the_pass` | FAILED |
| g19 | R's zero score is 0, not −0 | `the_trace_check_refuses_what_its_values_do_not_make` | FAILED |
| g20 | A code's value is P's coordinate (set to Q's) | `a_code_is_a_near_tie_only_when_every_differing_bit_is_near_zero` | FAILED |
| g21 | A bucket's boundary is the larger entry's, either way (set to P's entry + 1) | `a_bucket_is_a_near_tie_only_next_to_its_boundary` | FAILED |
| g22 | Lanes are walked in order (reversed) | `the_walk_follows_the_forwards_order` | FAILED |
| g23 | Zeros count in G₁₀ (filtered out) | `deciles_take_the_kth_smallest_value_with_zeros_and_without_step_0_keys` | FAILED |
| g24 | The pick's near tie needs the reference's margin | `the_pick_is_a_near_tie_only_when_both_sides_rank_it_within_g10` | FAILED |
| g25 | A repeated decision is its place, whatever value was forced (set to place and value) | `replay_accounts_reads_a_finished_replay` | FAILED |
| g26 | A float pick behind in f64 is not refused (set to refused) | `the_trace_check_refuses_what_its_values_do_not_make` | FAILED |
| g27 | A float side's negative margin is judged (set to refused) | `the_pick_is_a_near_tie_only_when_both_sides_rank_it_within_g10` | FAILED |
| g28 | The trace check refuses a step without the trace's layers | `the_trace_check_refuses_what_its_values_do_not_make` | FAILED |
| g29 | The deciles refuse a step without the trace's layers | `deciles_take_the_kth_smallest_value_with_zeros_and_without_step_0_keys` | FAILED |
| g30 | A trace has layers and lanes | the same | FAILED |
| g31 | The deciles refuse a rate score outside [0, 1] | the same | FAILED |
| g32 | The trace check refuses a layer without its lanes' entries and scores | `the_trace_check_refuses_what_its_values_do_not_make` | FAILED |

g01 to g18 first ran at `21b82f399`, and g01 to g19 at `bc02a2f3e`, with the same results. g15 then pinned the refusal on every side; it now pins R's alone.

## Skills used

None.

## Still open

- **First, repair the serving audit** that `TypeHeads::mean_by` (claude-249) fails on main, in `m5_full_path_cost_controls`.
- **Next, under the same card: the replay driver.**
  - It forces the first differing code or bucket in forward order, structure (F, T) and arithmetic (T, R), and grades each pass with `--decision-grade`.
  - It needs a float mode that runs only the windows it names, since each float pass is now a whole `score_rows` run (204 s on F's s1).
- **Check a T trace against its table** before the sheet. The CLI's T loader has not run on real data.
- **Review round 2's open points, for the replay driver's unit,** which changes the grader next:
  - bound how far a float pick may fall behind in f64 (an f32 rounding bound), and refuse beyond it;
  - on R, refuse a pick that ties the other side's but has the larger id (R takes the smaller);
  - test that a negative margin on the reference side is accepted (the gate: refuse it, and a test should fail);
  - test the CLI's refusals (an `--at` named twice, a side with no table, an offset past the window);
  - correct `grade_pass`'s doc: the reference side's entries and codes are checked even where forced;
  - consider refusing a deciles file that names no trace.
- **The sheet** registers claude-247's rule with this unit's amendment verbatim, with a fresh FIT and a pre-run review. It cites the code that forms each side's turn-memory key (this unit's card). Then the GPU run reads Q5.

## Plain summary

To find out why the integer model and the trained model sometimes pick different words, claude-247's rule replays each step where they part, one forced choice at a time. Whether each replayed difference is just a near tie is then judged by fixed thresholds. This unit builds that judge.

On real traces, the judge agrees that the trained model's recorded run matches itself: all 10,840 word picks, in about 13 seconds. Both models' records check against their own lookup tables. Fed a run in which a choice was forced to a value nothing would have chosen, it stops at exactly that choice and calls it a faulty input, not a near tie. On its first real run the judge also found a bug of its own: it scored an exact zero as "−0". That is fixed, and a test now catches it.

Review then found one rule too strict: the trained model picks with the graphics card's rounding, so at a near tie its recorded order can flip when recomputed more precisely. The judge now treats that as the near tie it is, and refuses an out-of-order pick only from the integer model, which has no rounding. The real readings did not change.

The next unit uses the judge to grade the actual replays.
