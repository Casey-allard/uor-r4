# REPORT: one-model-end-to-end, claude-395 (the first native training of the read's gates, on real states)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #4017. Sheet: [`one-model-end-to-end-native-read-train-2026-11-15`](../../../../harness/prep/one-model-end-to-end-native-read-train-2026-11-15.md).

**Outcome map reading: 6, "not the targets", at both ext states.** The pass is withheld.
- The native integer rule trains the read's gates from identity, on the GPU, and the A-ext arm meets bars 1, 2, 3 and 5 at both ext states.
- The other-fact control C2′ breaks bar 4. Outcomes are checked in the order 0, 5, 6, …, so outcome 6 is the result.

## What did not work, first

- **C2′ breaks bar 4 at both ext states.** Other-fact targets should raise `chose_other` by at least 0.10 over C0, and should not raise `chose_target` by more than 0.05.

  | State | `chose_other` rise, b1 / b2 | `chose_target` rise, b1 / b2 | Bar 4 |
  |---|---|---|---|
  | ext-s1 (shift 8) | +0.096 / +0.105 (3/3 → 52/57) | **+0.123 / +0.107** (10/12 → 73/67) | broken |
  | ext-s2 (shift 4) | **+0.088** / +0.123 (10/8 → 55/71) | falls (195/213 → 70/69) | broken |

- **The rule barely learns to pick the other fact.** C2′'s train `chose_other` reaches about 0.12 (500 and 548 of about 4,096 train rows a bucket at ext-s1), against about 0.98 for the asked fact in A-ext.
  - At ext-s2, C2′ accepts 25 steps, all within the first 64, and then never again.
  - C2 (random targets) accepts 14 steps at ext-s2, and none after step 64.
