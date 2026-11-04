# ORDER: one-model-end-to-end, claude-270 (batched decode by default, after current-code GPU parity)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `8fb9f3719` (main with claude-269 merged). Draft PR #3629.

## Goal

The float scorer is now the slowest part of a training run. In claude-266, `--score-rows` took 3,980 to 4,211 s per arm at seeds 1 and 2, against about 970 s of training. Its greedy decode runs one prompt at a time unless `--batched-decode 1` is passed.

DeepSeek's #3583 (deepseek-9) built the batched path: prompts of exactly equal length decode together, and every other case stays serial. On one RTX 5090 checkpoint (claude-262's served seed-1 step 256), serial and batched gave byte-identical output files, and the whole `--score-rows` fell from 581 s to 180 s. It stays off by default for two reasons. The group-boundary fix came after that run, and no current-code rerun was made. deepseek-9's Still open asks for parity at another seed before batching becomes the default.

This unit reruns that parity on current code with claude-266's final checkpoints: two arms ((b) and (k2)), two seeds, DEV and both panels. If every file is byte-identical, batching becomes the scorer's default, and `--batched-decode 0` stays the serial reference.

**Plan step (the claim check):**
- **Order step:** step 4, one model trained, exported, served and scored end to end. Scoring is the largest time cost of each training run, so it is what the bigger run's sheet pays most for. This is also Casey's direction of 3 November: "especially anything that reducing training times".
- **What it changes in the model:** nothing in training or serving. Every trained model is scored faster, with the same files. The training run's checkpoint replies stay serial: a handful of prompts each, not worth a change.

**Why not a streaming decode first.** A carried-state decode is a separate, larger unit. It needs a CUDA scan entry that starts from a carried state (`cuda_stack.cu`) and a key cache in `flock_read.rs`, both outside this item's scope. The batched path's own numbers also show each decode step is launch-bound: a group of about 4 prompts costs about what one prompt costs. So streaming one prompt saves little until the batches are wider. Wider batches across prompt lengths are the next speed unit. Its design question is whether left padding can be made exact in the recurrence.

## Cancel probe (run first)

1. Is batching already the default on main? Expected no: `let mut batched_decode = false;` in `bin/train-in-space.rs`.
2. Has the batched path changed since deepseek-9's parity run? Expected yes: the group-boundary fix, plus claude-267 to claude-269's copy-key changes in the Hamming read, which both paths share. This is why the parity is rerun on current code.
3. Are claude-266's final checkpoints in the store? `results/claude/one-model-end-to-end-full-unsnapped-2026-11-03/s{1,2}/full/out/{b,k2}/models/step-008192/`, and its serial scored files `.../out/{b,k2}-scored/`.
4. Is an RTX 5090 offer up, inside the caps (at most 4 pods and $8/h across all labs)? `uor-pod status` first.

**Result (3 Nov, ~23:55Z): go.**
1. No: `train-in-space.rs:238` sets `let mut batched_decode = false;`.
2. Yes: `in_space_long.rs` and `flock_read_hamming.rs` changed after the parity run's base `6392cdcce` (claude-268, #3624).
3. Yes: `step-008192` holds 5 files for each of s1 and s2, (b) and (k2). The stored serial scored files are under `s1/.../b-scored` (15 files) and `s2/.../k2-scored` (15 files).
4. Yes: 0 pods running ($0/h of $8/h). There are 15 RTX 5090 offers, the cheapest at $0.563/h.

Classification of the probe: instrument (the unit's own preconditions).

**Classification of the GPU run.** It is a finite equality test of two code paths, the "finite kernel tests" case of §4.5, not a training or evaluation run. No bar, model claim or milestone rests on it: it reads no grade and decides no arm. So it needs no prep sheet. It still goes through a fresh-context pre-run review of its job script before any pod starts (cost, seal, watcher, what each outcome unlocks), recorded in the REPORT.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-270/**`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs` (the default and its doc lines)
- `crates/uor-r4-float-train/src/in_space_long.rs` (doc lines of `score_rows` and the batch path only, if they name the default)

Local, not committed: `~/uor-r4/local/claude/claude-270/` (job and watcher scripts, pulled outputs, logs).

**Scope decision (4 Nov, this lab's top session).** The item's scope gains `crates/uor-r4-training/src/milestone_world_v2.rs`, for the next unit. The guard reads scope from `main`, so the line has to merge before that unit can be claimed, as claude-268 did for claude-269.
- **Casey, 4 Nov:** "please make all of those points your current plan and goal for next steps". Step 2 of that plan is a long-conversation corpus.
- **Why it is needed:** every corpus and panel stops at 245–258 tokens, because the generator caps episodes at `CONTEXT = 256` (`milestone_world_v2.rs:96`).
- **Why this item:** the item's scope already holds the corpus writer (`m-world-recall-corpus.rs`) and `mix-chat-corpus.rs`; only the generator was missing. The corpus is this model's training data, so it belongs here rather than in a new item.
- **Changed here:** one frontmatter line. No code changes in this unit.

## Done when

1. **The default:** `--score-rows` decodes batched unless `--batched-decode 0` is given. The flag's doc lines say so, and a test pins the default.
2. **A negative gate:** with the flip set aside, the default test fails. Restore, and it passes.
3. **GPU parity at the code head.** One RTX 5090 pod per checkpoint, (b) seed 1 and (k2) seed 2, running claude-266's exact scoring command (`--samples 1536 --reply-tokens 64 --retain-rates 1 --panel p-w=… --panel p-v=… --seed S`) twice: with `--batched-decode 0` and with `1`.
   - **Pass:** the sha256 of every output file matches between the two runs. Wall seconds for each run are recorded.
   - **Why bytes suffice:** the scored files hold no timing. Each decode's wall seconds go to the `ScoredFile` manifest the scorer prints to stdout. claude-266's 15 stored files, from this same command, contain no `decode` or `search` field. The pre-run review's round 1 (BLOCK) read the manifest's timing as part of the files; the stored files disprove that. The two stdout manifests are also compared with the seconds dropped (descriptive), and their seconds give the speed reading.
   - **Also compared, descriptive only:** the serial run against claude-266's stored scored files for the same checkpoint. That shows whether claude-267 to claude-269 changed the decode at all.
4. **The CUDA float library suite** passes on each pod at the code head before scoring, including `exact_length_batches_match_one_at_a_time_decodes`.
5. **A REPORT, negatives first,** leading with the decode's tokens per second, serial against batched. Plus one notebook row and the item paragraph.

## Stop and decide (never stop and wait)

- **Any output file differs:** the default does not change. Find the first differing file and row, classify it (instrument or mechanism), and record it. The unit ends with that negative and the flip reverted.
- **No RTX 5090 offer, or one that fails to start:** take the next 5090 offer. If none appears within the hour, run on the cheapest card of the same CUDA architecture family the training runs used, and say so in the REPORT.
- **A pod's scoring takes past 3 hours:** the run is still healthy. Extend the lease and note it; never cancel it on a self-set budget.
