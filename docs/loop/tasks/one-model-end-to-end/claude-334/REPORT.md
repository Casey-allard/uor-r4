# REPORT: one-model-end-to-end, claude-334

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-carry-hold-2026-11-09.md](../../../harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md) (FIT round 3; D67 pre-run review PASS).

## What did not work, first

- **B2 fails for both held arms, on both seeds, at 4,096 and at 16,384 steps. The hold does not carry the chosen value to the scored row.**
  - The asked value is answered on at most 2 of 512 queries in a far bucket in every B2 arm, with or without the hold, against the no-read twin B2-n's at most 5:

    | Arm (seed 1 / seed 2) | hits per far bucket, step 4,096 | step 16,384 |
    |---|---|---|
    | B2-n | 0, 2 / 5, 1 | 0, 2 / 0, 2 |
    | B2-aq (no hold) | 1, 1 / 0, 1 | not extended |
    | B2-aqH | 0, 1 / 0, 0 | 1, 0 / 0, 0 |
    | B2-aH | 0, 1 / 1, 1 | 1, 0 / 2, 1 |

  - The B1 rule fails for B2-aqH and B2-aH against B2-n at both lengths and both seeds: `R_ask` ≤ 0.004 against the 0.50 floor.
- **The hold learns its placement, and the placement does not deliver the value.**
  - B2-aqH's gain `g` grows on the question tokens and the header, and stays at zero on the value tokens and the rest:

    | `g`, mean (seed 1 / seed 2) | step 4,096 | step 16,384 |
    |---|---|---|
    | 16 question tokens | 0.275 / 0.311 | 2.857 / 0.434 |
    | 4 header tokens | 0.184 / 0.088 | −0.093 / 0.176 |
    | 512 value tokens | 0.000 / −0.001 | −0.004 / −0.002 |

  - The held term's own rank at the scored row `H4`, (r1h), does not settle near 1. It reads 74.2 / 13.4 at 4,096 and 12.4 / 174.0 at 16,384, so four times the steps moved seed 2 to chance.
  - With the held term added, B2-aqH's stream at `H4`, (r1hs), reads at chance at both lengths and seeds: 252.6–263.3.
- **A nearest-point rank cannot see the written value in the depth-2 stream, even in the arm that answers.**
  - (r1s), the whole stream at the `Q` row just after the write, reads 245.1–259.3 on the found lines (B1-a, B2-aq, B2-aqH) at both seeds. B1-a answers 512 / 512.
  - B2-aH reads 187.3–300.3, but only on its 3–23 found rows.
  - On the held arms, (r1s) as built ranks the stream with the held term at `Q` already added (the code review's finding, `in_space_long.rs:12540`). B1-a and B2-aq hold nothing, so their (r1s) is the write alone, and it is at chance there too.
  - This does not show the trunk cannot decode the value from the stream: the sheet itself says a swamped projection does not show that. It does show the ranks cannot tell a carried value from a lost one there.
  - The sheet made (r1s) descriptive only. That choice hid this from the outcome map. It is a lesson about the design, not a measurement fault.
- **On B1, (r2) at chance sits beside a correct answer.**
  - (r2) is the final point before the readout read. On B1-a it reads 261.7 / 256.1 while B1-a answers every query.
  - B1 is scored at the question's own position (`RecallQuery::scored_offset`, `in_space_long.rs:12016`; the window's doc, `in_space_long.rs:12036`). So the readout read at that row can fetch the value, and the value enters after the point (r2) ranks.
  - So, on this task, (r2) near chance is not evidence of a loss in the trunk by itself.
- **B2-aH's ask heads find almost nothing, so its hold has nothing to carry.**
  - B2-aH's found fraction at the `Q` row is 1.4% (seed 1) and 0.3% (seed 2) at 16,384. Its question-token `g` stays at 0.000 at both lengths.
  - That matches claude-330's B2-a: with the pull at the scored rows, the ask heads do not learn to choose.
