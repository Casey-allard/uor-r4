# REPORT: one-model-end-to-end, claude-281 (P2b's Negative classified: trained phrasing against held-out phrasing)

Lab: claude. [ORDER](./ORDER.md). PR #3680, item #3407. Plan step: THE ORDER of 4 November (#2028 6634218223), step 4, P2b's classification, carrying the four decode-cache SHOULDs from claude-279's review. Base `ae8fce0b7`. [The prep sheet](../../../../harness/prep/one-model-end-to-end-p2b-classification-2026-11-04.md) ran as pre-registered, with a fresh non-author FIT and pre-run review. Its gates passed at the pin `64475a8307711f9eca8765c713fea19e83d75ba5` on one RTX 5090, Vast `vast-59164039`. The pod was up from 21:27:38Z to 22:33:22Z at $0.56/h, about $0.61 for both runs.

**In one paragraph.** P2b's Negative is a mechanism result, not a generalization fault.
- **What was scored:** P2b's sealed (L) at seeds 1 and 2, at 2,048 with the decode cache. The four panels were p-long and three new ones: q-tt (trained phrasings, trained values; the positive control), q-td (trained phrasings, dev values) and q-dt (dev phrasings, trained values).
- **The deciding reading:** the trained model states 0 of q-tt's 283 far word answers at both seeds, as it states 0 of p-long's 306.
- **The Bar** reads **Mechanism**: s_tt < 10 at both seeds. The model does not recall a far note even in its trained cell, on panels drawn as TRAIN was.
- **What it closes:** P2b's read configuration (copy key 1, the earliest 16 matches averaged) at this recipe. It never closes the read (§4.2). The next read configuration, copy key 2 or the latest match, is chosen on claude-267's oracle.
- **What phrasing does change is the reply's form, not its value:**
  - on trained phrasings, 386–427 of 610–617 replies open as the trained answer does ("It's …", "That's …", "{k} is …");
  - on held-out phrasings, 139–173 do, and about 80 a panel are acknowledgements ("Okay, I'll remember …").
- **Every instrument check passes**, I-0 to I-7. The four p-long recall files equal P2b's sealed ones byte for byte.
- **The decode cache's four refusals** landed, each with a test that fails with its fix set aside.
- **The first pod run stopped at I-0n,** because the row-cut test could not fail (What did not work, item 1). The repair is test only, and run 2 passed I-0n.
- **The Bar's rename:** the ORDER called the passing outcome "Phrasing"; FIT round 1 renamed it "Generalization", since R1 changes the phrasing and the value at once.

## Tokens per second first

Decode tok/s for each queries file at 2,048 with the cache, from the `decode` pair `--score-rows` prints for it, on one RTX 5090:

| panel | s1 memory | s1 out | s2 memory | s2 out | pass, s1 / s2 |
|---|---|---|---|---|---|
| p-long | 144.8 | 147.6 | 144.5 | 147.1 | 305 / 306 s |
| q-tt | 138.9 | 141.3 | 138.7 | 141.2 | 320 / 320 s |
| q-td | 140.1 | 142.7 | 140.7 | 143.1 | 315 / 314 s |
| q-dt | 137.1 | 139.6 | 137.0 | 139.5 | 311 / 312 s |

- **Against earlier pods:** P2b's own pod decoded p-long's queries files at 15.2 tok/s without the cache ([claude-276](../claude-276/REPORT.md)). claude-279's pod did 98.7 with it, and took 441 s for a whole 2,048 pass ([claude-279](../claude-279/REPORT.md)).
- **This pod is faster with the same cache:** 137–148 tok/s, 305–320 s a pass. The pods differ, and this unit does not say why. A cached step was estimated as mostly a fixed cost per forward, so a faster host is one candidate, not measured.
- **These are the float scorer's speeds,** not the served integer model's.

## What did not work

