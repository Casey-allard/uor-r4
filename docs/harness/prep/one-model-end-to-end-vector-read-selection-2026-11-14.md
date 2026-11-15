# Prep sheet: which vector read chooses the asked fact (one-model-end-to-end, V2(b), 14 Nov 2026)

Written by claude-381 at the end of V2(a). claude-385 (#3985) repaired the reads it uses; claude-386 updated this sheet to match and runs the arms. Nothing here runs before a fresh non-author FIT passes.

## The question

V2(a) put three state-keyed look-back reads into the float trunk (`crates/uor-r4-float-train/src/vector_reads.rs`): `--starling-read lorentz|l2`, `--prime-route-read 1` and `--state-flock-read K`. Each position picks an earlier position by its own state and adds the shared table's point of the token that followed it; a margin hinge credits the best earlier position followed by the row's own next token over every other. V2(b) asks one thing: **does any of them learn to choose the asked fact when the query is not the key**, the case the 8 Nov sheet ([query-recall](one-model-end-to-end-query-recall-2026-11-08.md)) built for the Hamming read? The winner, if any, gets the integer form in V2(c) (D85: float is reference or diagnostic code; this test produces a choice, never a served model).

## Steps: the arms

The task is the 8 Nov sheet's two-class recall with the 4-token header (its B2 form, `--recall 4 --recall-query 1 --recall-header 4`), unchanged: 4 episodes a window over 2 relations each, each fact written `K v`, the asked relation queried as `Q(K_j) H1 H2 H3 H4`, the value predicted at `H4`, distances in the two far buckets, 256 held-out windows (`--recall-eval-windows 256`). Its B0 (`--recall 4`, the query writes the key again) is the instrument arm.

At `H4` the credit's only positive is the asked fact's relation position `p - 1` (the one earlier position followed by the asked value), so a read that chooses it carries exactly the asked value's table point.

| Arm | Read | Task |
|---|---|---|
| **R0** | none | B0 and B2 |
| **R-lor** | `--starling-read lorentz` | B0 and B2 |
| **R-l2** | `--starling-read l2` (the flat control) | B0 and B2 |
| **R-pr** | `--prime-route-read 1` | B0 and B2 |
| **R-fl** | `--state-flock-read 4` | B0 and B2 |
| **R-lor-g**, **R-l2-g**, **R-pr-g**, **R-fl-g** | each read above with `--vector-read-stream-grad 1` | B2 only |

R0 runs at the same seeds as every read arm, so each comparison with R0 is at one seed. Fourteen arm-tasks.

Common: the FBS sheet trunk at the 8 Nov sheet's settings (`--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 4096 --eval-every 512 --lr 0.001 --lr-end 0`, context 192), the read after the last layer (`--vector-read-depth` left out), margin 1 and credit weight 1, seeds 1 and 2, a third seed when they split. No Hamming read, no ask table, no carry.

**Built in claude-385 (#3985), before any arm:**
- `run_recall` takes a vector read as `run_long` does: identity rotations and a zero weight, its own optimizer at the trainer's rate, its credit added to the gradient of every training forward (the logged loss is the task's alone), saved with the final model.
- **The stream stop (the default):** the address reads a detached copy of the stream, so the credit and the straight-through factor reach only the read's rotations and offset; the payload still adds to the stream. `--vector-read-stream-grad 1` restores the claude-381 path.
- **Lorentz** in the cancellation-free form, `e = (|q - k|^2 - (l_q - l_k)^2) / 2`; **prime-route** energy from a detached 120 × 120 cell-pair table; **every read latest on a tie**, Flock's K too.
- **The cut reading `R^{-v}`:** every reading scores the held-out windows a second time with the read unset on the thread, so the trunk is the flags-off trunk (equal in value to weight 0). Written as `vector_read.cut_hits` and `cut_query` in `recall.jsonl`.
- **The choice at each scored row:** `vector_read.read`, `chose_target` (the chosen position is followed by the asked value: the read picked the asked fact) and `chose_other` (the other fact's), per far bucket; and the training forwards' counts since the last reading (`train`: rows, rows read, rows with a positive, rows that chose one, credit terms).
- **The Starling offset from the address scale:** the first training forward (never an eval forward) sets it to the lower median, over that forward's rows, of each row's nearest earlier distance (chord for Lorentz, L2 for the control), so about half the rows read from step 1. Printed as `calibrated_offset` at the step-512 reading. At the fixed 2.5 claude-381 saw 0 rows read; claude-385's 64-step B2 smokes settled at 59.5 (Lorentz) and 64.3 (L2), reading 311–355 of 512 rows per far bucket.

**Settings for the FIT to weigh:**
- **Credit weight 1, kept.** With the stream stop, claude-385's 256-step `p` smokes scored dev top-1 77, 79, 40 and 64 (Lorentz, L2, prime-route, Flock) against off's 88, where claude-381's had 4–10. Here the task's positives are its facts, and the question is whether the read can learn the choice at all; a smaller weight weakens the only signal that teaches where to look.
- **Both stream settings on B2, in the main pass (FIT change 1).** With the stop (the default), nothing shapes `H4`'s state toward `K_j`'s: the read sits after the last layer, only its per-block rotations of fixed states learn, and R0 already shows the trunk alone does not carry `K_j` to `H4`. So each read also runs B2 at `--vector-read-stream-grad 1`, where the credit's gradient reaches the trunk. The cut term guards the bar either way: a pass whose cut reading also passes is the trunk's, not the read's. B0 runs with the stop only.
- **The Starling gate is fixed by calibration.** The offset steps by Adam at most `lr` a step, about 2 units over 4,096 steps against a start near 60, and most of its credit terms are "do not read" terms. After the first training forward the gate moves only through the address scale. Every reading records the read fraction (`read / rows`).

## Bar (pre-registered), at step 4,096, both far buckets, both seeds

`R` is the held-out hit rate (`hits / rows` at places 2 and 3); `R_ask`, `R_oth` the two-class asked and other rates (`query.ask`, `query.oth`). On B2 every scored row is a query, so `query.*[b][s]` (far bucket × tail slot, 512 rows per far bucket) gives `R = R_ask`. Each read is judged at its better stream setting on B2, and both are reported.
- **B0 (instrument):** `R(R0, B0) <= 0.02` and, for each read arm, `R(B0) >= 0.90`. A read arm that misses B0 is read as not fit for B2 at this configuration; its B2 is still reported.
- **B2 (choice):** a read arm passes when `R_ask >= 0.90`, `R_ask >= R_oth + 0.25`, `R_ask >= R_ask(R0) + 0.10`, and its cut reading `R^{-v}_ask` is within 0.05 of R0's `R_ask` (the read, not the trunk, carries it).
- **Selection:** of the arms that pass B2 at both seeds, the highest mean `R_ask` wins; a tie within 0.02 goes to the cheaper serving read (prime-route, then Flock, then Starling). No arm passing is a configuration negative for each read (§4.1), not a mechanism close; selection then stays keyed by token identity, and the REPORT says so.
- **Lorentz against L2:** reported as a paired difference over the same held-out queries, descriptive.
- **Descriptive, every arm:** `chose_target / rows` at step 4,096, so a pass or a miss says whether the read looked in the right place.

## Outcome map

Counts are per far bucket, out of its 512 B2 rows. Chance for `chose_target` is one over the earlier positions a row can choose, about 1/170 to 1/190 at `H4`, so about 3 of 512 rows; Starling reads only the rows its gate opens, and its chance is over those.

0. **Instrument faults:** R0 reads above 0.02 on B0; a cut reading at step 0 differs from the full reading (the weight is 0 there, so they must be equal); or, at the step-512 reading of a read arm, `vector_read.weight_mean_abs` is 0 or not finite (the read never learned a weight). Whether the cut reading equals the full one at step 512 is recorded, not judged: a read that does not help yet gives equal hits either way (claude-385's 64-step Lorentz smoke: `hits` and `cut_hits` both `[0,0,0,1]`), so equality cannot tell a no-op cut from a real negative (L-007). That the cut unsets the read is pinned instead by the host test `the_cut_and_the_pick_recording_restore_what_they_found` and claude-381's `a_set_then_cleared_vector_read_leaves_the_trunk_bit_for_bit`. Any fault: repair and rerun.
1. **Undertrained:** an arm misses the B2 bar and its `R_ask` in either far bucket rises by more than 0.05 between the checkpoints at 3,584 and 4,096 steps. Extend that arm to 16,384 steps under this sheet.
2. **One or more arms pass B2:** the winner goes to V2(c), the integer form, with its own sheet.
3. **No choice learned:** none passes, and every read arm's `chose_target` is at most 26 of 512 rows (0.05, more than 8 times chance) in both far buckets at both stream settings. The credit does not teach the choice at this configuration. Next, under an addendum: depth (`--vector-read-depth`) and margin.
4. **The choice is learned but not used:** none passes, and some read arm's `chose_target` is at least 256 of 512 rows (0.5) in both far buckets, while its `R_ask` misses 0.90. The payload or its weight fails. Next: the payload's weight and the depth.
5. **In between:** `chose_target` between 26 and 256 of 512 for the best read arm. Reported as a partial choice with its per-bucket counts; next is outcome 1's extension for that arm if its `chose_target` still rises by more than 26 rows between 3,584 and 4,096, else outcome 3's addendum.
6. **The trunk carries it:** a read arm meets `R_ask >= 0.90` but its cut reading `R^{-v}_ask` is not within 0.05 of R0's. Not a pass: the credit's gradient taught the trunk (only possible at `--vector-read-stream-grad 1`). Reported as a trunk result for that read, with its cut and full readings.
7. **Seeds split:** an arm passes at one seed and misses at the other. A third seed runs for that arm (and R0 at that seed); it passes only when it passes at two of three, and the REPORT gives all three.
8. **A read arm misses B0:** reported as not fit for B2 at this configuration, with its B0 and B2 readings; it cannot win selection.
9. **A Starling arm reads under 5% of its rows at step 4,096:** the gate closed; report it as an instrument limit of the offset, not a mechanism result.

## Data split (generated, none from the store)

Every window comes from `(seed, index)`; held-out windows use `seed ^ RECALL_EVAL_SEED`, as the 8 Nov sheet.

## Base artifact

None. Every arm trains from step 0 at the code head the REPORT names.

## Minimum test power

B2: 512 held-out queries per far bucket (256 windows × 4 queries, two per bucket); near 0.9 one arm's standard deviation is about 0.013, so the 0.10 margin over R0 is more than 5 standard deviations; the paired 0.25 margin over `R_oth` is wider still. B0: 1,024 queries per far bucket.

## Cost

Twenty-eight runs (fourteen arm-tasks at two seeds) of 4,096 steps. claude-385's 64-step B2 smokes on wick's shared RTX 4090 ran off at 0.19–0.36 s a step and each read at 1.05–1.10 times that, two runs at once. At 0.4 s a step, 28 runs is about 12.7 GPU-hours, about 6.4 hours of wall clock two at a time. The runs go on wick, or on up to two Vast RTX 5090 pods inside the caps if wick's queue would pass 8 hours, under $7 on Vast; a 5090 needs an sm_120 build of the binary, made on the pod with nvcc.

## Results and stop rule

Results to `results/claude/one-model-end-to-end-vector-read-selection/` in the HF store with a `SHA256SUMS`, uploaded from the laptop with `uor-store` after the runs are fetched (the job script does not upload). The job script checks outcome 0 at step 0 and at step 512 of every read arm and stops that arm on a fault. Stop at R0's B0 fault, a non-finite loss, or outcome 0. A completion watcher is armed on every run (L-004).

TEST FITNESS: FIT WITH CHANGES, applied (claude-386, 15 Nov 2026: a fresh non-author FIT subagent; changes 1, 2, 4, 6 and 7 applied above, change 5 agreed with the sheet; change 3 applied as the step-512 weight check, with cut equality recorded but not judged, since equal counts cannot tell a no-op cut from a read that does not help)
