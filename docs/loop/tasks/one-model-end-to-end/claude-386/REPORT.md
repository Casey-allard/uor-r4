# REPORT: one-model-end-to-end, claude-386 (V2(b) part 2: which vector read chooses the asked fact)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3989. Base `34c00f478` (claude-385's approved head, merged as `8796e312e`, #3985); `origin/main` at `bbd5aa739` merged in before this REPORT.

Casey, 15 Nov 2026, relayed by the starting session: V2(b) is two PRs, the repair unit (claude-385, merged) then "the selection test on the repaired reads, with a non-author FIT first." "If a repair turns out to be the mechanism and not the instrument … report that as the result. Do not tune it away."

This unit ran [the sheet](../../../../harness/prep/one-model-end-to-end-vector-read-selection-2026-11-14.md) as its FIT left it: 14 arm-tasks at seeds 1 and 2, 4,096 steps each, then the extension (outcome 5's route to outcome 1's 16,384 steps) the sheet's map names. It is a float reference test (D85) and an inventory-class unit under the native-space skill; its result picks the read for V2(c), so the notebook word is **negative** for the bar and the configuration, never for a mechanism (§4.1).

**What is now true, at the binary built from `34c00f478` (the float-crate tree #3985 merged):**
- **No read passes the bar at either seed, so none is selected.** At 4,096 steps every read arm's held-out `R_ask` on B2 is at most 0.006 per far bucket, against the 0.90 bar and R0's 0 to 0.010; at 16,384, Flock-g's is at most 0.023. Every read arm also misses B0's 0.90 (outcome 8): the most any reaches is 0.004.
- **The sheet's outcome is 4: the choice is learned but not used.** At 4,096 steps the map read outcome 5 (a partial choice) and sent the best read, Flock-g, to outcome 1's extension; at 16,384 steps Flock-g meets outcome 4 at both seeds.
  - With `--vector-read-stream-grad 1` the credit's gradient reaches the trunk, and every read learns where to look. At 4,096 steps Flock-g picks the asked fact's position on 205 to 247 of 512 held-out queries per far bucket (chance is about 3), Lorentz-g on 177 to 227, L2-g on 101 to 172 of the 229 to 315 rows it reads, prime-route-g on 26 to 55. With the stream stop (the default) none does: at most 12 of 1,024 on B0 and 7 of 512 on B2.
  - **At 16,384 steps Flock-g picks the asked fact on 504 and 496 of 512 (seed 1) and 496 and 494 (seed 2), and the other fact on 1 or 2.** Its `R_ask` is 0.014 and 0.023 (seed 1) and 0.000 and 0.010 (seed 2), against R0's 0.000 and 0.004 at both seeds and the bar's 0.90. The choice has stopped rising (16 to 18 rows over the last 512 steps of seed 2, 19 and -3 of seed 1).
- **A read that looks in the right place does not change the answer.** So the read's payload, as it enters the stream, does not reach the prediction: at this configuration the payload, not the choice, fails. A 97% correct choice with almost no effect is extreme enough that the next unit should first measure the payload's share of the stream at the read and of the decoded point, as an instrument check, before it treats this as a matter of the weight (L-007).
- **No instrument fault (outcome 0).** R0 on B0 reads 0.003 / 0.003 (seed 1) and 0.005 / 0.003 (seed 2); every read arm's step-0 cut equals its full reading; every read arm's weight at step 512 is finite and above 0 (0.0146 to 0.0262).
- **The Starling L2 gate closes on B2 with the stream stop (outcome 9).** It reads about half the rows at step 512 and none of 512 at step 4,096, at both seeds: an instrument limit of the offset, not a mechanism result.

## What did not work, first

1. **No read passes; selection stays keyed by token identity.** This is a configuration negative for each of the four reads at this configuration (read after layer 6 of 6, margin 1, credit weight 1, 4,096 steps; Flock-g and R0 also at 16,384), not a mechanism close (§4.1).
2. **The payload does not move the answer where it is added.** The read adds `w * table[ids[s+1]]` to the stream after the last layer, and `final_norm` follows before the nearest-token pick (`geometric_stack_native.rs`, the forward; the hits are scored on `model.hidden`, the same forward, and at 16,384 steps Flock-g's cut reading differs from its full one, so the read is active where it is scored). Flock averages its K = 4 kept payloads, so a correct first pick carries a quarter of the sum. The learned weights reach a mean size of 0.02 to 0.12 with the stop, 0.06 to 0.37 at `-g` after 4,096 steps and 3.47 to 3.72 after 16,384. Even then a 97% correct choice scores at most 19 hits of 1,024, where R0 scores 2. My reading is that the payload is small against the last-layer stream, whose address-scale distances calibrate near 60. That is not measured: no run recorded `|x|` or the payload's norm at the read. The 8 Nov Hamming readout, by contrast, acts after `final_norm`, on the decoded point.
3. **With the stream stop, no read learns the choice, even on B0.** On B0 the query writes the key token again, so the target is the one earlier position that holds the same token. A per-block unit-quaternion turn of fixed last-layer states, folded by a fixed sum to 16 coordinates, is all the read can learn there, and it does not separate that position from about 180 rivals: `chose_target` stays at 0 to 12 of 1,024. In training the stop arms chose a positive on 0.1 to 4.7% of the rows that had one (Flock B2: 15 and 24%, over every positive in a window, not only at `H4`).
4. **The sheet's paired Lorentz–L2 comparison cannot be paired.** The instrument records counts per bucket, not each query's hit. Both reads are at chance (`R_ask` 0 to 0.006 at every bucket, seed and stream setting), so their difference is within ±0.006; per query it is unrecorded.
5. **The first wick pass ran out of card memory, and its driver stopped on it.** try-1 ran two arms at once on the shared 4090 beside claude-380's runs (peak 23,878 MiB) and one ran out of memory. The driver then wrote `STOP` because it could not read an R0 B0 result. I rewrote the driver to run one arm at a time and stop only on a finished R0 B0 run that fails the bar. Before the move, seed 2's R0 B0 had finished (hits [4, 2, 3, 2] of 1,024, within the bar) and wick try-2's R0 B2 seed 2 too (hits [0, 0, 2, 1] of 512, 1,088 s), which put 28 arms past 8 hours. Under the sheet's cost rule the runs moved to two Vast RTX 5090 pods (about 200 s an arm). I stopped the wick driver by its PIDs at `arm-lorentz-b0-2` (EXIT 9). Every number in this REPORT is from the pods; the wick runs are superseded and kept in its receipts.
6. **Two watchers ran on the seed-1 pod at once.** I restarted the seed-1 watcher at 06:05Z after a fix, and the first copy kept running beside the second (two `armed` lines, 31 s apart). The first fetched, checked the tar's sha256 and took the pod down. The second then found no pod, and its alive check read an empty ssh answer as "not dead", so it would have renewed forever; I stopped it by PID. The seed-1 watcher also kept a `pgrep -f chain-386p.sh` check that matches its own command line. The extension's watcher (`watch-386x.sh`) holds a lock directory per pod and counts an empty ssh answer as unknown.
7. **One extension launch did not start its chain.** On the seed-2 extension pod `uor-pod run` answered "no SSH address" right after the scripts shipped and their SHA-256 matched. With no chain process and no output folder on the pod, I started the same `run` by hand a minute later; the launcher's log records it.
8. **Not done:** no read was moved earlier (`--vector-read-depth`), given another margin, or given its payload after `final_norm`; none was scored on replies; none has an integer form (V2(c)).

## What changed

No code. The sheet's FIT line and arms (step 1 of the ORDER), this REPORT and one notebook row. The run kit is in `local/claude/claude-386/kit/` (receipts below): `job-386.sh` (one arm per stage, outcome 0 checked at step 0 and step 512; an `ext-` stage for the 16,384-step extension), `drive-386.sh` (one arm at a time), the pod chains, launchers and completion watchers, and `score-386.jq` (the bar's inputs per arm).

## Measurements

Two Vast RTX 5090 pods (`sm_120`), one per seed, each alone on its card: seed 1 on `vast-60747617` and seed 2 on `vast-60749566`, both in Alberta, CA, at $1.042/h. The binary was built on each pod from a clean checkout at `34c00f478` (sha256 `b5dfc9e71a19c17f00fdd6d62fccd0ff51d26014277dd60b05eea9d0d2138c34`, the same on both). Every arm ended with EXIT 0, in 166 to 222 s; card peak about 10 GB.

The task is the 8 Nov sheet's two-class recall: B0 `--recall 4`, B2 `--recall 4 --recall-query 1 --recall-header 4`, 256 held-out windows. Common: `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --lr 0.001 --lr-end 0`, 4,096 steps, eval every 512, the read after layer 6 of 6, margin 1, credit 1. Per far bucket (places 2 and 3): B0 1,024 rows, B2 512. `ct` is `chose_target` (the chosen position is followed by the asked value), at step 4,096 and in brackets at 3,584; `read` is rows that read; `cut` is the held-out hit rate with the read unset; `w` the weight's mean size at 4,096. On B2 every row is a query, so `R = R_ask`.

| Arm | Seed 1 | Seed 2 |
|---|---|---|
| R0, B0 | R 0.003, 0.003 | R 0.005, 0.003 |
| Lorentz, B0 | R 0.001, 0.004; ct 2, 2 (2, 1); read 273, 305; w 0.044 | R 0.001, 0.003; ct 4, 2 (2, 3); read 312, 339; w 0.040 |
| L2, B0 | R 0.003, 0.000; ct 0, 0 (0, 2); read 105, 120; w 0.039 | R 0.001, 0.002; ct 1, 0 (1, 1); read 116, 104; w 0.043 |
| prime-route, B0 | R 0.002, 0.003; ct 11, 7 (9, 5); read 1,024; w 0.046 | R 0.002, 0.004; ct 8, 6 (13, 7); read 1,024; w 0.050 |
| Flock 4, B0 | R 0.002, 0.002; ct 9, 10 (10, 2); read 1,024; w 0.115 | R 0.002, 0.001; ct 12, 10 (6, 8); read 1,024; w 0.119 |
| R0, B2 | R 0.000, 0.004 | R 0.010, 0.002 |
| Lorentz, B2 | R 0.000, 0.004; ct 1, 3 (2, 0); read 511, 511; w 0.028 | R 0.004, 0.002; ct 4, 3 (2, 2); read 508, 508; w 0.032 |
| L2, B2 | R 0.004, 0.002; ct 0, 0 (0, 0); **read 0, 0**; w 0.026 | R 0.004, 0.000; ct 0, 0 (0, 0); **read 0, 0**; w 0.020 |
| prime-route, B2 | R 0.000, 0.000; ct 3, 2 (2, 3); read 512; w 0.025 | R 0.002, 0.004; ct 2, 5 (3, 1); read 512; w 0.031 |
| Flock 4, B2 | R 0.004, 0.000; ct 7, 6 (9, 7); read 512; w 0.044 | R 0.002, 0.000; ct 3, 5 (6, 5); read 512; w 0.033 |
| Lorentz-g, B2 | R 0.002, 0.000; ct 177, 181 (188, 204); read 418, 422; w 0.237 | R 0.004, 0.006; ct 197, 227 (192, 224); read 469, 469; w 0.239 |
| L2-g, B2 | R 0.000, 0.002; ct 122, 101 (105, 103); read 249, 229; w 0.185 | R 0.006, 0.002; ct 158, 172 (143, 147); read 302, 315; w 0.194 |
| prime-route-g, B2 | R 0.000, 0.000; ct 49, 55 (67, 59); read 512; w 0.106 | R 0.000, 0.000; ct 27, 26 (32, 32); read 512; w 0.064 |
| Flock 4-g, B2 | R 0.000, 0.000; ct 213, 205 (187, 175); read 512; w 0.371 | R 0.000, 0.000; ct 227, 247 (181, 179); read 512; w 0.365 |

- **Cut readings:** equal to the full reading at every arm, bucket and seed at step 4,096, except L2-g seed 2's second bucket (cut 0.000, full 0.002). Equality is recorded, not judged (the sheet, outcome 0).
- **Wrong-fact choices** (`chose_other`, B2, of 512): at most 8 with the stop; 6 to 39 at `-g`, so the `-g` reads pick the asked fact 1.7 to 16 times as often as the other one (prime-route-g, seed 2: 27 against 16).
- **Training-side choice** over the last 512 steps (rows that chose a positive, of the 161,224 and 161,104 B2 rows with one; seeds 1, 2): Flock-g 67.0, 69.1%; Lorentz-g 47.1, 51.3%; L2-g 41.4, 44.6%; prime-route-g 14.4, 13.9%. With the stop: 0.1 to 4.7% (Flock B2 15.2, 23.8%).
- **Starling offsets:** calibrated at the first training forward to 63.7 to 63.9 (Lorentz, seed 1), 59.5 to 59.7 (Lorentz, seed 2), 68.0 to 68.3 (L2, seed 1) and 64.1 to 64.4 (L2, seed 2); each about 2 lower by step 4,096, as the sheet expected of Adam at `lr` a step.

**The outcome map applied at 4,096 steps:**
- 0, instrument faults: none (above).
- 1, `R_ask` rises by more than 0.05 between 3,584 and 4,096: no arm.
- 2, an arm passes B2: none. 6, the trunk carries it: none meets 0.90. 7, seeds split: none passes at either.
- 3, every read's `chose_target` at most 26 everywhere: no, the `-g` reads exceed it.
- 4, some read's `chose_target` at least 256 in both far buckets: no, Flock-g's best is 247.
- **5, in between: yes, Flock-g the best.** Its `chose_target` rose by 26 and 30 rows (seed 1) and 46 and 68 (seed 2) between 3,584 and 4,096, more than 26 in a bucket at both seeds, so the map's next step is outcome 1's extension for Flock-g: 16,384 steps under this sheet.
- 8, a read misses B0: every read, at both seeds.
- 9, a Starling read under 5% of rows at 4,096: L2 on B2 with the stop, at both seeds (0 of 512).

**The extension (outcome 5's route to outcome 1's 16,384 steps; outcome 1's own trigger, a rise in `R_ask`, did not fire), Flock-g and R0 on B2 at 16,384 steps, eval every 512.** Two more RTX 5090 pods, one per seed, `vast-60754764` (Czechia, $1.016/h, seed 1) and `vast-60754784` (Alberta, $1.042/h, seed 2), the binary built again on each at `34c00f478` (the same sha256). Flock-g 755 and 789 s, R0 609 and 623 s; all EXIT 0. The learning rate decays to 0 over the 16,384 steps (`--lr-end 0`), so step 4,096 here is not the 4,096-step arm's end.

| Flock-g, B2 | step 4,096 | 8,192 | 12,288 | 15,872 | 16,384 |
|---|---|---|---|---|---|
| seed 1: `chose_target` | 267, 293 | 332, 344 | 436, 450 | 485, 499 | 504, 496 |
| seed 1: hits of 512 | 0, 3 | 2, 0 | 4, 2 | 13, 8 | 7, 12 |
| seed 2: `chose_target` | 289, 291 | 457, 444 | 474, 481 | 478, 478 | 496, 494 |
| seed 2: hits of 512 | 1, 1 | 0, 2 | 2, 2 | 1, 4 | 0, 5 |

| At 16,384 | `R_ask` | `R_oth` | cut `R_ask` | `chose_other` | `w` | R0 `R_ask` |
|---|---|---|---|---|---|---|
| seed 1 | 0.014, 0.023 | 0.008, 0.010 | 0.000, 0.004 | 2, 1 | 3.716 | 0.000, 0.004 |
| seed 2 | 0.000, 0.010 | 0.002, 0.002 | 0.002, 0.002 | 2, 1 | 3.469 | 0.000, 0.004 |

- In training over the last 512 steps Flock-g chose a positive on 81.0% (seed 1) and 80.6% (seed 2) of the rows that had one.
- **The map at 16,384:** `chose_target` is at least 256 of 512 in both far buckets at both seeds and `R_ask` misses 0.90, so **outcome 4**. `R_ask` rose by less than 0.05 over the last 512 steps (no further outcome 1), and the cut reading is within 0.05 of R0's (no outcome 6).

**Cost:** four pods, about 2.9 GPU-hours and about $3 (0.9 h and 0.88 h at $1.042/h for the 28 arms; 0.55 h each at $1.016 and $1.042/h for the extension); before the move, wick try-1 and try-2 on the shared card.

## Checks run at the head

- **Each pod, `build-u` at `34c00f478`:** clean checkout, `cargo +1.97.1 build --release --locked -p uor-r4-float-train --features cuda --bin train-in-space`; `--lib vector_read` 23 passed; `--bin train-in-space` 16 passed; binary sha256 as above.
- **Outcome 0, every read arm:** step-0 cut equal to the full reading (`step0-cut.txt` true, 24 of 24); step-512 weight finite and above 0 (`check512.json`, 24 of 24).
- **Results copied off each pod as one tar, its sha256 checked against the pod's before the pod came down:** seed 1 `c359fc6d9ad683bad3ea5bd64ba327a25fe77363d4f6d5b69d2a3db5e7e13fc3`, seed 2 `424aa26ed16094851501355a4722c190b4ae47b1cbc40396fd99893d6e528041`, extension seed 1 `6a46971ce7b0aea4e420a73170e88abe2b6a19928cb81051ceac832fbbb34e36`, extension seed 2 `62d85c98b26e587ba12c883b26380b7fbfcde0c091158e53e79d5ef1bc4a29e9`. All four pods are down (`uor-pod status`: 0 running).
- **HF store:** `results/claude/one-model-end-to-end-vector-read-selection/`, the four tars and `SHA256SUMS`.
- **Receipts:** the kit, every launcher and watcher log, the four unpacked pod result trees and the superseded wick try-1 and try-2 as one tar (`wick/c386-wick.tar.gz`, sha256 `84bcbec604002f7a883abbe1f1c0d3e315939de92025cae969f019a11b91dd81`, checked against the container's) are in `local/claude/claude-386/`, 871 files (the store copies left out), `SHA256SUMS` sha256 `b44a67b295070d08d00bf9f2b9bf43842578489a3115e06d4a17c1495fa8cfbe`.
- **The sheet:** `uor-harness prep-check` FIT; its `TEST FITNESS` line records the non-author FIT and the changes applied.

## Skills used

`native-space-cycle` (read before the work: none of the five items is met, an inventory-class unit, recorded in the ORDER); the loop's ORDER/REPORT form; the pod kit (`scripts/pod/uor-pod` on Vast, a completion watcher armed before anything shipped, L-004); `uor-store` for the results.

## Still open

- **First, measure the payload's share (an instrument check, L-007).** Record `|x|` at the read, the payload's norm and the decoded point's move toward the asked token's table point, on Flock-g's 16,384-step state, before any change. A 97% correct choice that scores at most 19 hits of 1,024 (R0: 2) is extreme enough to rule out a scoring or wiring fault first.
- **Then the payload at the readout.** Add the chosen fact's point after `final_norm`, as the Hamming readout does, or weigh it against the stream's norm, and rerun Flock-g and Lorentz-g on B2. This is a code change and its own unit, with its own sheet; it is the lever the choice counts point to.
- **The sheet's outcome 3 addendum: depth and margin.** With the stop no read learns the choice at layer 6 of 6; `--vector-read-depth` earlier, where states still carry the current token, is the cheaper test of whether the stop can stay.
- **The Starling L2 gate on B2** closes by step 4,096 (outcome 9); an offset that tracks the address scale after the first forward would keep it open.
- **V2(c):** no read is selected, so no integer form follows yet (D85).

## Plain summary

The test asked whether any of four ways of looking back at earlier text learns to find the fact a question asks about. None of them answers the questions: every arm scores at chance, at both seeds. But when the learning signal is allowed to shape the model under the reads, they do learn where to look. The best, Flock, points at the right fact about 45% of the time after the first run, where chance is under 1%, and about 97% of the time after a run four times as long. The answer still comes out wrong, about 98 times in 100. My reading is that what the read brings back is added at a point where it is too small to change the model's output, but that is not yet measured. So the next step is to measure how much the read's result moves the output, then change where and how strongly it is added, not to give up on the reads.