- **Lost to instrument faults (L-007: not mechanism results):**
  - **Test pods t1 and t2** each stopped at I-1 with "no three digests at the pin" (exit 4), for $0.07 and $0.09.
    - The job's guard asked for a fresh "Compiling" line for the pin's own crate, and I-0 had already built it.
    - Fixed in `job-334c.sh`: the guard skips the line check for the pin's own checkout and keeps it for the base. t3 then passed.
  - **t3's first start was refused before it wrote anything:** "the unit pins 5b0ba15…, not 3620e46…".
    - My sed had moved the pin in the chain and seed scripts but not in the job's own guard.
    - The refused files were moved aside on the pod, and the chain restarted.
  - **Seed 1's first `uor-pod up` failed its git fetch** ("cannot fetch … to check --ref", nothing created), because seed 2's started at the same moment from the same checkout. The retry came up.
  - **Caught before any launch:** a jq precedence bug made `rjob-334.sh`'s I-R2 check error on every input. The laptop test against claude-330's sealed lines found it.
- **Deviations from the sheet, named:**
  - The no-read arms (B0-n, B2-n) took no `--recall-readings`, so they have no ranks.
  - The 8 host gates ran on the laptop.
  - The 9 CUDA gates, I-0, I-1 and I-7 ran on t3, not on the run pods. The run pods ran I-0 and the build again.

## What worked

- **Every instrument passes on both seeds.**
  - B0: B0-a 1.0 / 1.0; B0-n 0.0029 / 0.0029 (seed 1) and 0.0049 / 0.0029 (seed 2).
  - I-R1: B1-a answers 512 / 512 per far bucket at both seeds. Against claude-330's sealed B1-n, `R_ask` 1.000, `R_oth` 0.000, twin ≤ 0.002 at both seeds.
  - I-R2: B2-aq reproduces claude-330's sealed line at both seeds. The rows are equal and the hits equal ([0,0,1,1] and [0,0,0,1]). The question-row trace has the same keys, with `kept_ask` within 0.05 · rows.
- **The write is right at the question row.** On B2-aqH, (r1y), the ask payload, reads 1.0 / 1.0 at 4,096 and 2.4 / 1.7 at 16,384. (r1u), the write `w_y ⊙ y`, reads 1.0 / 1.0 and 4.6 / 2.2. The found fraction at `Q` is 1.0 / 0.93 and 1.0 / 0.82.
- **The hold trains through the geometry.** `g` grows from 0 to its placement on the question rows with no aid. The c-detach-g gate (`g` detached) fails as it must on t3.
- **The default paths are untouched (I-1).** The base and the pin print the same three digests on t3, among them `I1-DIGEST bc02586d4abfd94ab05ce312cb1c6b166bf2c3f2e595cdaf06bc22e66fc09663`, the same as claude-330's.
- **The build is reproducible.** The job's binary `train-in-space-3620e469` has sha256 `cda35f79dbb24d9c80dd87187c831ff8f38009d0d7df6b62897c3593dd0ad367` on both run pods.

## Outcome (the sheet's map, first match wins)

