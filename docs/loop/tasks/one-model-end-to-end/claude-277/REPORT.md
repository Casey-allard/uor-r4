# REPORT: one-model-end-to-end, claude-277 (P3(a): scoring passes side by side on one GPU)

Lab: claude. [ORDER](./ORDER.md). PR #3664, item #3407. Plan step: Casey's plan of 4 November, P3 (faster scoring), part (a) (bound: 3 units for P3). It ran [P3(a)'s sheet](../../../../harness/prep/one-model-end-to-end-side-by-side-2026-11-04.md) as pre-registered, at the pin `e1ad88afc5a7f43f5e7714d9ab70b8937aa6bdde`, on one RTX 5090.

**In one paragraph.** Scoring two passes side by side on one GPU never changed a byte, but it did not clearly save time, so **the sheet's Bar reads Inconclusive**.
- **G holds in all nine comparisons** and **I-3 holds**: every file a pass wrote equals the serial pass's file.
- **At 192 the evidence splits.**
  - B2 took 211.5 s, below the faster serial pair (A1, 211.7 s) by 0.19 s.
  - B1 took 226.1 s, above it.
  - The mean speedup is 1.12, under the 1.2 floor.
  - The two serial repeats differ by 31% on the same pod, which is far more than any gain.
- **What follows from Inconclusive:** the passes stay serial, and P5's pod repeats one A/B pair at no extra cost.
- **At 2,048, two passes at once took 6,269 s,** 2.39 times P2b's single pass on another pod (2,624 s). So this line's 2,048 passes stay one at a time.
- **Records:** the run cost about $2.45. All 456 sealed files are in the store and read back equal.

## Tokens per second first

Decode tok/s for each p-long queries file (609 queries, 32 tokens each, 19,488 tokens a file), from the `decode` pair `--score-rows` printed for it. Phases A ran one pass at a time, B two at once, C four at once, and D two 2,048 passes at once.

The two share columns divide decode seconds by the pass's wall time: all eleven `decode` pairs a pass prints, and the two queries files alone.

| pass | memory on | memory off | pass wall time (s) | all decode share | queries-file decode share |
|---|---|---|---|---|---|
| A1 (L) 192 | 643.1 | 649.3 | 109.3 | 85% | 55% |
| A1 (S) 192 | 701.1 | 707.9 | 102.3 | 83% | 54% |
| A2 (L) 192 | 638.7 | 493.0 | 150.2 | 86% | 47% |
| A2 (S) 192 | 613.3 | 622.0 | 127.5 | 85% | 50% |
| B1 (L) 192 | 281.2 | 305.1 | 226.1 | 85% | 59% |
| B1 (S) 192 | 311.7 | 337.5 | 218.2 | 85% | 55% |
| B2 (L) 192 | 300.9 | 324.5 | 211.5 | 86% | 59% |
| B2 (S) 192 | 331.5 | 352.8 | 206.0 | 85% | 55% |
| C, four passes, (L) / (S) | 139.9–141.8 / 153.5–153.8 | 160.5–161.2 / 176.5–177.9 | 432.2–443.1 | 82–83% | 55–59% |
| D (L) 2,048, each of two | 6.31 | 6.33 | 6,268.9 | 99% | 98% |