1. **Run 1 stopped at I-0n (EXIT 4), an instrument fault.** At `04b547c70`, I-0 passed, but the slide test also passed with the row-cut fix set aside, so the negative gate stopped the job before it fetched an input. The test could not fail:
   - every decode forward clears the row cut after it, so a cut left on the thread meets only the first exact-length group's first step;
   - that group is the 3-position prompt, and `prefill_refusal` refuses its prefill as a short prompt (`time` below `CONVOLUTION_WIDTH`, 4) before it asks about the cut.

   The repair, at `64475a830`, is test only: the row-cut check decodes the prompts from length 5 up. In run 2 the test passes with the fix. With the fix set aside it fails on "a row cut left on the thread changed the forwards" (`prefills: 2, steps: 11, whole: 43`, first refusal "Hamming read's row cut"), and it passes again once the fix is restored. Run 2 is the sheet's one re-run, in new folders. Run 1's 26 files are sealed in the store under `run1/`.
2. **The positive control recalls nothing.** q-tt, drawn as TRAIN was, states 0 of 283 far word answers at both seeds. q-td (0 of 287), q-dt (0 of 270) and p-long (0 of 306) read 0 as well.
   - **The test's reach:** the sheet's power table sees a recall rate of 5% or more at both seeds. Zero of 283 also bounds each seed's rate at about 1.1%, by the rule of three (3 / 283), for that seed's model. That bound is not pre-registered.
   - **Near notes and numbers miss too:**
     - no panel states a near word either (0 of 12–14 a panel, under 192 tokens back);
     - no q-tt number is stated (0 of 319), though 318 of them are values from TRAIN's own pool.
   - **The only stated replies** in the whole run are p-long seed 2's " That's 97." (query 20, a number 189 tokens back), memory on and memory out, as in P2b.
3. **R2 cannot pick a data change.** Every gap compares zeros, so the phrasing gap and the value gap both fail at both seeds. That is what the Mechanism branch expects: R2 picks a next step only when R1 passes.
4. **The laptop grade first read I-4 from the wrong file,** a reader fault. `grade-281.sh` looked for `decode_forwards` in each pass's `scored.jsonl`, and found none. The scorer prints it to stderr, `score.err`. The script now reads `score.err`. Nothing re-ran, and I-4 passes (Measurements).

## What worked

- **Every instrument check passed** (Measurements):
  - I-1 holds byte for byte at both seeds, on a third pod. So neither the scorer nor the cache moved P2b's recall grades.
  - I-7 shows that each seed's five `dev` files are identical across its four passes, so no pass's state reached another.
- **The cache ran in every pass:** 37,903 to 39,453 cached steps against 1,243 to 1,291 prefills. Every pass's first refusal is "window full", the slide at 2,048, as the slide test expects.
- **The four refusals and the census field** each fail their tests with the fix set aside:
  - the CPU tests on the laptop;
  - the row-cut fix's CUDA test on the pod (I-0n).
- **The reply's form follows the phrasing (R3).** Trained phrasings draw the trained answer's opening; held-out phrasings draw acknowledgements:

  | panel | ack (s1, s2) | answer opening (s1, s2) |
  |---|---|---|
  | q-tt | 11, 6 | 386, 420 |
  | q-td | 14, 9 | 412, 427 |
  | p-long | 81, 82 | 139, 173 |
  | q-dt | 80, 79 | 153, 151 |

  So the reply's form follows the phrasing. Its value is the note's in neither. This unit does not measure which part of the model sets the form.
- **The whole classification cost about $0.61,** both runs on one pod, inside the sheet's $3 bound. Its passes ran 305–320 s against the sheet's estimate of 441 s from claude-279's pod.

## What changed

