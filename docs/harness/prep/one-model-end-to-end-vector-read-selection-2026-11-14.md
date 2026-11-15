# Prep sheet: which vector read chooses the asked fact (one-model-end-to-end, V2(b), claude-381, 14 Nov 2026)

Written by claude-381 at the end of V2(a), which ported the three reads behind flags and ran no training. Not yet read by a non-author: the next unit's first step is that FIT, and nothing here runs before it passes.

## The question

V2(a) put three state-keyed look-back reads into the float trunk (`crates/uor-r4-float-train/src/vector_reads.rs`): `--starling-read lorentz|l2`, `--prime-route-read 1` and `--state-flock-read K`. Each position picks an earlier position by its own state and adds the shared table's point of the token that followed it; a margin hinge credits the best earlier position followed by the row's own next token over every other. V2(b) asks one thing: **does any of them learn to choose the asked fact when the query is not the key**, the case the 8 Nov sheet ([query-recall](one-model-end-to-end-query-recall-2026-11-08.md)) built for the Hamming read? The winner, if any, gets the integer form in V2(c) (D85: float is reference code).

## Steps: the arms

The task is the 8 Nov sheet's two-class recall with the 4-token header (its B2 form, `--recall 4 --recall-query 1 --recall-header 4`), unchanged: 4 episodes a window over 2 relations each, the asked relation written as `Q(K_j)`, the value predicted at `H4`, distances in the two far buckets, 256 held-out windows. Its B0 (`--recall 4`, the query writes the key) is the instrument arm.

| Arm | Read | Task |
|---|---|---|
| **R0** | none | B0 and B2 |
| **R-lor** | `--starling-read lorentz` | B0 and B2 |
| **R-l2** | `--starling-read l2` (the flat control) | B0 and B2 |
| **R-pr** | `--prime-route-read 1` | B0 and B2 |
| **R-fl** | `--state-flock-read 4` | B0 and B2 |

Common: the FBS sheet trunk at the 8 Nov sheet's settings (`--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 4096 --eval-every 512 --lr 0.001 --lr-end 0`, context 192), the read after the last layer (`--vector-read-depth` left out), margin 1 and credit weight 1, seeds 1 and 2, a third seed when they split.

**Code the next unit builds first** (V2(a) refuses it today): `--recall` with a vector read (`run_recall` refuses `config.vector_read`), the read's counts per held-out query, and one eval-only reading with the read cut (its weight set to 0 and restored), written `R^{-v}`.

## Bar (pre-registered), at step 4,096, both far buckets, both seeds

- **B0 (instrument):** `R(R0, B0) <= 0.02` and, for each read arm, `R(B0) >= 0.90`. A read arm that misses B0 is read as not fit for B2 at this configuration; its B2 is still reported.
- **B2 (choice):** a read arm passes when `R_ask >= 0.90`, `R_ask >= R_oth + 0.25`, `R_ask >= R_ask(R0) + 0.10`, and its cut reading `R^{-v}_ask` is within 0.05 of R0's `R_ask` (the read, not the trunk, carries it).
- **Selection:** of the arms that pass B2 at both seeds, the highest mean `R_ask` wins; a tie within 0.02 goes to the cheaper serving read (prime-route, then Flock, then Starling). No arm passing is a configuration negative for each read (§4.1), not a mechanism close.
- **Lorentz against L2:** reported as a paired difference over the same held-out queries, descriptive.

## Outcome map

0. **R0 reads above 0.02 on B0:** an instrument fault; repair and rerun.
1. **An arm still rises by more than 0.05 between its last two checkpoints and misses:** undertrained; extend to 16,384 steps under this sheet.
2. **One or more arms pass B2:** the winner goes to V2(c), the integer form, with its own sheet.
3. **None passes, and `[vread]` shows the chosen positive rate near chance (one over the earlier positions):** the credit does not teach the choice at this configuration. Next: depth and margin, under an addendum.
4. **None passes, and the chosen positive rate is high but `R_ask` low:** the payload or its weight fails. Next: the payload's weight and the depth.

## Data split (generated, none from the store)

Every window comes from `(seed, index)`; held-out windows use `seed ^ RECALL_EVAL_SEED`, as the 8 Nov sheet.

## Base artifact

None. Every arm trains from step 0 at the pin the next unit names.

## Minimum test power

512 held-out queries per bucket, as the 8 Nov sheet: near 0.9 one arm's standard deviation is about 0.013, so the 0.10 margin over R0 is more than 5 standard deviations; the paired 0.25 margin over `R_oth` is wider still.

## Cost

Ten arm-tasks at two seeds. Wick's RTX 4090 trains `p` at 0.174 s a step without a read; the reads' all-pairs addresses add `(batch, time, time, 16)` work, so V2(a)'s smokes give each read's step time before this is sized. At an assumed 0.3 s a step, 20 runs of 4,096 steps is about 7 hours on wick, or about $4 on Vast at the caps.

## Results and stop rule

Results to `results/claude/one-model-end-to-end-vector-read-selection/` in the HF store with a `SHA256SUMS`. Stop at R0's B0 fault or a non-finite loss. A watcher is armed on every run (L-004).

TEST FITNESS: PENDING (written by claude-381, 14 Nov 2026; a fresh non-author FIT is the next unit's first step)
