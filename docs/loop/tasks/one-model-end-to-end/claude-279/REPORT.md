# REPORT: one-model-end-to-end, claude-279 (P3, cache unit 2: the Hamming read's step and the decode switch)

Lab: claude. [ORDER](./ORDER.md). PR #3673, item #3407. Plan step: Casey's plan of 4 November, P3 (faster scoring), cache unit 2 of 2. Base `f34f0b17c`. [The prep sheet](../../../../harness/prep/one-model-end-to-end-decode-cache-2026-11-04.md) ran as pre-registered, with a fresh non-author FIT and pre-run reviews. Its gates passed at the pin `11aed950ae7a1ae83f9fff672d27c54f02b33590` on one RTX 5090, Vast `vast-59143240`. The pod was up for 1 h 43 min, about $1.68, over two runs.

**In one paragraph.** Scoring's decode now steps from the exact cache wherever the window fits, and every scored file is byte-identical with the cache on and off.
- **The files:** byte-identical at 192 and at 2,048, on P2b's sealed seed-1 checkpoints, and identical to the files the base commit writes on the same pod.
- **The speed:** at P2b's 2,048 window, the two `p-long` queries files decode at 98.7 and 98.8 tok/s, against 16.7 with the cache off, 5.9× faster. A whole 2,048 scoring pass takes 441 s instead of 2,380 s.
- **The cache's reach:** 99.2% of that pass's forwards after the prefills are cached steps (96.1% of all its forwards). The sheet predicted 99.2% from a count of the queries files' steps, a different quantity of the same size.
- **The Bar** reads "Exact and faster", so `--decode-cache` defaults to 1.
- **The first pod run failed:** the cache never ran (What did not work, item 1).
- **What is left of a pass:** a cached step costs an estimated 8–9 ms, for checkpoint (L) at 2,048 and at 192. That points to a fixed cost per forward rather than window length, and it is what is left to cut.

## Tokens per second first

Decode tok/s for each queries file, from the `decode` pair `--score-rows` prints for it, one pass each on one RTX 5090:

| window, checkpoint | file | cache off | cache on | on / off |
|---|---|---|---|---|
| 2,048, (L) | `samples-p-long-queries-memory.json` | 16.68 | 98.66 | 5.91× |
| 2,048, (L) | `samples-p-long-queries-no-memory.json` | 16.75 | 98.81 | 5.90× |
| 192, (L) | `samples-p-long-queries-memory.json` | 637.96 | 1,256.95 | 1.97× |
| 192, (L) | `samples-p-long-queries-no-memory.json` | 617.35 | 1,175.12 | 1.90× |
| 192, (S) | `samples-p-long-queries-memory.json` | 705.02 | 831.80 | 1.18× |
| 192, (S) | `samples-p-long-queries-no-memory.json` | 713.70 | 846.29 | 1.19× |