- **Shift 4 never accepts at ext-s1.** The loss stays at 267,327,455,773 for all 512 steps. Shift 8 is that state's chosen shift. Outcome 5 needs no accepted step at every shift, so it does not hold.
- **The 4k states decode badly.** Even where the native pick improves, the ×16 hard copy answers right on at most 35 of 512 rows: the trunk's decode at 4,096 steps does not carry the copy. A-4k is descriptive, as the sheet says.
- **A-4k is not monotone.** At 4k-s1, shift 4 peaks at 347/364 (step 128) and ends at 276/282 (step 512). The strict-drop accept is per batch, so held-out counts can fall while each step lowers its own batch's loss.
- **The shift was chosen on the last batch's loss, not the full train split's.** The trainer's `train_loss` is the loss of the step's own 64-window batch (the exact-head review's finding 2), and the sheet says "final train loss".
  - At the ext states the choice is the same on the full split: the chosen shift also has the highest train `chose_target` (ext-s1: s8 4,004/4,018 against s12 3,547/3,576 and s4 73/79; ext-s2: s4 4,091/4,095 against s8 3,989/3,994). So the bar's reading stands.
  - At the 4k states the full split would choose **shift 8**: 2,861/2,889 against s4's 2,292/2,325 at 4k-s1, and 2,922/2,916 against 2,732/2,706 at 4k-s2. A-4k is descriptive. At shift 8 its gain over C1 is +0.209/+0.271 (4k-s1) and +0.225/+0.232 (4k-s2).
- For C2 and C2′, the train split's `chose_target` counts the run's own targets (random rows, or the other fact), not the asked fact. The C2′ train `chose_other` above is therefore the same count. Bar 4 reads only the held-out split, where the asked and other files are always separate.
- The gain shift stays at the dump's `g = 0`. It enters neither the hinge nor the picks, and `R_ask` comes from the float hard copy at ×16, so nothing read here depends on it. The sheet's `g = 4` is not written into the saved reads.

## What changed

Three tools, each with laptop tests, built with CUDA and tested on wick (RTX 4090, `compute_89`):
1. **TRAIN dumps with targets.** `recall-payload-probe --native-train N`: the native replay and its dump read the first `N` TRAIN windows at the run's own seed. Beside every dump, held-out ones included, it writes:
   - `.ask.u32` and `.other.u32`, the asked and other value's window position `p` at each query's scored row (pick `s = p − 1` copies `ids[p]`), 0 elsewhere;
   - `.random.u32`, C2's seeded earlier row (seed 393, from window and slot only);
   - `.bucket.u8`, the bucket + 1.

   The writer refuses a duplicate row, an offset of 0 or ≥ 192, and a target that is not earlier. A test that moves the rows by one fails, as it should.
2. **The trainer.** The `uor-r4-discrete` example `native_read_train` (claude-395a's builder, commit `f1f71a630`):
   - the device step `read_train_pick_step` over 64-window batches in index order;
   - JSON-line readings per far bucket on both splits;
   - the saved read (`to_bytes`) and the eval picks;
   - `--check-host-steps` against the Rust oracle.

   The pre-run review added `--start dump` (the dump's own converted gates, for C1 and C3) and a readings file that is never written over.
3. **The injected-pick decode.** `recall-payload-probe --native-picks FILE` uses `vector_reads::with_injected_first_picks`: in every pass with the read, Flock's first ranked column is replaced by the native picks. The hard copy then adds `w ⊙` that pick's next-token row, decoded by the float trunk (diagnostic code, D85). The forward refuses a count other than its rows, a pick that is not earlier, and running without the hard copy. A CUDA test pins the added row.

The exact-head review of the trainer added two refusals after the run, in a later commit than the binary that ran. A saved read or picks file is never written over, and the paths are checked before training. An eval dump whose read differs from the train dump's is refused, because it is not the same state. Job B gave every run new paths and paired each state with itself, so no reading changes.

The pre-run review (fresh context, §5.10) returned GO WITH FIXES. The target convention checks out against `RecallQuery` and the read's `ids[s + 1]` copy. The two fixes are in commit `f05f35a8a`, and I read the part it left unread (`bucket_counts`) myself.

## Measurements

**Outcome 0, the instrument checks: all pass.**
- All eight state sha256 match claude-389's ORDER.
- C1 (the dump's converted gates, step 0) per bucket against the float first pick:

  | State | C1 `chose_target` = float first right | Native vs float picks |
  |---|---|---|
  | ext-s1 | 504 / 496 | 1,024 of 1,024 |
  | ext-s2 | 496 / 494 | 1,024 of 1,024 |
  | 4k-s1 | 213 / 205 | 1,024 of 1,024 |
  | 4k-s2 | 227 / 247 | 1,024 of 1,024 |

- C1's injected-pick ×16 decode equals the float hard copy's **row for row** at every state: 504/496 = 0.984/0.969 at ext-s1 and 496/494 = 0.969/0.965 at ext-s2, which are claude-388's counts.
- Host equals device on the first two steps of every run, and no train target was refused.
- The replay of A-ext s1 is byte-identical: read `b05b544a…`, picks and readings file.

**The bar at step 512, held-out, 512 rows a bucket.** Shifts were chosen by the last batch's train loss (see the first section): ext-s1 8, ext-s2 4, 4k-s1 4, 4k-s2 4.

| Bar | ext-s1 (shift 8) | ext-s2 (shift 4) |
|---|---|---|
| 1. `chose_target` ≥ 0.90 | **0.977 / 0.973** (500 / 498) | **0.992 / 0.996** (508 / 510) |
| 2. ≥ C0 + 0.10 | C0 0.020 / 0.023 (10 / 12): +0.957 / +0.949 | C0 0.381 / 0.416 (195 / 213): +0.611 / +0.580 |
| 3. `R_ask` ≥ 0.90 and ≥ `R_oth` + 0.25 | 0.977 / 0.973, `R_oth` 0.002 / 0.002 | 0.992 / 0.996, `R_oth` 0.000 / 0.002 |
| 4a. C2 `chose_target` ≤ C0 + 0.05 | 7 / 5: holds | 1 / 0: holds |
| 4b. C2′ | **broken** (table above) | **broken** (table above) |
| 5. replay, host = device | holds | holds (host = device) |

**A-ext by step,** held-out `chose_target` b1/b2:

| Step | ext-s1 s4 | ext-s1 s8 | ext-s1 s12 | ext-s2 s4 | ext-s2 s8 | ext-s2 s12 |
|---|---|---|---|---|---|---|
| 0 (C0) | 10/12 | 10/12 | 10/12 | 195/213 | 195/213 | 195/213 |
| 64 | 10/12 | 469/463 | 313/318 | 499/505 | 475/477 | 299/310 |
| 128 | 10/12 | 487/483 | 370/364 | 506/510 | 483/484 | 361/371 |
| 512 | 10/12 | 500/498 | 448/435 | 508/510 | 496/500 | 460/455 |

Train `chose_target` at the chosen shifts is 4,004/4,018 (ext-s1) and 4,091/4,095 (ext-s2), so outcome 9 (overfit) does not hold.

**A-4k against C1 (the pre-registered descriptive reading).** At the chosen shift 4, step 512:
- 4k-s1: 276/282 against C1's 213/205, which is **+0.123 / +0.150**;
- 4k-s2: 342/332 against 227/247, which is **+0.225 / +0.166**.

Both clear 0.10 in both buckets at both states. At shift 8 the gain is larger and still rising: 320/344 at 4k-s1 and 342/366 at 4k-s2. The ×16 decode stays at 13 to 35 right of 512.

**C3 (warm start from C1):** 504/496 → 510/508 at ext-s1, and 496/494 → 509/509 at ext-s2. The rule keeps the float choice and raises it.

**Classification (§4.1, §4.2).** This is a configuration result under the registered bar, not a mechanism close.
- The instrument is sound: outcome 0 passes exactly, and C2 behaves.
- What C2′ shows is that on these states the gates alone reach the other fact's row on only about 12% of train rows. As they try, the asked pick rises at ext-s1.
- That reading fits the sheet's concern, drift toward rows the stream makes salient.
- It also fits a reachability limit: the trunk's queries and keys were trained so that the asked fact is the near one, and a rotation of the gates may not reverse that. This run cannot tell the two apart.

**Results** are in the HF store at `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-native-read-train-395-2026-11-15/`: 126 files with `SHA256SUMS` (sha256 `10698314…02401880`), checked on the laptop with `sha256sum -c`. They include:
- every run's readings, saved read and picks;
- every ×16 probe;
- job A's probe JSONs and the held-out target files;
- both job scripts and logs.

The dumps stay on wick under `/workspace/out/c395/`, with their sha256 in `SHA256SUMS-A`. Saved reads at the chosen shifts: ext-s1 `b05b544a…`, ext-s2 `b07af69b…`.

## Checks run at the head

- **Laptop:**
  - `uor-r4-float-train` lib: `recall_row_targets_*` (3), `native_picks_read_one_per_row`, `the_payload_scale_and_probe_restore_what_they_found`, 5 passed;
  - `recall-payload-probe` bin: 5 passed;
  - `native_read_train`: 5 passed. The new `saved_files_and_other_states_are_refused` fails with each refusal set aside in turn (at its "never writes over" and "not the same state" assertions), and passes again with both restored.
  - `git diff --check` is clean, and the added lines have no banned word and no line starting with `.0`.
- **wick**, binaries `recall-payload-probe` `a4bcfd75…` and `native_read_train` `a539d65e…`:
  - the trainer's 4 tests;
  - 28 CUDA `vector_read` tests, including `injected_first_picks_choose_the_added_row` and `the_hard_copy_adds_the_first_pick_alone`;
  - the host and bin tests.
- The local stub-nvcc CUDA check cannot build `candle-kernels` without a real nvcc, so every CUDA build and test here ran on wick.
- Native-space five items (SKILL):
  - **state:** the read's Q31 gates;
  - **operator:** `train_pick_step` with its device twin;
  - **replay oracle:** host equals device on every run's first two steps, and the replay is byte-identical;
  - **training path:** integer throughout;
  - **artifact:** `to_bytes`, with sha256 recorded.

## Skills used

- `native-space-cycle`: the five items above. The float trunk is the fixed source of states, and the float decode is diagnostic code.
- `solution-integrity`: outcome 0 ran before any arm, and the map was read in its registered order. The A-ext pass is withheld because C2′ broke bar 4, even though bars 1 to 3 pass with room.
- `scope-guard`: the three tools and the run only. No bar, status or sheet was changed.

## Still open

1. **Tell the two readings of C2′ apart, under an addendum.** Two candidates:
   - a reachability control: the best train `chose_other` any gates can reach on these states, for example from the train split's own geometry;
   - C2′ on a state whose trunk never saw the query form.

   If the other fact is unreachable by gates alone, bar 4b tests the states, not the rule.
2. **A-4k is the learning signal.** It is +0.12 to +0.23 over the float read and still rising at shift 8, but not monotone at shift 4. A longer run at shift 8 is the next reading, with a step-size schedule (claude-391 Still open 4).
3. **The weights' and gain's own update** (claude-391 Still open 3), so that ×16 is trained rather than fixed.
4. **The 4k trunk's decode** does not answer from a right pick. Native training of the read alone cannot reach a serving reply there; the trunk is D86 step 6's.

## Plain summary

The project's integer training rule now trains the native read on real model states, on the GPU, and the results replay exactly.
- **It learns.** From scratch it recovers the float read's choice of the asked fact: 98 to 99.6% of held-out questions. At the earlier, weaker states it beats the float read by 12 to 23 points.
- **One safeguard failed.** The sheet set a control that asks the rule to learn the other fact instead. The rule barely manages that (about 12%), and at one state it drifts toward the asked fact as it tries.
- **So it is not called a pass.** This run cannot yet say whether the rule follows its targets or the states simply favour the asked fact. That is the next unit.
