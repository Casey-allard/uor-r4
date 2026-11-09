# REPORT: one-model-end-to-end, claude-335

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-question-carry-2026-11-09.md](../../../harness/prep/one-model-end-to-end-question-carry-2026-11-09.md) (FIT round 1, both slices; D67 pre-run review PASS-WITH-MINORS). Code pin `0ca41675a7eafe9d36c9213c5da9bbc2f8ebed25`.

## What did not work, first

- **B2-aC (the carry with the pull at the scored rows only) fails the bar at seed 1.**
  - It reads `R_ask` 0.240 / 0.258 per far bucket at step 16,384 (123 and 132 of 512), against the 0.50 floor.
  - It is **undertrained** by the sheet's test: `R_ask` rose from 0.041 / 0.066 at 14,336 to 0.240 / 0.258 at 16,384.
  - The same arm passes at seed 2 (below), so the seeds split on whether the scored loss alone trains the carry in this budget. The REPORT makes no claim on that question.
- **The carry gain `c` is not largest on the question tokens alone, as the sheet expected.**
  - On every carried arm, the largest `|c|` on the 16 question tokens is near 7.57, and on the 4 header tokens near 8.2 (B2-aC at seed 1: header mean 6.15, minimum 0.0). The header's extremes are as large as the question's, or larger.
  - On B2-aqC at seed 1 the header mean is 8.09 against the question's 7.55. At seed 2 the header tokens split in sign (mean 0.00, extremes −8.22 / +8.22), while the question tokens stay at +7.53 to +7.57.
  - On B2-aC the question tokens also split in sign: mean 1.89 (seed 1) and −2.83 (seed 2), extremes ±7.57.
  - On the `both` arms the value tokens stay small (mean −0.03 to 0.10, extremes about ±2.1) and the rest stays at about 0. On B2-aC both groups are exactly 0.
  - So the carry weights the question and the header, not the question alone. A negative `c` adds the token's antipode; the sheet did not foresee the sign split, and this unit does not read what it does.
- **The question row's own choice is lost on B2-aqC8 and B2-aC, while the scored row answers.**
  - The readout read's question-row trace (`position == "query"`, depth 6) reads below 0.5 · rows on B2-aqC8 (138–168 of 512 at both seeds) and on B2-aC (152–160 at seed 1, 84–146 at seed 2). B2-aqC keeps it, lower than without the carry: 335–407 (seed 1) and 288–357 (seed 2). B2-aq, with no carry, keeps 410–512 there.
  - So once the question is carried, the depth-6 heads at the question row stop choosing the key there. The answer does not need them to. This is descriptive: no branch reads it once a carried arm passes.
- **Lost to instrument faults (L-007: not mechanism results):**
  - **Three run pods never reached SSH** in 900 s, on Vast South Korea 5090 hosts: `vast-59854566` (seed 1), `vast-59856231` and `vast-59858741` (seed 2). `uor-pod` deleted each one. Nothing ran on them.
  - After two failed ups in an hour, `uor-pod`'s circuit breaker refused a third. A copy of the launcher, `rseed-335b.sh`, adds `--force-retry` and is otherwise the same; its hash is in the run pins. The pod-side files are pinned by hash and did not change.
  - On the laptop, the first host stub of the `c-carry-own-row` mutation matched twice and did not build ("MUTATE FAILED"). `mutate-335b.sh` narrows it to the held term's loop, and every run pod's gate then fails as it must.
- **Deviations from the sheet, named:**
  - **No seed 3.** The seeds agree on the branch (1) and on its next step, and differ inside it, on B2-aC (below). The lab's reading is that a third seed could not change the branch or the next step, so none ran. That is my reading, not the sheet's rule. A seed 3 would also need a sealed claude-330 B2-aq line at seed 3 for I-R2, and claude-330 sealed seeds 1 and 2 only.

## What worked

- **B2 passes on the question carry, at both seeds, and only through the carry.**

  | Arm, step 16,384 (seed 1 / seed 2) | hits per far bucket, of 512 | `R_ask` | `R_ask` under `−C` | Bar | `−C` |
  |---|---|---|---|---|---|
  | B2-n (no read) | 0, 2 / 0, 2 | 0.000, 0.004 / 0.000, 0.004 | — | — | — |
  | **B2-aqC** (`h = 4`, pull at both) | 512, 512 / 496, 500 | 1.000, 1.000 / 0.969, 0.977 | 0.105, 0.104 / 0.027, 0.035 | PASS / PASS | FAIL / FAIL |
  | **B2-aqC8** (`h = 8`, pull at both) | 512, 512 / 466, 483 | 1.000, 1.000 / 0.910, 0.943 | 0.021, 0.031 / 0.020, 0.047 | PASS / PASS | FAIL / FAIL |
  | B2-aC (`h = 4`, pull at the scored rows) | 123, 132 / 512, 512 | 0.240, 0.258 / 1.000, 1.000 | 0.014, 0.014 / 0.008, 0.029 | FAIL / PASS | FAIL / FAIL |

  - `R_oth` is at most 0.002 on every passing cell. Each twin is B2-n at the same seed, at most 0.004.
  - Cutting the carry at evaluation (`−C`) drops every carried arm to 0.008–0.105, so the B1 rule fails under the cut. The answer comes through the carried question, not another route.
  - The sheet's undertrained test applies to an arm that misses the bar, so it does not apply to a passing arm. The job records it for every carried arm anyway: B2-aqC8 at seed 1 reads "yes", because it rose from 0.930 / 0.918 at 14,336 to 1.000 / 1.000 at 16,384. Every other passing cell reads "no".