- **The deciding reading is the first two rows:** P2b's 2,048 queries files, about 98% of a 2,048 pass. P2b's own pod decoded them at 15.2 tok/s ([claude-276](../claude-276/REPORT.md)); this pod, with the cache off, at 16.7.
- **The other samples files** range from 0.96× to 1.33× (the grade lists all 30 files; four S192 files are below 1×, the lowest S192's `samples-memory.json` at 0.955×). This unit does not say why their gain is smaller: the forward counts are per pass, not per file, and at L192 no step of any file slid.
- **The whole pass, by wall seconds:**
  - L2048: 2,380.2 → 441.3 s (0.185);
  - L192: 111.3 → 78.3 s (0.70);
  - S192: 100.3 → 91.3 s (0.91).
- **Where the time is now (an estimate, not a measurement):** a cache-on pass's decode seconds, less its prefills and whole-window forwards at the cache-off pass's cost per forward, divided by its steps:
  - about 8.1 ms a step at 2,048, checkpoint (L);
  - about 8.8 ms a step at 192, checkpoint (L). S192 was not estimated.

  The estimate charges each prefill the cache-off cost of a forward, though a prefill runs over its prompt, not always a full window. A step's cost barely moves when the window grows tenfold, so what remains is most likely a fixed cost per forward. Kernel launches are the likely part, not measured here. That is the fused-kernel item's target ([Still open](#still-open), item 3). A step still recomputes the read's keys, rotations and prefix frames over the whole window, so part of its cost grows with the window; that part did not show at these two windows.
- These are the float scorer's speeds, not the served integer model's.

## What did not work

1. **The first pod run's cache never ran.** Run 1, at the pin `06b225d17`, passed I-0 and I-1, then stopped at I-2 with EXIT 5. Pass `on-L192` decoded with 0 prefills and 0 steps, out of 6,797 whole-window forwards.
   - **Classified as instrument:** the switch did not engage, so nothing about exactness or speed was measured.
   - **The cause:** the scorer sets the one-atom FBS arm on every pass (`score_rows`; `train-in-space` defaults `--fbs-arm` to one-atom). The cache refused any FBS arm. The GPU tests had left the arm unset, so they never saw it.
   - **The repair, `11aed950`:**
     - the one-atom arm projects through the same `plan.forward` as no arm (`spiralcore_fbs_arms::project`), so the cache now carries it, and refuses only the three arms that run their own kernels or backward;
     - the CPU refusal test fails with the repair set aside, and passes with it;
     - the CUDA decode-switch test now sets the arm as the scorer does.
   - **A fresh reviewer checked the repair before run 2:** every projection takes one route, and the one-atom arm returns before any other branch (#3407, 6639175570).
   - **Run 1 is sealed** in the store under `claude-279/run1/` and was read back, 152 of 152 files.
   - **Run 1 did not name its refusal.** The decode does not log which refusal fired, so run 1's 0/0 fitted the arm but did not prove it was the only cause. Before run 2, the checkpoints and P2b's command were checked for the other refusals: no Flock, content, opening or register read; no decision trace; the row cut cleared before every forward. Run 2's steps confirm it.
2. **The sheet's 192 prediction was wrong.** It predicted that 32% of the queries files' steps after the first would fit at 192.
   - **(L) at 192:** the whole pass re-ran the whole window 0 times, so every step of the queries files fitted.
   - **(S) at 192:** 68.4% of the pass's forwards after the prefills were steps. The counts are per pass, so the queries files' own share is not known.
   - **The likely cause (not measured):** the count assumed every reply decodes all 32 tokens, but a reply stops at its end marker, and a reply that stops early never reaches the slide.
   - **At 2,048 the prediction held in size:** the pass's forwards after the prefills were 99.2% steps, where the queries files are about 98% of the pass.
3. **Not tested, as the sheet says:**
   - **The serial decode** (`decode_reply_with_mode`), which serves the forced, keyed and served arms and the read scopes. It keeps the full re-run. Its share of every pass here was 0 s, because P2b's commands decode none of those arms. So G compares no file the serial decode writes.
   - **The refused cases:** bf16, the window slide, the read hooks the checkpoints do not carry, the instruments and the other FBS arms. Each is a typed refusal that returns the group to the whole-window re-run.
4. **The files differ from P2b's sealed ones across pods** (reported, not gated, as the sheet set it). Against P2b's seed-1 files from its own pod:
   - in this pod's off-L2048 pass, 19 of 609 query replies differ in at least one token, with memory on and with it off alike;
   - 266 of 95,856 nearest top-1 picks on p-long differ, and 19 of 14,287 on DEV;
   - base-L192 and base-S192 differ the same way, in the `rows-*`, Flock and queries files;
   - every `recall-*` grade file is byte-identical.

   The float crate's one change between P2b's pin and this base is claude-278, whose whole-window path runs the same ops. So this most likely matches claude-277's finding that two 5090 pods disagree on some top-1 picks with one binary. It was not separated from the code here. Every gate of this unit compares passes on one pod.
5. **The CPU refusal test clears the arm by hand,** not with a drop guard, so a panic inside it would leave the arm set on that test thread (a review NIT; not fixed).

## What worked

- **G, cache on against off, byte for byte:** every output file, at L192, S192 and L2048. The printed lines are identical, once the decode and search wall seconds are dropped.
- **G0, the base commit against the pin with the cache off,** on the same pod, at L192 and S192: identical. The new code changed nothing the scorer writes.
- **I-3, determinism:** a repeated cache-off pass at L192 equals the first, byte for byte. I-3b's repeats were not needed: G and G0 held.
- **I-4, coverage:** at each window, the cache-on pass's prefills, steps and whole-window forwards add up to the cache-off pass's forwards. G therefore compared the cache where it ran:
  - L192: 218 + 6,579 + 0 = 6,797;
  - S192: 204 + 4,113 + 1,897 = 6,214;
  - L2048: 1,278 + 39,260 + 316 = 40,854.
- **I-0:** the crate's CUDA lib suite at the pin passed, 338 passed, 0 failed, 8 ignored, in 246.65 s. The sheet's 17 named tests are among them, the widened step-equals-full test (96 cases, with P2b's two Hamming reads) and the decode-switch test included.
- **I-1:** the ten checkpoint files equal P2b's seal lines, and the panels equal their SHA-256s.
- **Unit 1's deferred check, the lowest-bit flip** (reported, not gated). Flipping the lowest bit of layer 0's carried scan state, component 0, leaves 0 differences after one step, in the points and in every carry. The flip is absorbed inside the step itself. So claude-278's first G3 did not miss a state change: there was none left to see. The top-bit flip stays the gated negative, and it fails the compare.
- **GPU memory:** peaks of 3,532 to 4,364 MiB with the cache on, against 3,564 to 3,596 off. The largest is on-L192 (+768 MiB), likely because 16 rows' read residuals are held at once (not measured).

## What changed

Against the base `f34f0b17c`: +841 −88 lines in four files.
- **`crates/uor-r4-float-train/src/flock_read_hamming.rs`:**
  - the read takes the first query position, and whole-window calls pass 0;
  - `apply_last` and `apply_readout_last` run the read's own code with the query axis cut to the new position;
  - `step_instrument` returns the typed refusal for each instrument the step cannot carry: the decode cut, the row cut, the decision trace and forced codes.
- **`crates/uor-r4-float-train/src/geometric_stack_native.rs`:**
  - `DecodeCache` carries each Hamming read's input residual at every position so far, and each row's ids;
  - `prefill_refusal`, `step_refusal` and `carried_refusal` name every case the cache refuses;
  - the step-equals-full test is widened to the two Hamming reads, and the carry compare now asserts equal layer, input and read counts before it zips them;
  - the one-atom FBS arm is carried (`11aed950`).
- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - the decode switch in `decode_replies_with_mode`: it prefills, steps while the window fits, and re-runs the whole window from the first step that would slide or that the cache refuses;
  - turn-memory reply rows are named in step rows;
  - `decode_forwards` counts prefills, steps and whole-window forwards, printed per pass;
  - the decode-switch test runs through a window slide.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:** `--decode-cache 0|1`, default 1.
- **No kernel changed.** The step runs the existing kernels.

## Measurements

| pass | wall s | decode s | prefills | steps | whole | GPU peak MiB |
|---|---|---|---|---|---|---|
| base-L192 | 105.8 | 89.2 | — | — | — | 3,564 |
| off-L192 | 111.3 | 94.4 | 0 | 0 | 6,797 | 3,596 |
| on-L192 | 78.3 | 61.1 | 218 | 6,579 | 0 | 4,364 |
| off-L192-again | 109.8 | 92.8 | 0 | 0 | 6,797 | 3,596 |
| base-S192 | 100.6 | 83.5 | — | — | — | 3,564 |
| off-S192 | 100.3 | 83.3 | 0 | 0 | 6,214 | 3,564 |
| on-S192 | 91.3 | 74.0 | 204 | 4,113 | 1,897 | 3,564 |
| on-L2048 | 441.3 | 410.3 | 1,278 | 39,260 | 316 | 3,532 |
| off-L2048 | 2,380.2 | 2,349.2 | 0 | 0 | 40,854 | 3,564 |

- **The binaries' SHA-256s:**
  - the pin's `train-in-space` is `302576978212416f65a41dd81cb3513f65cd7123dc80db057fd28f317832805e`;
  - the base's is `4f41711cdb3e3b5d0f905ef9235dd7f728cb6e48a485c3046dbe3318d22bb679`.
- **Run 2's seal:** 335 files. `SHA256SUMS.files` has SHA-256 `2f184e2ec21631075d6497236e380a5486c555ffec5f4aa8320f31d877f8c20b`.
- **The laptop gates at the pin:**
  - the crate's CPU suite passes, 275 tests;
  - the stub-nvcc `--features cuda --all-targets` check passes.

## Checks run at the head

- **The head adds this REPORT, the notebook row, the item line and the sheet's outcome** to the gate commit `11aed950`. No code changed after the run.
- **The seals,** checked on the laptop:
  - run 2: 335 of 335 files after the copy off the pod, and 335 of 335 again after a full read-back from the store;
  - run 1: 152 of 152 after its read-back.
- **The store:** `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-decode-cache-2026-11-04/claude-279/` holds `run1/` and `run2/`.
- **The pod** was released and deleted after the read-back. It was up from 18:15:26Z to 19:58:41Z at $0.975/h.
- **The grade** (`grade-279.sh`, kept with the lab's scripts) reads I-0 to I-4, G and G0 true, and the Bar "Exact and faster".
- **`loop report --check` and `loop pr-check --local`** were run at the head before review.
- **Reviews, each in a fresh context:**
  - the sheet's FIT: FIT WITH CHANGES (#3407, 6638239076);
  - the pre-run reviews: NO GO, then GO in round 2 (6638389538), and GO for run 2 (6639175570);
  - the exact-head review of this PR, in three fresh slices, all APPROVE WITH CHANGES with no blocker: the read and the switch; the cache; and a claims check of every number here against the run's files. The claims check's wording fixes are in this REPORT. The code reviews' four SHOULDs are [Still open](#still-open), item 1. The decode-switch test already asserts `whole < off.whole` per case and `steps > 0, prefills > 0` overall, so it fails if the cache never runs. It is recorded by `loop review`.

## Skills used

None applied.

## Still open

1. **Harden the refusals (the code reviews' SHOULDs).** None is reached by any scoring path today, and each only adds a refusal, which returns the group to the whole-window re-run:
   - `step_instrument` does not refuse the Hamming read's tie rule being off (`set_hamming_tie_rule(false)`). Equal distances could then keep different candidates in a step and a whole window;
   - `step_refusal` checks that the reads have the same number of sites as at the prefill, not that they are the same reads. A swap to other reads with the same sites would be accepted. The fix is an identity recorded at the prefill;
   - `prefill_refusal` runs before `set_row_cut(None)`, so a row cut left by an earlier pass refuses the first prefill. That is safe but slow.
   
   These land with P2b's classification unit, the next unit to score with the cache, so its pod's I-0 runs their CUDA tests. They were kept out of this PR so the merged code is exactly the code the run qualified.
2. **Name the refusal in `decode_forwards`.** The first refusal that drops a group's cache is not logged, so a 0/0 run cannot say why. One field would have saved run 1's diagnosis.
3. **The fixed cost per step:** an estimated 8–9 ms a position, for (L) at both windows. The `float-kernel-fusion` item (Casey's call of 4 Nov, #2028 6638209391) fuses the served quantizers' op chains. Its cost reading should time a cached step too.
4. **The serial decode** keeps the full re-run. P2b's commands do not use it. A unit that scores the forced, keyed or served arms at 2,048 would need its own switch.
5. **Cross-pod drift:** 19 of 609 replies and 0.13–0.28% of top-1 picks differ from P2b's pod. A reading that rests on matching files across pods must re-score on one pod, as P2b's classification will.
6. **Then, as #2028 orders:** P2b's classification, now cheap at 2,048 with this cache; the kernel unit; the real-text corpus; P4, P5 and P6.

## Plain summary

When the model scores an answer, it writes the answer one word at a time. Until now, for every new word, it re-read the whole conversation, up to 2,048 tokens. The last unit gave the model a memory of where it left off. This one switches scoring over to that memory, with one rule: not one byte of any score may change.

On a graphics card, none did. The scores with the memory on and off are identical, byte for byte, at short and long windows, and they also match the code from before this change. At the long window, the answers that decide P2b's results now come out about six times faster, 98.7 tokens a second instead of 16.7. A whole scoring pass takes about 7 minutes instead of 40.

The first try failed for a plain reason. The scorer always sets one switch that the memory refused, so the memory never ran. The fix accepts that switch, because it computes exactly the same thing. The next speed-up is no longer the length of the conversation: each new word now costs an estimated 8 or 9 milliseconds at both windows measured, and that cost is the next target.

The two test runs cost about $1.68.

References #3407.
