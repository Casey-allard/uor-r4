# ORDER: one-model-end-to-end, claude-261 (the copy read's key: two tokens, trained as served)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `eac5c1d90`.

## Goal

Casey, 2 Nov 2026: "your goal is to get us to a working chat model using the training and serving methods we have described - you need to figure out how to stitch all of that together". Asked "Trace the copy read on conv-a to confirm the cause before I build the new key?", Casey answered: "um yes I think so". The trace is done ([claude-259's REPORT](../claude-259/REPORT.md), "Also found"; the trace file is in `docs/labs/one-model-end-to-end/capacity-2026-11-02/trace/`). claude-260 (#3578, `eac5c1d90`) widened the item's scope to the files the key changes.

**Plan step (the claim check):** path step 4 of the lab's order (`local/claude/onboarding/run-claude.txt`): one model trained with all of its parts, exported, served and scored end to end. The served chat answers "your friend is 47" because of a fault the trace confirmed. **What it changes in the model:** the copy heads' key, in training (`flock_read_hamming.rs`), in the artifact (`stack/format.rs`, `fbs_snapshot.rs`, `fbs_export.rs`) and in serving (`stack/hamming.rs`). It is the first change to what the read can find since the read was built (claude-146).

**The fault, measured.**
- A copy head's key at `s` names only the token before `s`. Its query at `t` names only `x_t` (`flock_read_hamming.rs`, module doc, points 1 and 5).
- In conv-a turn 3, at the reply's " is" (position 80), both copy heads at both depths keep the five positions that follow an earlier " is" (8, 26, 37, 51, 64). They read the mean of what follows. Maya (64) is found but cannot be preferred. The served next id was the space before 47.
- A key on (" friend", " is") matches only position 64.

**The change.** Copy key 2 (the ordered two-token key):
- A copy head's 64-bit code holds two token codes. Its first half is from the token before `s`; its second half is from the token two before `s`, through the same key rotation.
- The query takes its first half from `x_t` and its second half from `x_{t−1}`.
- The distance is still one popcount, and it is exactly the sum of the two tokens' half-distances. So an exact two-token repeat is distance 0. Where only the last token repeats, the distance is about a quarter of the head's bits on average, which is the copy heads' starting margin.
- Type heads, weights, margins, the payload and the tie rule are unchanged. An artifact with copy key 1 stays byte for byte as before.

**My decisions (logged on #3407, reversible by Casey):**
- **TRAIN stays the pinned corpus.** claude-259's REPORT also named memory lines in TRAIN (`recall=oracle`). Changing the corpus and the key in one run would leave either result unreadable. So the memory-line corpus is its own unit, and this run changes the key only.
- **The slot key (gcd over a clause's content atoms) comes later.** It needs entity and attribute atoms that the trunk does not mark. The two-token key fixes the measured case with no new parameters. If the two-token key leaves the copy fault, the slot key is the next unit.
- **DeepSeek's FIT four-run comparison folds in** ([sheet](../../../../harness/prep/one-model-end-to-end-served-comparison-2026-11-02.md), deepseek-5, `NOT_RUN` for lack of a GPU). It tests plain against served training, at two seeds and 256 steps each, for about $1.50. This run trains with that served form, so the comparison is its throughput probe. It runs on seed 1's pod first, read by its own sheet's bar.
- **`fbs-toks --copy-codes` (claude-237) stays copy-key-1 only.** Its bin is outside the scope. Under copy key 2 the vocabulary codes it checks are still exactly the halves the key is built from, but a trace's query is a pair of them, so its query comparison does not apply. The REPORT names it; this run does not use it.

## Cancel probe (run first; results)

1. **A two-token copy key already?** `git grep -i -e copy_key -e "two-token key" -- crates docs/mechanisms` finds none for the Hamming read: the hits are the n-gram tables' two-token keys (`uor-r4-graph-format/src/ngram.rs`, the old table model) and a binding probe. `docs/mechanisms/exact-memory.md:86` lists "Ordered n-let keys" as not built. Not cancelled.
2. **Another lab on these files?** `gh pr list --state open` shows only this draft (#3579). Not cancelled.
3. **Inputs:** TRAIN, DEV, the tokenizer, p-w and p-v are in the HF store at the sheet `one-model-end-to-end-capacity-2026-11-02.md`'s paths and sums. The base recipe is that sheet's (b). Not cancelled.

Classed instrument (no model ran).

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/labs/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs` (added at I-0: the served activation quantizer's dtype repair, below)
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`

**Found at the exact-head review, moved out of this unit.** The CPU trainer's port (`crates/uor-r4-cpu-train/src/fbs.rs`, `CpuFbsForward::from_artifact`) would serve a copy-key-2 artifact with one-token keys. A decision card to add that file here passed its round-4 review. But the guard reads an item's scope as it is on `origin/main`, so a branch cannot widen its own scope. So the refusal, with its test, is the next unit, under the CPU trainer's own item, right after this one merges.

## Done when

1. **The key, end to end.** `--hamming-copy-key 1|2` (default 1) trains the key above. It is saved in `hamming_read.safetensors` and `config.json`, carried by `FbsHammingSnapshot` and `StackHamming` as `copy_key`, and served by `CopyHeads`. The tests show:
   - (a) on a window holding two facts with the same last token, a copy-key-2 head keeps the right position at distance 0, ahead of the other; under copy key 1 the two tie. In the float read (CUDA, run on every pod as I-0) and in the integer read (laptop);
   - (b) the integer read under copy key 2 equals the f64 reference of the float read's steps (claude-189's test, extended), and the trained export's first tokens agree with the float file within the sheet's I-3 bar;
   - (c) a copy-key-1 artifact, snapshot and `config.json` are byte for byte as before;
   - (d) a copy key other than 1 or 2, or copy key 2 on a head whose halves do not fall on whole lanes, is refused.
2. **The sheet** `docs/harness/prep/one-model-end-to-end-copy-key-2026-11-02.md`, one page, with the shape the harness reads (Base artifact, Data split, Bar, Steps, Minimum test power), a fresh FIT, and a fresh pre-run review (D67) on #3407.
3. **The run** on Vast RTX 5090s inside the caps. Arms (b) and (k2), at seeds 1–3, 8,192 steps each, trained as served. DeepSeek's four runs go first on seed 1's pod. The outputs are sealed in the HF store with `SHA256SUMS`.
4. **Scored and served:** the sheet's readings and decision table. Each arm's seed-1 export is served on conv-a, conv-b and conv-c, with transcripts and decode tok/s.
5. **The REPORT**, negatives first, with one notebook row. It also corrects claude-259's REPORT, which cites deepseek-3 under #3565 (it landed as #3563).

## Found at I-0 (seed 1's pod, head `af7154c00`)

The whole CUDA library suite failed 2 of 291. Both failures are in #3563 and #3565's served quantization, which had not run on a GPU before.
- `activation_quantize16` (`hamilton_sheet.rs`) built its two scale factors from untyped `0.5` and `1.0`, which are F64 tensors. Multiplying them into the F32 scale refuses: "dtype mismatch in mul, lhs: F32, rhs: F64". Every forward under `--train-as-served 1` runs through it, so both arms would have failed their smoke. The function is compiled only with `cuda`, so laptop tests never reached it.
- Its CUDA test built its activations from untyped literals too (F64).

Classed instrument: a fault in the served path both arms share, not a reading of either. It is repaired in this unit (the factors take the activation's dtype; the test's literals are `f32`). The values are unchanged: 0.5 and 1.0 are exact in both types. No training had started. The two tests are the repair's own check: they failed at `af7154c00` and run again as I-0 on every pod.

## Found in the main run (seed 2's pod, `vast-58895486`)

Seed 2's (k2) ran out of GPU memory at 01:12Z, 1,612 s into its 2,048 steps, with `Error: Candle(DriverError(CUDA_ERROR_OUT_OF_MEMORY, "out of memory"))`. Its only checkpoint is step 0's. Seed 2's (b) trained to its end, in 2,371 s.
- The pod's GPU log (`gpu.csv`, once a minute) shows the two arms' memory together. It climbed from 23,611 MiB at 01:06Z to 31,323 MiB at 01:12Z, against the card's 32,607 MiB. It fell to 7,888 MiB once (k2) stopped.
- Seeds 1 and 3 share their cards the same way, and their peaks so far are 28,603 and 29,275 MiB.

Classed instrument (CHARTER §4.2, L-007): the shared card ran out of memory. That says nothing about (k2)'s read. The sheet's stop rule covers a diverged loss and an export refusal, but not a resource fault, so this case is decided here, on claude's decision review:
- An arm that runs out of GPU memory while it shares the card trains again at its seed, alone on a fresh pod. It runs at the paired arm's step count, through `job.sh SHA SEED ARM STEPS`.
- Its seed is read once it has a checkpoint.
- If it runs out alone too, it is absent at that seed (row 4), and the REPORT names memory as the cause.
- The failed run stays as written under its pod's sealed root. Nothing of it is scored.

Seed 2's (k2) trains again on `vast-58901106`, at 2,048 steps, from 01:29Z (`job.sh 36ca53c81195969f9487747163cc13c50d145efa 2 k2 2048`).

Seed 1's (b) ran out the same way on `vast-58894902`, at 01:52Z, 3,210 s in, after its step-1,024 checkpoint. The card held 30,555 MiB at 01:44Z. Its memory fell to 25,915 MiB at the step-1,024 eval, then climbed again to 29,819 MiB at 01:51Z. Seed 1's (k2) trained to its end. The same rule applies: (b) trains again at seed 1, alone on a fresh pod, at 2,048 steps, once a pod is free under the cap. Its step-1,024 checkpoint is never scored. It trains again on `vast-58912154` from 03:34Z (`job.sh 36ca53c81195969f9487747163cc13c50d145efa 1 b 2048`), in the slot seed 2's shared pod left. That pod's outputs were sealed (EXIT 6, the missing (k2)), stored as `s2/s2.tar` and read back, sha256 matched, before it went down.

While two arms share the card, their memory together climbs through training, by about 1,000 to 1,300 MiB a minute, and falls back at times. Trained alone, (k2) at seed 2 held 5,040–7,216 MiB through its whole 2,048 steps (9,424 MiB at the final eval), and trained in 1,565 s, against 3,182–3,284 s for each shared arm. So sharing caused the climb, and it bought no speed: two arms trained one after the other on one pod take about the same time. The REPORT names this for the next unit: arms train one after the other, and I-2 projects each arm alone (6,144 steps fit in 90 minutes alone, against 2,048 shared).

Owner decision, 3 Nov 2026 at about 02:40Z. Asked "Which fix should the next runs use?", Casey answered, verbatim: "Arms in turn". From the next unit, every pod job trains (and scores) its arms one after the other, never two on one card, and I-2 projects each arm's steps from its own throughput alone. No memory trace is run to find why sharing climbs. This run keeps its design: seed 3's shared arms stand, and every rerun trains alone.

Owner decision, 3 Nov 2026 at about 02:50Z, on the scorer's speed. Scoring one arm took over an hour on each pod: one CPU core at 100%, the GPU 57–97% busy (mean 71%). The greedy decode runs one sample at a time and runs the forward over the whole window for each reply token. Asked "Should a batched scorer be the next unit after this run?", Casey answered, verbatim: "Yes, scorer next". So the unit after this one decodes a panel's samples together, with one batched forward a token, and must match the one-at-a-time decode exactly.

Found at seed 2's grade (04:11Z): under main's `--train-as-served 1` (#3565's forward surrogates), both arms learn far less per step than claude-259's runs did before #3565.
- At step 2,048, DEV nearest is 921–2,314 of 14,287 here, against claude-259's 11,357 and up at the same step.
- deepseek-5's 256-step runs at this head isolate it to the served form: 20–23 nearest hits served against 225–229 plain, and 355 s against 51–52 s to train.

Owner decision, 3 Nov 2026 at about 04:30Z, on the order of work after this finding. Claude recommended:
1. Fix the served-training fault now, in its own session (claude's session "Fix why trained-as-served learns 10x less per step", which Casey started).
2. Close this unit, then the batched scorer, in this session.
3. Then the served quantizer's speed, and the full-length one-model run.

Casey answered, verbatim: "Yes, please proceed on your recommended path, trying to fix these problems".

## Stop and ask if

Each of these is decided in the unit, never waited on:
- **The run needs a new instrument, trace or audit.** That is outside the track's bound: bring it to Casey.
- **An arm's smoke fails in the trainer or the exporter.** If (k2) fails, the code is repaired in this unit before any long run. If (b) fails, the served form itself is broken: stop the pod and record it.
- **The projected cost passes the caps**, or an arm projects past about an hour of training: cut its steps equally for both arms, and record why. (Amended at the sheet's FIT: past 90 minutes, claude-259's limit for this recipe, since its (b) and (h) took 33–52 minutes sharing a GPU.)