- **The scored-row trace reads "kept" on every passing arm.** At step 16,384, depth 6, `copy == false`, the ask heads keep the asked key at `H4`:
  - B2-aqC: 512 in every entry (seed 1); 446, 446, 512, 512 (seed 2).
  - B2-aqC8: 448, 440, 512, 512 (seed 1); 431, 425, 512, 512 (seed 2).
  - B2-aC: 512 in every entry (seed 2). At seed 1, where it fails: 226, 206, 434, 464, partly kept.
  - The readout weight is on the asked row: `weight_ask` 145–298 against `weight_oth` 0.16–5.2 per entry on the passing arms.
- **Every instrument passes on both seeds.**
  - I-R1: B1-a answers 512 / 512 per far bucket. Against claude-330's sealed B1-n, `R_ask` 1.000, `R_oth` 0.000, twin ≤ 0.002.
  - I-R2: B2-aq's step-4,096 line matches claude-330's sealed line exactly at both seeds: the rows, the hits ([0,0,1,1] and [0,0,0,1]) and every question-row `kept_ask` entry.
  - I-T: B1-a's scored-row trace is kept, 512 in every entry (seed 1) and 479–512 (seed 2).
  - I-T0: B2-aq's scored-row trace is lost, 0 in every entry at both seeds.
  - I-6 ok on all six arms at both seeds. The I-4 step-0 picks match each arm's twin, and the read's weight is off zero (0.126–0.199).
- **The code's checks pass and their mutations fail.**
  - On t1 (`vast-59852185`, at `310e7d031`): I-0 457 passed; I-1 MATCH, with `I1-DIGEST bc02586d4abfd94ab05ce312cb1c6b166bf2c3f2e595cdaf06bc22e66fc09663`, the same as claude-330's and claude-334's; I-7 3 of 3; and 8 CUDA gates FAIL AS THEY MUST (`c-carry-ones`, `-forward`, `-own-row`, `-far`, `-window`, `-pull`, `-detach`, `-off`).
  - On both run pods, at the pin: I-0 457 passed, 0 failed, 18 ignored; the 5 gates `c-carry-no-a`, `-forward`, `-own-row`, `-far` and `-window` FAIL AS THEY MUST.
- **The build is reproducible.** The job's binary `train-in-space-0ca41675` has sha256 `fcc281a6e9a63f0df4b3868b46ef169cf18a317e883a8605a8c9aff62b73c256` on both run pods.

## Outcome (the sheet's map, first match wins)

- **Branch 0** (an instrument fails): no. I-R1, I-R2, I-T and I-T0 pass at both seeds.
- **Branch 1** (a carried arm passes, and its `−C` fails the bar): **yes, at both seeds.** The carried question reaches the scored row and answers there.
  - **Seed 1:** only the `both` arms pass (B2-aqC, B2-aqC8). The `both` arms also put half the pull's weight on the scored rows, so the difference from B2-aC is the placement and the scored-row weight together, not the placement alone.
  - **Seed 2:** B2-aC passes too, so there the scored loss alone trains the carry.
  - **The seeds split on the sub-case only.** B2-aC is undertrained at seed 1 and still rising (0.04 to 0.24 in the last 2,048 steps). The REPORT claims the `both` arms at both seeds, and leaves open whether the scored pull alone suffices.
  - `c` and the scored-row trace, which the branch says to read: the trace is kept on every passing arm (above). `c` is large on the question tokens, as expected, and as large on the header tokens, which the sheet did not expect (above).