- **At 192, two passes at once decode at 281–353 tok/s each,** against 493–708 for one pass alone (613–708 but for A2's one slow file). Together that is roughly what one pass gives alone.
- **At 2,048, two passes at once decode at 6.3 tok/s each,** 12.6 between them. P2b's single pass on another pod did 15.2. This pod decoded about 1.37 times as fast as P2b's at 192, so the cost of time-slicing at 2,048 is not measured here (What did not work, item 4).
- **Decode is 99% of a 2,048 pass and 82–86% of a 192 pass.** The two queries files alone are about half of a 192 pass. The rest of a 192 pass is row scoring and loading. So a faster decode shortens nearly the whole pass at 2,048, and most of it at 192.
- **GPU utilization (R4),** from `smi.log`'s samples inside each phase's start and end:

  | | A1 | B1 | A2 | B2 | C | D |
  |---|---|---|---|---|---|---|
  | mean utilization | 68% | 93% | 52% | 94% | 97% | 99% |
  | samples under 50% | 31% | 3.3% | 43% | 2.5% | 0.4% | 0.1% |
  | samples | 393 | 420 | 501 | 393 | 818 | 11,570 |

  Utilization × wall time, a rough measure of the time a kernel ran, is about 145 s in both A1 and A2, and 198–210 s in B1 and B2 for the same two passes. Side by side filled the GPU's idle time, but the kernels' own time grew by about as much as it saved.
- **GPU memory peaks:**
  - one pass: 3,532 and 3,596 MiB;
  - two at once: 7,027 and 7,155 MiB;
  - four at once: 14,274 MiB;
  - D: 7,155 MiB.
- These are the float scorer's speeds on one pod, not the served integer model's.

## What did not work

1. **No clear speedup at 192: Inconclusive.** R1, from the job's millisecond stamps:

   | phase | A1 | A2 | B1 | B2 |
   |---|---|---|---|---|
   | wall time (s) | 211.690 | 277.687 | 226.142 | 211.497 |

   - B2 is below min(A1, A2) = 211.690 s by 0.193 s, but B1 is not. Exactly one B is below, so the Bar reads Inconclusive.
   - mean(A) / mean(B) = 1.118, under the 1.2 floor.
   - The warm pair A2 / B2 reads 1.313. It is reported only: it rests on the one slow repeat.
2. **The two serial repeats differ by 31% on one pod, larger than the effect the run looks for.** A2 took 277.7 s against A1's 211.7 s, with the same checkpoints, the same commands and byte-identical files.
   - **This was not the expected cold start.** The sheet expected A1 to be the slow repeat, because of cold caches. The warm repeat was the slow one.
   - **Where A2's extra time went:**
     - most of it is in its (L) pass, 150.2 s against 109.3 s;
     - its memory-off queries file decoded at 493.0 tok/s, against 638.7 for memory on in the same pass.
   - **The GPU trace puts it on the host side.** A2's mean utilization was 52%, against A1's 68%, with 43% of its samples under 50% against 31%. Utilization × wall time is about 145 s in both. So the GPU did about the same work in each, and in A2 it waited longer between kernels.
   - **Cause:** not identified. The run has no host-side trace, so it cannot say what the host was doing.
   - **What it means:** on this pod type, a single A/B pair cannot resolve a 1.2× gain. The Bar's two-repeat rule exists for this reason.
3. **Four at once does not beat two pairs (R2).**
   - C took 443.150 s, against 489.377 s for its four passes run serially (1.104×) and 437.639 s for two B pairs back to back.
   - The width stays one, since the Bar keeps the passes serial on Inconclusive.
4. **Two 2,048 passes at once gain nothing (R3).**
   - D's wall time was 6,268.912 s. Against P2b's single pass of 2,624 s, r = 2.389, 19% more than twice that pass.
   - r ≥ 1.7, so this line's 2,048 passes stay one at a time.
   - **The size of any overhead is unresolved.** No single 2,048 pass ran on this pod. The sheet bounded the cross-pod bias at 7%, but this pod decoded about 1.37 times as fast as P2b's at 192 (A1's 643–708 tok/s against 466–517), on another driver (item 5). So whether time-slicing added overhead at 2,048 is not measured.
   - **The step holds anyway.** r stays at 1.7 or above unless one pass alone here would take more than 3,688 s (6,268.912 / 1.7), about 40% slower than P2b's pod. At 192 this pod was faster.
