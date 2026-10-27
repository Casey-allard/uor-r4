# Prep sheet: Hamming flock attention on associative recall (in-space-trunk-to-text, claude-146, 27 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-146](../../loop/tasks/in-space-trunk-to-text/claude-146/ORDER.md). Code: branch `claude/hamming-read` at `39f5cc57af3ffb33aa524b8c02cc74b2fe0dcafc`. Every pod runs `recall.sh 39f5cc57af3ffb33aa524b8c02cc74b2fe0dcafc SEED`, which re-runs the CUDA tests on its own build first.

TEST FITNESS: FIT after two text fixes. One fresh non-author review covers the FIT and the D67 pre-run review (PASS after the fixes): light gates, per the owner's 27 Oct direction on #2028. The fixes put a number on "the near bucket works", and correct the power to two-arm standard deviations.

The job script, `local/claude/hamming/recall.sh`, sha256 `c81ae656266a2b67eaa826f027718216892a8d694bc565c19abfc50ac553e1f2`, folds the three negative gates into its EXIT: a gate counts only with code 101 and its test named FAILED. Each arm's tokens/s is measured with three arms sharing one GPU.

## The question

**Does Hamming flock attention give the trunk retrieval it cannot do alone: recall a value placed up to 180 tokens back?**

This is the owner's direction of 27 Oct: an MQAR-style recall probe before any language run. Recall is what the literature puts at the centre of the gap between attention and recurrent models. The trunk's reach is about 8–12 tokens (`R16` at most `0.0195`).

## The task

`train-in-space --recall 4` (`in_space_long::recall_window`), after the project's MQAR bench (`uor-r4-training/examples/mqar-bench.rs`):
- Each 192-token window holds 16 key–value pairs, 4 per distance bucket.
- A key sits at `p`, its value at `p + 1`, and the key again at `q = p + d`. The only scored target, at `q`, is the value.
- The buckets are `d` in 2–8, 9–32, 33–96 and 97–180.
- Keys, values and filler come from disjoint token ranges, and pairings are fresh every window, so a value can only come from the context.
- Every step trains on fresh windows. 128 held-out windows (2,048 queries, 512 per bucket) are scored at each checkpoint, decoded as the whole table's nearest token. Chance is about 1 in 4,096.

## Steps: the arms (identical except as stated)

- **Common to every arm:** `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 4096 --eval-every 512 --lr 0.001 --lr-end 0`, on the FBS sheet trunk.
- **Layout:** one RTX 5090 per seed, with the three arms side by side.

| Arm | Read |
|---|---|
| `none` | none |
| `content` | `--content-read 7` (claude-145's read) |
| `hamming` | `--hamming-read 4` |

Seeds 1 and 2, one pod each. Every read starts with a zero weight, so each arm's step-0 reading must equal `none`'s (I-2).

## Bar (pre-registered), at step 4,096

`R_b(arm)` is the held-out recall in bucket `b`, as hits over 512.

- **Recall passes** when, at both seeds, `R_b(hamming) ≥ 0.50` and `R_b(hamming) ≥ R_b(none) + 0.10`, for both far buckets: 33–96 and 97–180.
- **Pass:** the Hamming read retrieves. Next is the language run, under claude-145's bars (sheet `in-space-trunk-to-text-content-2026-10-27`), with `--hamming-read 4` against no read.
- **Fail:** no language run. Classify the fail:
  - **Instrument:** `R_2–8(none) < 0.50`, so the no-read arm cannot recall a value 1–7 tokens back, inside the trunk's own 8–12-token reach; or the step-0 readings differ. The recall task, training or decode is at fault.
  - **Mechanism:** `R_2–8(none) ≥ 0.50`, so the near bucket works, but the Hamming arm misses the far-bucket bar. The read does not retrieve at this configuration.

  Either way, the REPORT records which.
- **Seeds split:** if one seed passes and one fails, a third seed runs, and two of three decide.
- **Reported, not barred:**
  - the content arm, which by design brings back the kept position's own residual, not the value after it;
  - every bucket for every arm;
  - each read's weight.

## Instrument checks

- **I-1:** the crate's CUDA library tests pass on each run's build, including the Hamming read's tests:
  - the popcount parity;
  - a zero weight;
  - causality with a zero margin;
  - the margin's credit sign;
  - a two-window causality test;
  - the trunk hook.

  Three negative gates must each fail their test (code 101). Each gate checks that its patch changed the file, and reports code 9 if not:
  - the margin's sign flipped;
  - a row allowed to keep itself;
  - picks that lose their window's offset.
- **I-2:** every arm's step-0 reading equals `none`'s.
- **I-3:** the Hamming weight moves off 0, by `recall.jsonl`'s `hamming_weight_mean_abs` at step 4,096. The margins are logged too: a margin below every distance is a dead head.
- **I-4:** no non-finite loss; the runner stops on one.

## Probe ladder

The CUDA tests and the negative gate run first in each job. The run is short, about 15 minutes, so it is its own throughput check: each arm's tokens/s is logged at every checkpoint.

## Data split (generated, none from the store)

- **Training:** fresh windows every step, from `recall_window(seed, step · batch + b)`.
- **Held out:** 128 windows from `recall_window(seed ^ RECALL_EVAL_SEED, i)`, a different seed, so no held-out window is a training window.

Both are pinned by the generator's host tests.

## Base artifact

No trained checkpoint. Every arm starts from `build_fbs_model(OneAtomActive, seed)`.

## Minimum test power

512 held-out queries per bucket per arm. At a recall near 0.5, one arm's standard deviation is about 0.022, so the difference between two arms has about 0.031, and the 0.10 margin is about 3.2 standard deviations. Two seeds must agree.

## Cost

Two RTX 5090 pods at about `$0.56/h` for about 30 minutes: under `$0.60`.

## Stop rule

- Stop on a non-finite loss or a crash. A healthy run is never cut short.
- Results go to `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`, sealed with a SHA256SUMS, and are read back before any pod comes down.
