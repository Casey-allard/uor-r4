# REPORT: one-model-end-to-end, claude-330

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-question-keyed-read-2026-11-08.md](../../../harness/prep/one-model-end-to-end-question-keyed-read-2026-11-08.md) (FIT; D67 pre-run review FIT TO RUN).

## What did not work, first

- **B2 fails on both seeds: the chosen value does not reach a scored position four tokens after the question.**
  - At step 4,096, B2-a and B2-aq answer the asked value on at most 2 of 512 queries in a far bucket, and their no-read twin B2-n on at most 5:
    - B2-a: 0 / 2 at seed 1, 0 / 1 at seed 2;
    - B2-aq: 1 / 1 and 0 / 1;
    - B2-n: 0 / 2 and 5 / 1.
  - Every B2 arm stays at chance from step 512 to step 4,096, so outcome 1 (undertrained) does not apply.
  - The outcome map lands on **outcome 3, second branch** (below). B2-aq's ask heads keep the asked position at the question row `Q(K_j)` in 376–512 of 512 rows, at both depths and on both seeds, and keep nothing at the scored row.
  - So the read chooses at the question, and the value is lost between that choice and the scored row. The loss is either in the ask heads' write to the stream at the `Q` row (`w_y ⊙ y`) or in the trunk's four-token carry. This run does not separate the two (the sheet says so).
- **With the pull at the scored rows (B2-a), the ask heads do not learn to choose at all.**
  - At the scored row, B2-a's ask heads keep the asked and the other position about equally: 25 against 26 and 33 against 23 at seed 2's depth-2 head 2. Seed 1 keeps neither. At the query row they keep nothing.
  - B2-a's pull distance falls only from 0.49–0.51 at step 0 to 0.41–0.42. B2-aq's falls to 0.14–0.16.
  - The pull has to reach the question's own row. That is a property of this recall task, whose question row is not scored.
- **Without the pull, the ask heads do not learn the question (B1-a0, descriptive).**
  - B1-a0 answers 2 / 0 (seed 1) and 2 / 1 (seed 2) of 512, against B1-a's 512 / 512. Its loss equals the no-read twin's (1.0023 against 1.0021 at seed 1).
  - The task loss alone does not train the offsets in 4,096 steps. The found-copy pull is the working signal.
- **One mutation gate does not fire, and it is claude-328's known limit, not a new gap.**
  - `i7-m1-class-mask` reads DID_NOT_FAIL on seed 1's pod. As the job ran it, the mutation also missed its target: the pin's runner loop reads `RECALL_ABLATIONS.into_iter().chain(asks)`, so `mutate-328.sh`'s reversal pattern matched a test's loop instead. Its line count still read 1+2.
  - **Re-aimed at the runner loop**, the gate still did not fire: both I-7 tests passed in 724.94 s (`local/claude/claude-330/gatefix-330.sh`, sha256 `b076efcc541408581eb2a4fe1d8792ccb86728175bc97f0112f1387a722662be`, results in `local/claude/claude-330/gatefix/`). It ran on seed 1's pod, in its own checkout and target.
  - claude-328's REPORT gives the reason, and its pods saw the same: on this configuration `−c2` reads within one hit of the plain read, so a leaked mask changes almost no pick.
  - The mask reset is guarded on the laptop by `the_ablation_guard_clears_every_mask_and_cut`, which passed in I-0 at the pin. `−aQ` sets an empty mask, so it adds nothing for that test to see.
  - The other 14 CUDA gates fail as they must, and the 8 host gates fail as they must on the laptop.
- **The first launch was lost to an instrument fault (L-007: not a mechanism result).**
  - I-1 built the base checkout into the pin's cargo target. Cargo leaves a workspace member's absolute path out of its artifact hash, so it reused the base's test binary for the pin: "Finished in 0.06 s", the same binary hash `092ee3c6f32e4c6e` in both logs, and 442 tests counted (426 run), the pin's new ones absent.
  - Both pods stopped at I-0 and were sealed to `runs/s1` and `runs/s2`, which are kept as they are (L-002). They cost $0.12.
  - **Fixed:** the base builds into its own target, and two guards stop the job unless the crate compiled from the right checkout. The rerun went to `runs-2`.
- **Deviations from the sheet, named:**
  - The CUDA gates ran on seed 1's pod only.
  - The 8 host gates (I-A1's margin, I-A2's window start, I-A3's λ = 0, I-A5's three, the ask refusals) ran on the laptop.
  - I-1 at the base appends the pin's digest block to the base's read.
  - The re-aimed `i7-m1` gate shared seed 1's GPU from about 01:13Z to 01:26Z. B0-h at seed 1 ran in 738 s at 18,445 context tok/s, against seed 2's 364 s and 37,213 tok/s. Its recall still passed I-R, and no other timing is compared across seeds.
  - The first `uor-pod up` for seed 1's rerun failed its git fetch while seed 2's started at the same moment. Nothing was created, and the retry a minute later came up.