- **Outcome 0** (an instrument fails): no; B0, I-R1 and I-R2 pass on both seeds.
- **Outcome 1** (undertrained, no held arm passing): **yes at step 4,096.** B2-aH was undertrained there: its question-token `g` stays within ±0.009, below the 0.01 test. B2-aqH was not undertrained (question `g` 0.275 / 0.311).
  - The map extends each undertrained arm. The D67 pre-run review (the sheet's "Major (4a)") extends both held arms and B2-n once, to 16,384 steps at every seed, whatever the 4,096 bar says, because each pod sees one seed. All three were extended.
  - Unextended, B2-aqH's (r1h) at 4,096 reads 74.2 / 13.4, which is also "any other pattern" below, so the extension does not change the outcome.
  - **Read again at step 16,384:**
    - B2-aH is still undertrained: its question-token `g` is 0.000. The map records this as a configuration result for `h = 8` and this `lr`: the hold on the question tokens did not grow in this budget. Its header weights did grow (mean 0.385 / −0.203, extremes 1.18 / −1.40). Its ask heads find almost nothing (above), so there is little to hold.
    - B2-aqH is not undertrained: `g` grew, and `R_ask` did not rise.
- **Outcome 4** (both held arms fail), split on B2-aqH at 16,384:
  - Found fraction at `Q` ≥ 0.5, (r1y) near 1 and (r1u) near 1, at both seeds.
  - (r1h) at `H4` is **174.0 at seed 2, near chance**: the write is right, and the hold does not deliver it.
  - At seed 1 it is **12.4, neither near 1 nor near chance.**
  - **The seeds disagree on the branch, so this is the map's "any other pattern":** a configuration result for `h = 8`, `lr` 0.001, named from the readings.
  - **The map's next step** is a diagnostic sheet that cuts at the first rank on the path whose mean passes 64. That rank is (r1h) at seed 2 and (r1hs) at seed 1, and the map chooses no lever in advance.
- **Seeds:** the sheet runs seed 3 "if the two seeds split". **The lab's reading:** the seeds agree on the bar at both lengths and on the outcome ("any other pattern"), and differ only inside it, on which rank first passes 64. A third seed could not change the outcome, so none runs. That is my reading of the sheet, not its rule.

## The lab decision on the next lever (D67, logged for Casey)

- **The lab's judgement, from the readings (not something they exclude):**
  - A carry into the depth-2 stream (this hold, or a carried register in the same place) adds the value where the nearest-point ranks (r1s, r1hs) cannot see it, B1-a's included. So a second carry there could not be read by these instruments either, whether it works or not.
  - The read that works is B1's: the readout read at the scored row asks and fetches the value itself.
- **Decision:** claude-335 carries the **question** to the scored row and lets the readout read's ask heads ask with it there. That is the form B1 passes, 512 / 512. The ranks are taken against what the readout reads, and a host gate requires them to read near 1 on a B1-a-shaped case before any pod.
  - A carried register for the value, overwritten only by a found write, runs beside it as a second arm, so the two placements are compared in one run against B2-n.
  - Casey agreed to "the carried register, with the fixed rank checked on B1 first" as the next unit, in chat on 9 Nov. Carrying the question is my addition after reading the full ranks, and the arms test both.
- **Bound:** claude-333's ORDER bounded the hold at two units, and this is the second. The hold's form (`--hamming-hold`, a soft sum of earlier writes) is closed for `h = 8`, `lr` 0.001, 4,096 and 16,384 steps. The mechanism is not closed (§4.1).

## What changed

- `crates/uor-r4-float-train`, all behind default-off flags:
  - `−aS` (an ask-stream cut) and `−H` (the hold off) as ablations;
  - a capture of the `Q` and `H4` rows;
  - the six value ranks with the found fraction;
  - `g` by token group and `|w_y|` per depth in `recall.jsonl`;
  - the guard setter checks.
  - Tests: I-H5, I-V (raw and scaled ties), I-7 on B2-aqH and B1-a's I-7 with `−aS`, and claude-333's two review minors.
- The sheet: FIT rounds 1–3, the D67 pre-run review and its fixes.
- This REPORT, and one notebook row in `docs/research/notebook/float-training.md`.
- No bar, status, scope or cap changed.

## Measurements

- **Pods:** five Vast RTX 5090 pods at $0.563/h, all down:
  - test pods t1 `vast-59832881` ($0.07), t2 `vast-59833652` ($0.09) and t3 `vast-59834854` ($0.21);
  - run pods `vast-59837390` (seed 1, $0.90) and `vast-59836856` (seed 2, $0.92).
  - Each run pod trained seven arms at 4,096 steps and three extended arms at 16,384, all exiting 0.
  - **The unit's GPU total is $2.19.**
- **Training speed, context tokens per second, step 4,096** (seed 1 / seed 2):
  - B1-a 43,409 / 42,691;
  - B2-aq 43,423 / 42,689;
  - B2-aqH 42,786 / 42,057;
  - B2-aH 42,786 / 42,050;
  - B2-n 88,906 / 87,631.
  - The hold costs about 1.5% beside the ask heads.
- **Training loss at the last step:** B0-a 0.841 / 0.842 and B1-a 0.973 / 0.989. Every B2 arm reads 1.000–1.001 at both lengths, the same as the no-read twin.
- **Instrument checks:**
  - **On t3:**
    - I-0: 448 passed, 0 failed, 18 ignored, then the named tests;
    - I-1: MATCH;
    - the 9 CUDA gates FAIL AS THEY MUST;
    - I-7: 3 of 3.
  - **On both run pods:** I-0 448 passed; I-6 ok on all seven arms; the I-4 step-0 picks are the same as each arm's twin; and the read's weight is off zero at step 512 (0.037–0.394).
  - **On the laptop:** the 8 host gates FAIL AS THEY MUST at the pin (`local/claude/claude-334/host-gates-334d.out`).
- **Results:** sealed to the store at `results/claude/one-model-end-to-end-carry-hold-2026-11-09/runs/s{1,2}/` and t3 at `…/tests/t3/`. They were read back on the laptop with `SHA256SUMS` OK.
  - seed 1 tarball sha256: `57cbd25a941f56458436caaa258b0afbe8a79a5520d5f31a371e7b33cf795d93`
  - seed 2 tarball sha256: `4d952047d69e4f80bcd1385d95c0189d268c1949d82503941c70e8395d2314fd`
  - t3 tarball sha256: `3a6f20d6b2234fe837f2d4a515bfbef92e48931dad4db8a217c7be7b760850f2`

## What this does and does not show

- **It shows:**
  - A learned per-token hold, summing the last 8 rows' ask writes into the depth-2 stream, does not carry a chosen value four tokens to a scored row, at either length, on either seed.
  - It learns to weight the question rows.
  - A nearest-point rank cannot see the value in the stream it adds into, even at the write row and even in B1, where the answer is right.
- **It does not show:**
  - that no carry works;
  - anything about a carry into the readout read, about language text or about the served integer path. No held model is exported or served: the export and the decode step refuse the hold.

Closes nothing: #3407 stays open.

## Checks run at the head

- **On the pods, at the code pin `3620e4697`** (the head's later commits touch only docs): I-0, I-1, I-7, the 9 CUDA gates and the smoke on t3; I-0 and the build on both run pods.
- **On the laptop:** the 8 host gates at the pin, and `cargo test --lib` of `uor-r4-float-train` at the pin: 330 passed (`local/claude/claude-334/host-lib-3620e46.out`).
- `loop gates --full`, `loop report --check` and `loop pr-check --local` run at the head before the queue. The fresh exact-head review is posted with `loop review`.

## Skills used

- None beyond the loop itself: the ORDER, three FIT rounds, the D67 pre-run review, and the fresh exact-head review.

## Still open

- **claude-335: the question carry into the readout read, and a value register beside it, on B2**, under its own sheet. The ranks are re-taken against what the readout reads and gated on a B1-a-shaped host case first. Under claude-326's lab decision, no language run on this line starts before a read passes B2.
- **(r1s) promoted from descriptive to a gated reading** in the next sheet.
- **Minors carried from claude-330:** an I-7 configuration where `−c2` matters; no end-to-end `run_recall` test with ask heads on the laptop.
- **Minors from this unit's code review, not fixed here** (no sealed number changes):
  - a rank of an all-zero point (the `−aS` (r1u) and (r1h) cells, the `−H` (r1h) cell) prints the id tie-break's mean, about 256, rather than "none";
  - (r1s) on held arms includes the held term at `Q`;
  - loading now refuses a held checkpoint whose gain count differs from its ask rows, which claude-333's could have loaded.

## Plain summary

The model can find the right fact at the question: it picks the right value nearly every time. It still cannot say that fact four words later.

This unit tried a short memory that adds up the last eight words' found facts and passes the sum forward. The memory learned to favour the question word, as intended, but the answer stayed at chance: at most 2 right of 512, the same as a model with no memory, even after four times the training.

The readings suggest why. In the channel where the memory adds the fact, our measurements cannot pick the fact out, even in the test the model gets 100% right. That test works because a later lookup fetches the answer directly at the row being scored.

The next unit carries the question forward to that later lookup, instead of carrying the answer through that channel. Five rented GPUs ran it for $2.19.