- **Next (the map's):** serve the carry at the decode step and in the export. It needs only the last `h` rows' ids. Then a language run with the carry, under its own sheet.

## What changed

- `crates/uor-r4-float-train`, all behind default-off flags:
  - `--hamming-carry-question h`: at the readout read, each ask head's asked point adds the last `h` rows' points in the same window, each weighted by a learned per-token gain `c`, which starts at 0. The found-copy pull builds its question point the same way.
  - `--hamming-attract-at both`: the pull at the question rows and the scored rows together, with `λ/2` at each.
  - `−C`, the carry off at evaluation, as the last ablation.
  - `c` by token group in `recall.jsonl`; save and load of `c`.
  - The decode step and the export refuse the carry, each by name.
  - Tests, each with a mutation that fails it: the carry is off at `c = 0`; a constructed carry reaches exactly the next `h` rows of its window and no further; nothing crosses a window; the pull's gradient reaches `c`; `−C` drops the carry alone.
- The sheet: the pre-registration, FIT round 1 and its findings, and the D67 pre-run review.
- This REPORT, and one notebook row in `docs/research/notebook/float-training.md`.
- No bar, status, scope or cap changed.

## Measurements

- **Pods:** Vast RTX 5090s, all down:
  - t1 `vast-59852185` ($0.17);
  - run pods `vast-59858070` (seed 1, $0.87) and `vast-59860889` (seed 2, $0.88);
  - the three pods that never reached SSH, about 15 minutes each at $0.563–0.603/h, at most about $0.43 if Vast bills them.
  - **The unit's GPU total is $1.92, plus at most $0.43.**
- **Training speed, context tokens per second** (seed 1 / seed 2):
  - B1-a 43,837 / 43,060 and B2-aq 43,852 / 43,053, at 4,096;
  - B2-n 89,873 / 88,192; B2-aqC 42,474 / 41,696; B2-aqC8 42,062 / 41,327; B2-aC 42,488 / 41,741, at 16,384.
  - The carry costs about 3% beside the ask heads with no carry. Doubling `h` from 4 to 8 costs about 1% more.
- **Wall time per arm** (seed 1 / seed 2): B1-a and B2-aq 339 / 345 s; B2-n 567 / 577 s; the carried arms 1,237–1,284 s.
- **Training loss at the last step:** B1-a 0.973 / 0.989; B2-aqC 0.973 / 0.992; B2-aqC8 0.989 / 0.996; B2-aC 1.000 / 0.980. B2-aq and B2-n read 1.000–1.001, the level of an arm that does not answer.
- **Results:** sealed to the store and read back on the laptop with `SHA256SUMS` OK:
  - `results/claude/one-model-end-to-end-question-carry-2026-11-09/runs/s1/`, tarball sha256 `e4d341a57273f7641f482e4e24c9cb7af443fe2a0f1058b9ce045af9ce3dd282`;
  - `…/runs/s2/`, tarball sha256 `9f0efd1cf1664a395798f93fa7415f7b0de7bfdda54e99fc216da61a0bdc301d`;
  - `…/tests/t1/`, tarball sha256 `aa6fb48538a4d3037785fce85e6b908a0eacc2f1f99f4682a83ab6b025eaa309`.

## What this does and does not show

- **It shows:**
  - With the question's ask point carried four rows forward to the readout read, the model answers B2's question four tokens after it is asked: 466–512 of 512 per far bucket, against at most 2 of 512 with no read, on both seeds, with `h = 4` and `h = 8`.
  - The answer comes through the carry: with the carry cut at evaluation, the same models answer at most 54 of 512.
  - The learned gain picks the question and the header tokens out of the window, with no hand-set value: the found-copy pull trains it.
- **It does not show:**
  - whether the pull at the scored rows alone trains the carry (the seeds split);
  - anything about language text, longer gaps than `h`, or the served integer path. No carried model is exported or served: the export and the decode step refuse the carry.

Closes nothing: #3407 stays open.

## Checks run at the head

- **On the pods, at the code pin `0ca41675a`** (the head's later commits touch only docs): I-0, the build and the 5 gates on both run pods.
- **On t1, at `310e7d031`:** I-0, I-1, I-7, 8 CUDA gates and the smoke. Between `310e7d031` and the pin only the I-C2 test changed (inside `mod tests` of `flock_read_hamming.rs`, FIT round 1's minor 1: the test's question now carries `point + a`). So I-1, I-7 and the other gates hold at the pin, and the run pods re-ran I-0 and the 5 gates on the changed test.
- **On the laptop:** the host mutation gates (`local/claude/claude-335/mut-host.out`).
- `loop gates --full`, `loop report --check` and `loop pr-check --local` run at the head before the queue. The fresh exact-head review is posted with `loop review`.

## Skills used

- None beyond the loop itself: the ORDER, one FIT round, the D67 pre-run review, and the fresh exact-head review.

## Still open

- **Serve the carry:** the decode step and the export, which need only the last `h` rows' ids, then a language run with the carry under its own sheet. Under claude-326's lab decision this is the first read on this line to pass B2, so the language run is no longer held behind B2.
- **Whether the scored pull alone trains the carry:** B2-aC split across the seeds (undertrained at seed 1). A longer B2-aC run would settle it. It does not block serving.
- **The sign of `c`:** the header tokens and B2-aC's question tokens split in sign. A negative gain adds the antipode, and no reading here says what it does.
- **Minors carried from claude-334:** the rank of an all-zero point prints about 256 rather than "none"; an I-7 configuration where `−c2` matters; no end-to-end `run_recall` test with ask heads on the laptop.

## Plain summary

On a synthetic recall task, the model can now answer a question four tokens after it is asked. Before this unit it picked the right fact at the question but could not give it four tokens later: at most 2 right answers out of 512.

This unit carries the question itself forward, four or eight words, to the place where the model looks up the answer. With that, it answers 466 to 512 of 512 on both test seeds. With the carried question switched off, the same models fall back to at most 54 of 512, so the carry is what does it.

One version, trained only where the answer is scored, worked on one seed and was still learning on the other. Three rented GPUs did the work for $1.92, plus at most $0.43 for three that never started. The next step is to make the served model use the carry, then try it on real text.