- **The decode cache's four refusals** (claude-279's review SHOULDs). Each returns the group to the whole-window re-run, so none changes a scored token, which I-1 checks:
  1. `step_instrument` (`flock_read_hamming.rs`) refuses the Hamming read's tie rule being off. With it off, a CUDA sort leaves the order of equal keys open, so a step's read could break a tie the whole window's does not. CPU test `the_cache_refuses_what_it_cannot_carry`; with the fix set aside it fails at `geometric_stack_native.rs:3492`.
  2. `step_refusal` (`geometric_stack_native.rs`) refuses reads set since the prefill. It now compares an epoch that `set_hamming_reads` counts and the prefill records. Before, it compared only the number of read sites, so other reads at as many sites passed. Same CPU test; with the fix set aside it fails at `:3437`.
  3. The decode switch (`in_space_long.rs`) clears a row cut left on the thread before it asks the cache, so a stale cut no longer refuses the first prefill. CUDA test `the_decode_cache_decodes_the_same_replies_through_a_window_slide`; I-0n on the pod.
  4. `DecodeForwards::first_refusal` names the kind of the first refusal that kept a group from prefilling or dropped its cache (`CacheRefusal::kind`). The same CUDA test asserts none with the cache off, and "short prompt" or "window full" with it on.
- **The census's `found_queries`:** every query whose answer it finds in TRAIN, by index, so a reading can set them aside. The old field listed at most 16 values. CPU tests `the_answer_census_counts_numbers_and_words_apart` and `the_answer_census_names_every_query_it_finds`; with the fix set aside, both fail.
- **The slide test's row-cut check** (`64475a830`), as in What did not work, item 1.
- **The sheet:** its FIT, pre-run review, run 1 note and outcome.

## Measurements

**R1 and the bins**, far words memory on, census-clean (sheet, Readings). Each panel's n, then stated at seed 1 and seed 2:

| panel | far n | stated s1 / s2 | near n | near stated | 192–511 | ≥ 512 | median far distance |
|---|---|---|---|---|---|---|---|
| p-long (dev, dev) | 306 | 0 / 0 | 13 | 0 / 0 | 0 / 0 | 0 / 0 | 1,242 |
| q-tt (train, train) | 283 | 0 / 0 | 12 | 0 / 0 | 0 / 0 | 0 / 0 | 1,256 |
| q-td (train, dev) | 287 | 0 / 0 | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1,251 |
| q-dt (dev, train) | 270 | 0 / 0 | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1,258 |

- **R1:** s_tt 0, s_dd 0 at each seed, so it fails (s_tt < 10). R1 inside each bin fails too, reported only.
- **R2:** the phrasing gap (q-tt 0 against q-dt 0) and the value gap (q-tt 0 against q-td 0) fail at both seeds.
- **R2n (numbers, reported only):** stated numbers, census-clean / all.
  - At seed 1: 0 / 0 on every panel.
  - At seed 2: p-long 1 / 1 (query 20), and 0 / 0 elsewhere.
  - The census counts numbers in TRAIN: q-tt 318 of 319, q-dt 303 of 304, q-td 10 of 309, p-long 6 of 290.
- **R3 (the reply form, memory on, reported only):**

  | panel | s1: ack / opening (shape) / other | s2: ack / opening (shape) / other | n |
  |---|---|---|---|
  | p-long | 81 / 139 (112) / 389 | 82 / 173 (153) / 354 | 609 |
  | q-tt | 11 / 386 (333) / 220 | 6 / 420 (394) / 191 | 617 |
  | q-td | 14 / 412 (353) / 184 | 9 / 427 (399) / 174 | 610 |
  | q-dt | 80 / 153 (113) / 357 | 79 / 151 (136) / 360 | 590 |

  On P2b's own pod, the sheet's test gave seed 1's p-long 82 / 136 / 391. The two pods' samples files differ in a few replies, as claude-279 found, while their recall files are identical (I-1).