## What worked

- **B1 passes on both seeds: the question-keyed read chooses a value by a question that is not its key.**
  - B1-a answers the asked value on 512 / 512 in both far buckets at both seeds, so `R_ask` = 1.000, `R_oth` = 0.000.
  - B1-n reads 1 / 0 (seed 1) and 1 / 1 (seed 2), so `R_ask(B1-n)` ≤ 0.002.
  - All three bar rules pass with margin: 1.0 ≥ 0.50; 1.0 ≥ 0 + 0.25; 1.0 ≥ 0.002 + 0.10.
  - claude-328's read failed this same form at `R_ask` ≤ 0.004.
- **The ask heads alone carry B1's recall.**
  - B1-a's ablations at step 4,096: `−t` (ask heads off) and `−aQ` (ask heads cut at the question rows) drop it to 0 / 0 (seed 1) and 1 / 1 (seed 2).
  - Every copy-head cut (`−c2`, `−c6`, `−c`, `−cQ`) leaves 512 / 512.
  - The ask heads keep the asked position in 479–512 of 512 rows at both depths, with the other kept on 0–4.
- **It locks on, not by slow memorization.** B1-a's far-bucket hits by checkpoint:
  - seed 1: 0, 2/0, 9/7, then 512/512 from step 1,536;
  - seed 2: 1/1, 1/0, 3/2, 508/507, then 512/512 from step 2,048.
  - The pull distance falls from 0.49–0.51 at step 0 to 0.09–0.12 at step 4,096.
- **The default paths are untouched (I-1).** The base `e4b2977e6` and the pin print the same digest on both seeds, `bc02586d4abfd94ab05ce312cb1c6b166bf2c3f2e595cdaf06bc22e66fc09663`.
- **B0 and I-R pass on both seeds.**
  - B0-a 1.0 / 1.0; B0-n 0.0029 / 0.0029 (seed 1) and 0.0049 / 0.0029 (seed 2).
  - B0-h reproduces claude-328's sealed step-4,096 line at both seeds: rows equal, hits within 0.02 · rows.
- **The build is reproducible:** the job's binary `train-in-space-7275fb01` has the same sha256 on both pods, `156635b56fffa3c284327e02ffbec629c841193d83552e49acdf378d3054a383`.

## Outcome (the sheet's map, first match wins)

- **Outcome 0** (B0-a or I-R fails): no; both pass on both seeds.
- **Outcome 1** (undertrained): no; no failing arm rises by more than 0.05 over its last two checkpoints.
- **Outcome 2** (B2-a passes): no.
- **Outcome 3, second branch:** B1-a passes, and B2-a fails. B1-a0 fails, so the pull is the working signal. B2-aq fails, and an ask head is kept at `Q(K_j)` in its trace.
  - **Next, per the sheet:** the carry lever (claude-326, Q4), tested on B2 first. Its sheet adds an ask-stream cut, to split the ask heads' stream write at the `Q` row from the trunk's carry.
  - Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **Seeds:** the two agree on every bar and branch, so no seed 3 runs.

## What changed

- `crates/uor-r4-float-train`:
  - the ask heads (`HammingReads::with_ask`): a per-token offset table that starts at 0, a query `rotate(key_rotation, point + a[x])`, the copy key, payload and combine for the type heads, saved and loaded;
  - the found-copy pull (`--hamming-attract λ`, `--hamming-attract-at question`);
  - `ReadScope::Ask` and the `−aQ` ablation;
  - the pull distance in `recall.jsonl`;
  - the refusals;
  - tests I-1, I-A1 to I-A5, I-4's ask gates, and I-7 on B1-a.
  - All of it is behind `--hamming-ask 1` and off by default.
- The sheet: its FIT, the D67 pre-run review, the first launch and its fix.
- This REPORT and one notebook row in `docs/research/notebook/float-training.md`.
- No bar, status, scope or cap changed.

## Measurements

- **Pods:** two Vast RTX 5090 pods at $0.563/h: `vast-59809744` (seed 1, $0.67) and `vast-59809724` (seed 2, $0.51). Nine arms each, 4,096 steps, batch 16. All 18 exited 0.
  - The first launch's two pods cost $0.12. **The unit's GPU total is $1.30.** Every pod is down.
- **Training speed, context tokens per second, step 4,096** (seed 2; seed 1 within 1%, except B0-h above and B0-n, 4% slower at 74,381 against 77,461):
  - ask arms (B0-a, B1-a, B1-a0, B2-a, B2-aq): 40,187–43,226;
  - claude-328's read B0-h: 37,213;
  - no-read twins: 77,461–88,133.
  - The ask heads cost about half the trunk's throughput, the same as the plain read.
