# REPORT: one-model-end-to-end, claude-328

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-query-recall-2026-11-08.md](../../../harness/prep/one-model-end-to-end-query-recall-2026-11-08.md) (FIT, round 4; D67 pre-run review PASS).

## What did not work, first

- **The trained Hamming read does not choose a value by a query that is not its key.** On both seeds, B1-h and B2-h answer the asked value on at most 2 of 512 queries in any far bucket (`R_ask` ≤ 0.004, against the bar's 0.50). The no-read twins score the same. Outcome map: **outcome 4, a query failure** (below). This closes the configuration (type-head rotation query, read 4, width 256) on this task; it does not close the Hamming read (§4.1).
- **With the read on, B1 and B2 learn nothing at all.** The training loss of B1-h equals B1-n's to three decimals at every checkpoint (1.0021 against 1.0021 at step 4,096, seed 1), and B2-h's equals B2-n's (1.0001 against 1.0003). `R_any`, any of the window's values, stays at 0.004–0.016 for the read arms at step 4,096 (at most 0.020 over every arm and checkpoint), so the read does not even copy some other value. The read's combine weight grows only to 0.075–0.104, against B0-h's 0.66–0.70.
- **Two mutation gates did not fire on the pod** (`gates.txt`, both seeds). Under the ORDER's done-when 2, each needs naming:
  - **`i4-window-offset` is an equivalent mutation.** It changes `previous[r]` only at a window's first row (`r % time == 0` becomes `r == 0`, `flock_read_hamming.rs:1161`). That row's copy key is never read, because position 0 is never a candidate: both of the read's choice calls pass `first = 1` (`flock_read_hamming.rs:1330`, `:1332`), and `previous` feeds only the copy key. No test can fail it. The gate targeted a dead line. A live window-offset mutation was not written in this unit.
  - **`i7-m1-class-mask` is not equivalent, and the pod test could not see it.** The mutation stops the readings' guard from clearing the class mask and runs the ablations in reverse, so `−c2`'s mask is left on for training. On this configuration `−c2` reads within one hit of the plain read (B0-h 1020 / 1017 both ways at seed 1; 1018 / 1004 against 1018 / 1003 at seed 2), so a leaked depth-2 copy mask changes almost no pick, and the bit-identical loss check did not see it. **Fixed in this PR:** a laptop test, `the_ablation_guard_clears_every_mask_and_cut`, drops the guard after each of the five ablations' masks and a row cut, then checks both are clear. Both guard mutations fail it (`local/claude/claude-328/gt/mut-i7.txt`, 2 of 2). The pod test still cannot see a `−c2` leak.
- **A2: the question does not steer which value the served models rank higher at the first reply position.** Over 1,536 pairs (six artifacts × two renderings × 128), the same value is ranked higher in both cases of the pair 1,415 times, whichever value is asked. In 62 pairs both cases rank the asked value higher, and in 59 both rank the other. The asked value's first token is never top-1; its median rank is 1,007–2,430 of 4,096. That position is a frame word (claude-327), so this reading is bounded: it measures the value order beneath the frame word, not a choice the model makes.
- **Deviations from the sheet, named:**
  - I-7's first mutation is compound (guard and order together), as the sheet's round-4 minor asked. It was blind on the pod, as above.
  - I-4's step-0 check compares the picks bit for bit, not the points or the loss.
  - I-4's gates mutate the module, not one function each.
  - **The sheet's third I-4 gate was never written.** The sheet names it "picks that lose their window's offset", a fault in pick indexing. The gate as written mutates how the copy key's previous token is built, which is the equivalent mutation above. A fault in pick indexing across windows has no gate; the sheet's gate is still owed. B0 at 0.98 or above makes such a fault unlikely to touch this result.
  - The gate runner records a gate that does not fire and goes on; it does not stop the run. Both unfired gates were read after the run, not before it.
  - Seed 3 and outcome 1's 16,384-step extension were not scripted, as the sheet says. Neither was needed: the seeds agree, and no arm rose.
  - Before the pod, the CUDA paths were only checked with a stub nvcc on the laptop. They were built with nvcc on the pod.

## What changed

- `crates/uor-r4-float-train`: the two-class (`--recall-query 1`) and header (`--recall-header 4`) recall forms, both default off; the held-out window count; the outcome counts `R_ask`, `R_oth` and `R_any` per bucket and tail slot; the Hamming trace at the query and scored positions, with weights; the five eval-only ablations (`−c2`, `−c6`, `−c`, `−t`, `−cQ`); tests I-1 to I-5 and I-7, plus the laptop guard test.
- `docs/labs/one-model-end-to-end/instrument`: `served-reply-panel --paired-first 1` (A2), which needs `--teacher-forced 1`. It ranks the pair's other value's first token at the first reply position. The report without the switch is unchanged.
- The sheet: FIT rounds 3 and 4, and the D67 pre-run review.
- This REPORT and one notebook row in `docs/research/notebook/float-training.md`.
- No bar, status, scope or cap changed.

## Measurements

**The run.**
- Two Vast RTX 5090 pods, one per seed: `vast-59789826` (seed 1) and `vast-59789855` (seed 2). Six arms each, 4,096 steps, batch 16. All 12 arms exited 0.
- The crate's code was at `2904f2f1e` (the D67 pin). The later commits before this REPORT touch only `served-reply-panel`.
- Results were sealed to the store at `results/claude/one-model-end-to-end-query-recall-2026-11-08/runs/s{1,2}/` and read back; `SHA256SUMS` OK.
- About 38 minutes and $0.40 per pod, **$0.80 in all**. Both pods are down.

**Instrument checks on the pod:**
- **I-4:** the step-0 picks are the same with the read on and off in B0, B1 and B2. The read's weight is off zero at step 512 (B0 0.206, B1 0.022, B2 0.038).
- **I-6:** ok on all six arms.
- **Tests:** 424 lib tests pass.
- **I-7:** the pod test is ok.
- **Gates:** `i7-m2-row-cut`, `i5-readout-index`, `i4-margin-sign` and `i4-keeps-itself` fail as they must.

**B0 (the positive control) passes on both seeds.** The rates are hits of 1,024 per far bucket, 33–96 / 97–180:

| Seed | B0-h | B0-n | `−c` | `−t` | `−c6` | `−c2` | `−cQ` |
|---|---|---|---|---|---|---|---|
| 1 | 0.996 / 0.993 | 0.003 / 0.003 | 0 / 0 | 0.994 / 0.992 | 0 / 0 | 0.996 / 0.993 | 0 / 0 |
| 2 | 0.994 / 0.980 | 0.005 / 0.003 | 0 / 0 | 0.990 / 0.977 | 0 / 0 | 0.994 / 0.980 | 0 / 0 |

Every B0 control passes (`−c` ≤ 0.10, `−t` ≥ 0.95, `max(−c2, −c6)` ≥ 0.90, `−cQ` ≤ 0.10). The depth-6 copy heads carry B0; the depth-2 copy heads carry nothing on this configuration.

**B1 and B2 at step 4,096.** The rates are `R_ask` / `R_oth` / `R_any`, of 512 queries per far bucket:

| Arm | Seed 1, 33–96 | Seed 1, 97–180 | Seed 2, 33–96 | Seed 2, 97–180 | Loss (s1 / s2) |
|---|---|---|---|---|---|
| B1-h | 0.002 / 0 / 0.010 | 0.002 / 0.002 / 0.010 | 0.002 / 0 / 0.008 | 0 / 0.006 / 0.004 | 1.0021 / 1.0023 |
| B1-n | 0.002 / 0.002 / 0.018 | 0 / 0.002 / 0.010 | 0.002 / 0 / 0.016 | 0.002 / 0 / 0.008 | 1.0021 / 1.0025 |
| B2-h | 0.004 / 0.002 / 0.006 | 0 / 0 / 0.010 | 0.002 / 0 / 0.012 | 0.002 / 0.006 / 0.016 | 1.0001 / 1.0001 |
| B2-n | 0 / 0 / 0.008 | 0.004 / 0.002 / 0.016 | 0.010 / 0 / 0.014 | 0.002 / 0.002 / 0.010 | 1.0003 / 1.0002 |

B2-h's slot-1 `R_ask` is at most 0.008 on both seeds. B0-h's loss is 0.738 (s1) and 0.706 (s2).

**The trace, 64 cells a seed.** No cell is kept: no cell has `kept_ask` ≥ 0.5 and ≥ `kept_oth` + 0.25.

| Arm, seed | Type heads: max `kept_ask` / max `kept_oth` / mean chance | Copy heads |
|---|---|---|
| B1-h s1 | 0.104 / 0.107 / 0.086 | keep neither value (0 / 0) |
| B2-h s1 | 0.127 / 0.121 / 0.090 | 0 / 0 |
| B1-h s2 | 0.084 / 0.088 / 0.086 | 0 / 0 |
| B2-h s2 | 0.135 / 0.131 / 0.090 | 0 / 0 |

The type heads keep the asked and the other value's positions at the same rate, near chance. The copy heads keep neither, as the task's design requires: `Q(K_j)` never occurred earlier in the window, so a current-token match finds nothing.

**Reading against the outcome map, in order:**
- **0 (B0 fails):** no; B0 passes.
- **1 (undertrained):** no. No arm's `R_ask` rose more than 0.05 between steps 3,584 and 4,096; every change is within ±0.004.
- **2 (B2 passes):** no.
- **Header interference:** no. `R^{−c2}(B2-h)` and `R^{−c6}(B2-h)` do not reach `R_ask(B2-h)` + 0.10; every ablated `R_ask` is at most 0.004.
- **3 (B1 passes):** no.
- **4:** the first branch fails: no depth-2 copy or type head keeps the asked position at B2-h's `Q(K_j)`. The second branch holds: no type head keeps it at either position, and slot-1 `R_ask` is at most 0.008 < 0.50. **A query failure**, mechanism-side for this configuration (§4.1, §4.2).

**Instrument or mechanism (L-007):** read as mechanism-side for the configuration because:
- B0, on the same code path, scores 0.98–0.996 with every control in range;
- the trace's picks equal the plain read's picks (the readings' own check);
- the asked position is among the type heads' candidates (their `kept_ask` is above 0, near the chance rate);
- I-4 shows the read's weight trains off zero in B1 and B2.

What fails is the learning: the type heads' rotation query never moves off chance, so the read's output carries noise, and its combine weight stays small. Only the no-read-equal loss says anything about why. The sheet does not test a cold-start reading (the type heads get no useful gradient while their picks are random). That reading is a hypothesis for lever (b), not a result.

**A2.** All on the laptop. `served-reply-panel --teacher-forced 1 --paired-first 1` was built at `1ee44639f` and run on claude-322's, claude-323's and claude-325's served artifacts, two seeds each, under both renderings, with claude-309's panel inputs. Outputs are in `local/claude/claude-328/a2/`, with `SHA256SUMS`.

| Artifact | one-user-turn: both-asked / both-other / fixed | per-turn: both-asked / both-other / fixed | asked rank, median (one / per) |
|---|---|---|---|
| claude-322 s1 | 2 / 2 / 124 | 10 / 10 / 108 | 1,315 / 1,007 |
| claude-322 s2 | 3 / 3 / 122 | 8 / 9 / 111 | 2,032 / 2,316 |
| claude-323 s1 | 3 / 3 / 122 | 13 / 10 / 105 | 1,333 / 1,287 |
| claude-323 s2 | 0 / 0 / 128 | 6 / 5 / 117 | 2,430 / 2,041 |
| claude-325 s1 | 1 / 1 / 126 | 7 / 8 / 113 | 2,361 / 1,964 |
| claude-325 s2 | 4 / 4 / 120 | 5 / 4 / 119 | 1,492 / 1,087 |

- No pair has identical ranks (distinct first tokens are never tied).
- The A2 code has 10 mutation gates, all caught (`gt/mut-a2.txt`). Its fresh review (round 1) found one major, `paired_first_row` untested end to end, and two minors. All three were fixed at `1ee44639f`.

**Reviews:**
- FIT rounds 3 (NOT FIT) and 4 (FIT) are recorded in the sheet.
- The D67 pre-run review: PASS.
- The fresh exact-head review of this PR is recorded with `loop review`.

## Checks run at the head

- `cargo test --lib` of `uor-r4-float-train` on the laptop (CPU tests; the CUDA tests ran on the pod at `2904f2f1e`).
- `cargo test --features cpu-parity --bin served-reply-panel` with the pinned tokenizer: 20 passed. The root workspace and the instrument's share `local/target` and left a stale `uor-r4-integer` build twice (`could not find stack`), so each passing run followed `cargo clean -p uor-r4-integer`.
- **Coverage limits the exact-head review named (minors, not fixed):**
  - The A2 end-to-end test skips when `UOR_PANEL_TOKENIZER` is unset, and no workflow sets it, so CI does not pin it.
  - The laptop guard test pins what `UndoAblation`'s drop clears, not that the readings hold the guard across each ablation.
- The mutation gates: `gt/mut-a2.sh` (10 of 10) and `gt/mut-i7.sh` (2 of 2); the pod gates are in `gates.txt` (4 fire, 2 named above).
- `rustfmt --check` on `served-reply-panel.rs` (clean). `in_space_long.rs` is not rustfmt-clean on `main` (442 diff hunks), so the new tests are hand-formatted and the file was not reformatted.
- `loop report --check`; `loop pr-check --local`.
- The fresh exact-head review is posted with `loop review` before the queue.

## Skills used

- None beyond the loop itself: ORDER, FIT rounds, the D67 pre-run review, the A2 slice review and the fresh exact-head review.

## Still open

- **Lever (b) of claude-326's assessment, a question-keyed read,** under its own sheet and FIT. It must pass B2 here before any language run on this line. claude-326's lab decision stands: no one-knob language run until a read passes B2.
- **A live window-offset mutation for I-4**, replacing the equivalent one, and an I-7 pod configuration where `−c2` matters.
- **The Hamming trace at the first reply position** on the served artifacts (from claude-327).

## Plain summary

We tested whether the model's memory-lookup part can learn to fetch a fact when the question is worded differently from how the fact was stored. In the simple case, where the question repeats the fact's own key, it fetches the right fact 98–99.6% of the time. When the question uses a different word for the same key, it gets the right fact essentially never (under 1%). Turning the lookup off entirely makes no difference to that number. So this version of the lookup cannot connect a question to its fact. The next step is a lookup built to be keyed by the question itself. Separately, the chat models trained earlier do not change which fact they lean towards when the question changes. The two pod runs cost $0.80.