- **R4 (memory on against out, far clean words, paired by query, reported only):** b = 0 and c = 0 on every panel at both seeds.
- **R5:** Tokens per second first.
- **The instrument checks:**
  - **I-0:** the float crate's CUDA library suite at the pin, `--no-fail-fast`: 339 passed, 0 failed, 8 ignored, the named tests included.
  - **I-0n:** as in What did not work, item 1.
  - **I-1:** the four re-scored p-long recall files equal P2b's sealed ones:
    - s1: `dd9096df…` (memory) and `259b4d94…` (out);
    - s2: `e85b85ab…` and `e46daceb…`.
  - **I-2:** every SHA-256 matched on the pod, for the panels, p-long, the census, the driver data and each seed's five checkpoint files.
  - **I-3 and I-5:** hold on all 16 recall files.
  - **I-6:** the second grade from `reply_text` agrees with `stated` on every query of every panel, memory on and out, at both seeds: 0 disagreements.
  - **I-4:** steps > 0 in every pass:

    | pass | prefills | steps | whole | first refusal |
    |---|---|---|---|---|
    | s1 p-long / q-tt / q-td / q-dt | 1,278 / 1,291 / 1,283 / 1,243 | 39,260 / 39,440 / 39,236 / 37,903 | 316 / 532 / 472 / 568 | window full |
    | s2 p-long / q-tt / q-td / q-dt | 1,278 / 1,291 / 1,283 / 1,243 | 39,267 / 39,453 / 39,266 / 37,914 | 316 / 532 / 472 / 568 | window full |

  - **I-7:** at each seed, `rows-dev-memory.json`, `rows-dev-no-memory.json`, `samples-memory.json`, `samples-no-memory.json` and `samples-dev-flock-memory.json` have one SHA-256 across the four passes.

## Checks run at the head

- **The head** adds to `07e898498` this REPORT, the notebook row, the item line and the sheet's outcome. No code changed after the pin `64475a830`: `07e898498` changes only the sheet.
- **The laptop gates at `07e898498`:**
  - the float crate's CPU suite passes, `--no-fail-fast`, 276 tests;
  - the stub-nvcc `--features cuda --all-targets` check passes;
  - the three CPU tests in What changed pass, and fail with their fixes' own lines set aside. The epoch and census fixes were set aside together, since they fail separate tests; the tie rule was set aside alone, since it fails the same test as the epoch. All three pass again after the restore and rebuild.
- **The binary:** `train-in-space-64475a83`, SHA-256 `dd6253ec17226f2361aab5a15dff7f6a1f54ff2c7687404dffb7a5272f48ac68`, built on the pod at the pin and sealed with the run.
- **The seal:** `SHA256SUMS.files`, SHA-256 `626f6036a24eae5787bc3564529b0e1a00246020eda122465f7c27e1e8a407a7`. It checks 269 of 269 after the copy off the pod, and 269 of 269 again after a full read-back from the store.
- **The store:** `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-p2b-classification-2026-11-04/` holds:
  - `inputs/`;
  - `run1/`, 26 files;
  - `run2/`, 273 files: the 269 sealed, the seal and its empty error file, `DONE` and `EXIT`.