5. **Files from another pod differ, for the same binary, checkpoint and pin.** This is reported beside the gate, not part of it.
   - **What matched and what differed:**
     - Set beside P2b's sealed seed-1 files, A1's greedy-sample and recall files are byte-identical.
     - Its rows files and its Flock and queries sample files differ.
   - **The size of the difference,** on `rows-p-v-memory.json` at 192:
     - 26 of 15,961 top-1 picks differ, and 179 of 15,961 writer picks;
     - positions and targets are equal;
     - top-1 hits are 9,056 on P2b's pod and 9,059 here.
   - **What is the same on both pods:** the binary (`train-in-space-e1ad88af`, SHA-256 `a81ca0053a0899d992022712df38ead160f5055484301655a6f68b432ac9f074`) and the GPU model (RTX 5090).
   - **What is different:** the driver, 580.178.04 there and 595.71.05 here.
   - **The likely cause:** the kernels ship as PTX for `compute_120`, which each driver compiles to machine code when the module loads. Two drivers can compile the same PTX differently, and that would flip near-ties in the float scores. This run does not test that cause.
   - **What it means:** float scores compared across pods carry a difference of this size. Kernels compiled ahead to `sm_120` machine code would not depend on the driver. That is a candidate for the instrument, not adopted here.
   - **Decode speed differs across the pods too:** A1 decoded at 643–708 tok/s at 192, against 466–517 on P2b's pod, for the same binary. Driver and host are not separated.

## What worked

- **Running passes side by side never changed a scored byte (G).** Every pass at 192, in B1, B2 and all four of C, equals A1's pass of the same checkpoint, file for file. D's two passes equal each other.
- **The scorer is deterministic on one pod (I-3).** A1 and A2 wrote identical files.
- **Every phase completed:** all fourteen passes exited 0, and the job ended `EXIT 0 all steps passed`.

## R3's next step, and P3's next unit

- **R3's step:** this line's 2,048 passes stay one at a time.
- **The sheet said** P3(c), memory on and off in one forward, is next whatever R3 reads.
- **What changed after launch.** Casey asked at about 14:45Z, while D ran: "Can we add cache?" P3's order was then changed by a lab decision, posted for Casey to reverse on #2028 (6634218223), after a fresh decision review read APPROVE WITH CHANGES:
  - an exact decode cache replaces P3(b) and carries P3(c)'s aim;
  - it takes P3's two remaining units.
- **Why this run supports the change:** decode is 99% of a 2,048 pass here, and the 2,048 pass was 2,624 s of a P2b seed's 2,910 s of scoring.
- **The next unit** is the cache, unit 1, gated byte for byte against today's full re-run.

## Outside the sheet: an MPS availability probe

Casey asked what MPS gives and costs. After the seal and the read-back, and before `down`, `local/claude/p3/mps-probe.sh` ran on the same pod. It wrote only under `/tmp`, and its output is at `local/claude/p3/mps-probe-277.txt`.
- **The pod:** GeForce RTX 5090, compute mode Default, driver 595.71.05, running as root in the container.
- **It works:**
  - `nvidia-cuda-mps-control -d` returned 0, and `nvidia-cuda-mps-server` ran (52 MiB);
  - two CUDA test programs connected to it at a time, three times over;
  - all six runs passed their golden-value tests (`geometric_rung0_cuda_composition_matches_zero_carrier_golden` and `rung0_full_batch_writer_forward_backward_without_updates`).
- **No speed was measured.** This shows only that MPS is available on this pod type.

## What changed

No code. This unit adds:
- this REPORT;
- the sheet's outcome line;
- one row in the [float-training notebook](../../../../research/notebook/float-training.md);
- a paragraph in the [item](../../../../roadmap/items/one-model-end-to-end.md).

## Measurements

- **The pod:** `vast-59110873`, 1 × RTX 5090 at $0.975/h on Vast (Alberta, `local/claude/p3/up-277.log`). It booted at 13:34Z and was deleted at about 16:05Z, after the MPS probe (whose log ends at 16:04Z); `uor-pod down`'s output was not saved to a file. That is about 2.5 h and $2.45, inside the sheet's $1.8–2.7. D took 104 minutes, past the sheet's 45–90.
- **The job:** `local/claude/p3/job-p3a.sh`, SHA-256 `02a52e0fda6089c026cfc100bfdea5bddff177cf09a456eaf0a3d92d41efbf93`, the same on the laptop and the pod.
  - It started at 13:48:54Z and ended `EXIT 0 all steps passed` at 16:01:53Z.
  - The build step took about 40 s.