- **Training loss at step 4,096**, seed 1 / seed 2:

  | Arm | Loss |
  |---|---|
  | B1-a | 0.973 / 0.989 |
  | B1-n | 1.002 / 1.002 |
  | B2-a | 1.001 / 1.000 |
  | B2-n | 1.000 / 1.000 |
  | B0-a | 0.841 / 0.842 |
  | B0-h | 0.738 / 0.706 |

- **Instrument checks on the pod**, both seeds:
  - I-0: 436 passed, 0 failed, 17 ignored, and the unit's 15 named tests passed.
  - I-1: equal.
  - I-4: step-0 picks identical with the read on and off for every ask arm against its twin. The read's weight is off zero at step 512, 0.017–0.394.
  - I-6: ok on all nine.
  - I-7: both readings tests pass.
  - I-R: PASS.
- **Results:** sealed to the store at `results/claude/one-model-end-to-end-question-keyed-read-2026-11-08/runs-2/s{1,2}/` and read back on the laptop, with `SHA256SUMS` OK.
  - seed 1 tarball sha256: `e2e67a8ea0dd5fedd6392666cf35a14cc9df775854bdaf303d593e4ab9d5f2d8`
  - seed 2 tarball sha256: `2e8ec2b3e10831f7ce8e2006a4a9b9ca7e20ec73611da2138affd198fda3f862`
  - The first launch's I-0 failures are at `runs/s{1,2}/`.

## What this does and does not show

- **It shows:** a geometric read can learn to choose, by a question token, which of two earlier facts to read. It does this exactly, on held-out windows, trained by a geometric pull toward the found copy, on both seeds. This is the reply panel's missing step at the level of the mechanism.
- **It does not show:**
  - that the value reaches an answer that is not right after the question (B2 fails);
  - anything about language text, the served integer path, or the reply panel. No ask-head model is exported or served.
  - The panel is claude-328's synthetic two-class recall task.

Closes nothing: #3407 stays open.

## Checks run at the head

- **On the pod, at the code pin `7275fb01e`** (the head's later commits touch only docs): both seeds ran
  - I-0 (`cargo test --release --locked -p uor-r4-float-train --features cuda --lib`: 436 passed, 0 failed, 17 ignored, then the unit's 15 named tests);
  - I-1, I-7 and the build.
  - Seed 1's pod also ran the 15 CUDA gates (`gates.txt`: 14 fire, `i7-m1` named above) and the re-aimed `i7-m1` (`gatefix-330.sh`).
- **On the laptop:** the 8 host gates at the pin (`local/claude/claude-330/host-gates-pin.out`, 8 of 8 fail as they must), and the CUDA type check under the stub nvcc. `cargo test --lib` of `uor-r4-float-train` at the head: 324 passed, 0 failed, 0 ignored, compiled from this worktree (`local/claude/claude-330/head-lib.out`).
- `loop gates --full` at `38df2a575`: 11 of 11 GREEN (`local/claude/claude-330/head-gates.out`); the later commit changes only this REPORT and the notebook row. `loop report --check` and `loop pr-check --local` pass at the head.
- The fresh exact-head review is posted with `loop review` before the queue.

## Skills used

- None beyond the loop itself: the ORDER, the FIT, two code review rounds, the D67 pre-run review, and the fresh exact-head review.

## Still open

- **The carry lever (claude-326, Q4), tested on B2 first**, under its own sheet. The sheet adds an ask-stream cut that separates the ask heads' write at the `Q` row from the trunk's four-token carry. It is outcome 3's next step, and no language run on this line starts before a read passes B2.
- **An I-7 pod configuration where `−c2` matters**, so the pod test can see a leaked class mask (carried from claude-328). The laptop guard test holds the reset meanwhile.
- **Minors from the code reviews**, not fixed here:
  - no end-to-end `run_recall` test with ask heads on the laptop (the CUDA I-7 test covers it on the pod);
  - `−aQ` is not read on B0-a;
  - the held-out reading's memory is not reported;
  - I-A2's tie-break mutation is equivalent.

## Plain summary

We gave the model's memory lookup a new way to ask for a fact: it learns, for each word, a small shift that turns a question word into the key of the fact it should fetch. It is trained by pulling that shift toward the right earlier fact.

When the answer comes right after the question, it fetches the right fact every time on fresh test cases, on both test runs (1,024 of 1,024). Without the pull, or without the lookup, it gets essentially none (at most 3 of 1,024).

When four other words sit between the question and the answer, it still picks the right fact at the question, but the fact never arrives at the answer's position. The next step is to test a way of carrying the fetched fact forward. Two rented GPUs ran it for $1.30 in all.