- **The pod** was released and deleted after the read-back. `uor-pod status` then showed 0 running pods.
- **The grade** (`grade-281.sh`, kept with the lab's scripts) reads I-1 and I-3 to I-7 true, and the Bar Mechanism.
- **`loop report --check` and `loop pr-check --local`** were run at the head before review.
- **Reviews, each in a fresh context:**
  - the sheet's FIT: NOT FIT at round 1 (two majors, both fixed), then FIT at round 2 (#3407, 6642210770);
  - the pre-run review: incomplete at round 1 (it ran out of context; the gap it named was fixed), then GO after fixes, in four narrow parts (6642548198);
  - the run 1 note (6642885669);
  - the exact-head review of this PR, at `aa4f64ac6`, in two fresh slices:
    - **the code:** APPROVE, no blocker. It confirms that nothing can change a scored token or the cache-off path, and that each test reaches the path it guards. Its three minors and two nits are [Still open](#still-open), item 2.
    - **a claims check of every number here** against the run's files: APPROVE WITH CHANGES, no blocker. Its fixes are in this REPORT, the notebook row and the item line:
      - two overclaims: the Plain summary and R3 named the read or the trunk as the cause, which this test cannot separate from the recipe; Still open named P6 as the next change, which only the oracle ranks;
      - the rule-of-three bound, now per seed;
      - a misstated cause of the cost margin;
      - two nits.

      The items it had no budget to check were checked by hand:
      - the cited numbers of claude-267, claude-276 and claude-279;
      - the CPU suite's 276 tests and the set-asides' line numbers;
      - that I-0's job required each named test to pass;
      - the stated replies in all 16 recall files. That found query 20 stated memory out as well, now in What did not work, item 2.
  - It is recorded by `loop review`.

## Skills used

None applied.

## Still open

1. **The next read configuration.** The Mechanism branch names copy key 2 or the latest match, chosen on [claude-267](../claude-267/REPORT.md)'s oracle. That oracle scores the latest exact match, where P2b's read averages the earliest 16. On 8,243 copyable DEV positions it found these right:
   - copy key 1: 3,859;
   - copy key 2: 3,506, at purity 0.800;
   - the mixed pair `k2>k1`: 4,388, the most.

   The oracle, not this test, ranks the mixed pair highest: copy key 2, else copy key 1, at the latest match. That pair is P6's configuration (#3394's mixed copy pair). Which configuration the next training unit runs is for that unit's own sheet. This unit does not change the order of THE ORDER's remaining steps.
   - **It is a training unit:** a sheet, a FIT and a pre-run review, then a run at 2,048.
   - **This test does not separate** the read's configuration from the recipe (8,192 steps of 2,048 × 4 on this TRAIN). A miss there leaves the recipe open.
2. **The exact-head code review's points** (APPROVE, no blocker). Each only sharpens a diagnostic or a comment, and none is reached by a scored token. They are kept out of this PR, so the merged code is exactly the code the run qualified. They land with the next unit that scores with the cache:
   - `first_refusal` stays `None` when no group is ever asked to prefill: every prompt already fills the window, or every reply is one token long. Such a pass reads `prefills 0, steps 0` with no reason given.
   - "First" is per pass, and groups run shortest first, so an early group's short prompt or slide can hide a later group's refusal. A count per kind would show both.
   - The epoch comment calls it "the identity of the reads set". It counts calls to `set_hamming_reads`, so an in-place update of the reads' tensors would not bump it. Nothing does that inside a decode.
   - Two nits: a `ReadHook` name reaches `first_refusal` with its trailing comma, and a doc comment in `DecodeForwards` has a broken line.
3. **The real-text corpus** (THE ORDER step 6) still stands. This result does not clear the data; it says the trained model does not recall first, even on data drawn as TRAIN was. It does not separate the read from the recipe.
4. **The tok/s difference between pods** (137–148 here against claude-279's 98.7, with the same cache) is not explained. The kernel unit (#3678) times a cached step, and should time it on one pod type, or report the host.
5. **Then, as #2028 orders:**
   - the kernel unit (#3678): the generated kernel list first, then the measurement, then the fusion if the chains are at least 5% of a step;
   - the real-text corpus;
   - P4 (#3471);
   - P5;
   - P6 (#3394).

## Plain summary

P2b trained the model on long conversations, then asked it about something it had been told much earlier. It almost never answered right: once, at one of its two training seeds. That could mean two things. Either it can recall, but only when the question is worded the way it saw in training, or it cannot recall at all yet.

This unit asked again, this time worded exactly as in training. It still answered 0 of 283 at both training seeds. So the problem is not the new wording: the trained model does not recall the earlier note at this setting. This test cannot say whether the fault is the part that reads the note or the way it was trained. What the wording does change is the shape of the answer. Asked in the trained wording, the model answers "It's …", but with the wrong word. Asked in new wording, it often answers as if told something new: "Okay, I'll remember that."

The next step tries a different setting for the read, chosen from an earlier measurement of which setting finds the right note.

The scoring checks all held. The re-scored files match P2b's earlier ones byte for byte. The unit also hardened the scoring speed-up's safety checks. Both runs together cost about $0.61.

References #3407.