- **The grade:** `local/claude/p3/grade-p3a.sh`, SHA-256 `b9200a5b518c2a548d751502f06af291831e42d45a1fcaaf72eed1917a65bd55`, run on the read-back. Its output is `local/claude/p3/grade-277.txt`.
- **Instrument checks:**
  - **I-0:** the CUDA library suite at the pin passed.
  - **I-1:** every input folder's `SHA256SUMS` passed, and the ten checkpoint files matched P2b's seal lines. That seal's SHA-256 is `ca0b8aeda4e23a6f793df8b503534a48ff090724632dc384c66a0b4b27424b23`.
- **Phase walls (s):**

  | A1 | B1 | A2 | B2 | C | D |
  |---|---|---|---|---|---|
  | 211.690 | 226.142 | 277.687 | 211.497 | 443.150 | 6,268.912 |

- **The store:** `results/claude/one-model-end-to-end-side-by-side-2026-11-04/run/`, 460 files (the inputs left out).
  - Its `SHA256SUMS.files` (456 lines) has SHA-256 `dd13fa1d835a6b87d7a2fae4396767cb3a78143a6675eb68a84fa7aced9117b7`.
  - It was read back to the laptop, and 456 of 456 files checked OK. The pod was deleted only after that.
- **One watcher event.** At 15:04:54Z the laptop watcher's 45-minute no-change rule stopped the watcher during D (`local/claude/p3/watch-277.log`, lines 81–82). D writes no file until a 2,048 pass ends, and the GPU read 99%.
  - The watcher was re-armed with a 120-minute rule.
  - The job and the pod were not touched.

## Checks run at the head

- **No code changed** since the pin: only this unit's ORDER and REPORT, the sheet's FIT, PRE-RUN and outcome lines, the notebook row and the item paragraph.
- **The seal,** checked twice: on the pod before upload, and on the laptop after a full read-back from the store, 456 of 456 OK. `SHA256SUMS.files` has SHA-256 `dd13fa1d835a6b87d7a2fae4396767cb3a78143a6675eb68a84fa7aced9117b7`.
- **The store:** `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-side-by-side-2026-11-04/run/` holds 460 files: the 456 sealed files, their `SHA256SUMS.files`, and the run's `DONE`, `EXIT` and `job.log`. The inputs are left out; they are P2b's sealed checkpoints and claude-275's inputs, already in the store.
- **The pod** was released and deleted after the read-back and the MPS probe.
- **`loop report --check` and `loop pr-check --local`** were run at the head before review.
- **Reviews:**
  - the sheet's FIT, in two rounds, and the pre-run review, in two rounds, all before launch (the sheet's FIT and PRE-RUN lines);
  - the claims review of this REPORT read APPROVE WITH CHANGES, and its six findings are folded in. It is recorded on PR #3664 by `loop review`.

## Skills used

None applied.

## Still open

1. **P3's next unit: the exact decode cache, unit 1 of 2.** It carries each layer's scan and convolution state across decode steps, gated byte for byte against today's full re-run.
2. **The serial schedule at 192 is not closed.** P5's pod repeats one A and one B pair, as the Bar's Inconclusive rule says.
3. **The cross-pod file difference** (What did not work, item 5). Kernels compiled ahead to `sm_120` machine code are the candidate fix; untested.
4. **P2b's classification,** instrument or mechanism, comes after the cache, and before P4.

## Plain summary

Scoring a trained model means making it write many answers and checking them, and that is slow. This unit tried a cheap speed-up: run two or four scoring jobs on one graphics card at the same time instead of one after another.

It never changed a single result: every file the jobs wrote was identical either way. But it didn't clearly save time. Two jobs at once took about as long as running them in turn, and the same job run twice in a row varied by almost a third, which is more than any saving. On the longest conversations, two jobs at once were slower.

The measurements show where the time goes: almost all of it is spent writing answers, because each new word re-reads the whole conversation. So the next step is a cache, which lets each new word reuse the work already done on the words before it. It will be checked to give exactly the same answers as today.

The run cost about $2.45.

References #3407.
