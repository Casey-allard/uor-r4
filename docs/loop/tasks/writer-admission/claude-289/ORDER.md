# ORDER: writer-admission, claude-289

Lab: claude. Item: [writer-admission](../../../../roadmap/items/writer-admission.md). Base: `ef620df0a`.

## Goal

The item's done-when reads:

> 1. **A diagnosis of seed 2's failure,** from saved checkpoints: the ports' scores against the roots, the cells admitted, and how they move over training. The first unit may be this alone, on the sealed claude-150 checkpoints.
> 2. **A geometric change to admission,** for example a margin the 16 roots' spacing allows, cells kept by the target's own ports, or a kept count chosen by the margin. It ships with host tests, and with CUDA tests where tensors are involved.
> 3. **A pre-registered run** (a sheet with a fresh FIT, CHARTER §4.5): claude-152's (a) recipe plus the change, two seeds, the writer and whole-table nearest read on the same checkpoints.
>    - **Bar:** the writer within a pre-registered margin of nearest at both seeds.
>    - **The cost per token** of each decoder, measured.
> 4. One notebook row and a REPORT.

**This unit is done-when 1 alone,** the diagnosis on claude-150's sealed checkpoints, in the order Casey's call on #3394 set ("Yes, add both", 2 Nov; owner log 6585646629): the diagnosis first.

**Claim check.**
- (a) **The step:** P6 of Casey's 4 Nov plan (PLACE.md: "the mixed copy pair's GPU run, then writer admission (#3394)"), and step 4 of the chat path, which #3394 joined on Casey's call.
- (b) **What it changes in the model:** nothing yet. It is a diagnosis that unblocks done-when 2, the admission change. That change makes the served readout the cheap one: whole-table nearest costs 486 µs a token on the M1, against the writer's 23 µs (claude-158). It is also 484 of a 744 µs serving step at short context (claude-288's parts).

## Cancel probe (run first)

Is the diagnosis still needed, and are its inputs there?

1. Is another open order or a notebook row already answering it? (`gh pr list`, `docs/loop/tasks/writer-admission/`, the float-training notebook.)
2. Are claude-150's checkpoints sealed in the store? Check 17 per seed, with their run `config.json` and `progress.jsonl`.
3. Is the writer's admission rule (`native_vector_writer.rs`) unchanged since claude-150's code `074d54b2`? If so, a diagnosis of those checkpoints reads the rule the item has to fix.
4. Does the logged reading still show seed 2's failure?

**Result (5 Nov): go. The failure is the model's, not an instrument's.** claude-150's I-2 held with gap `0`.
1. **No other answer:** the only open writer-admission PR is this unit's draft, #3722, and this is the item's only task folder. No notebook row after claude-150 reads admission.
2. **The inputs are sealed:** each seed's SHA256SUMS lists 17 `run/models/step-*/model.safetensors` plus the run's `config.json` and `progress.jsonl`. For example, seed 2's step 32,768 is `2599a6bf3f85eae656a29a6efb83e2c25c40e8e0b1029edda00bdc1d73a88ff6`.
3. **The rule is unchanged:** `git log 074d54b2..origin/main -- native_vector_writer.rs` lists no commits. The rest of the crate moved by about 31,000 lines, `hamilton_sheet.rs` and `in_space_long.rs` included, so I-2 (the panel reproduces the logged counts) is a real check.
4. **The logs still show the failure, and give a lead.**
   - At seed 2, `dev_admitted_hits` stays at 208–714 from step 2,048 on, against 7,759–9,563 at seed 1.
   - From step 4,096 to step 32,768, seed 2's `dev_address_margin` reads `1.0000` to the fourth decimal (`1.000030` at step 4,096, `1.0000007` at step 32,768). That is the mean of `max(0, rival − target + 1)`. Seed 1 reads `0.87`–`0.98` over the same span.
   - So at seed 2 the best kept rival cell scores the same as the target's cell, on average to within `1e-5`. All 256 cell scores may tie. If they do, the writer's tie rule keeps cells 0–7, and only targets in those cells are admitted.
   - The notebook's 27 Oct exploration row recorded the same reading under the writer objective at lr 1e-3: "the model flattened its ports until the hinge read exactly 1.0000".
   - The sheet pre-registers this "flat ports" reading among the possible ones.

## Files in scope

- `crates/uor-r4-float-train/src/native_vector_writer.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `docs/harness/prep/writer-admission*`
- `docs/loop/tasks/writer-admission/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/writer-admission.md`

## Done when

1. **The instrument.** `train-in-space --inspect` reads, at every saved checkpoint and over all of DEV's scored rows:
   - each port's scores against the 16 roots;
   - the cells admitted;
   - where the target's cell and the target's two roots rank.

   Each port is the query's first or last 8 coordinates. The instrument ships with:
   - **a CUDA test** on a synthetic query whose readings are known;
   - **host tests** of the counts' merge;
   - **instrument check I-1** in code: the reading's admitted count equals the DEV panel's `admitted_target_hits` at each checkpoint, or the inspection fails.
2. **A prep sheet,** `docs/harness/prep/writer-admission-diagnosis-2026-11-05.md`, with a fresh FIT and a D67 pre-run review, both posted on #3394.
3. **The pod run:** `--inspect` over the 17 checkpoints of both seeds. **Instrument check I-2:** the panel reproduces claude-150's logged `dev_top1_hits` and `dev_admitted_hits` at every step. The results are sealed in the store with a SHA256SUMS.
4. **The write-up:**
   - **a REPORT, negatives first:** which pre-registered reading the measurements support, and the change it points done-when 2 to;
   - one row in the float-training notebook;
   - the item file's "What is missing" updated with the diagnosis. The status stays `ready`, because done-whens 2–4 remain.

## Stop and ask if

Stop and decide; never stop and wait.

- **I-2 fails at this unit's build.** The current forward does not reproduce claude-150's checkpoints, so that is an instrument fault. Build the instrument on `074d54b2` (an evidence branch with this unit's instrument commits) and probe again. Never read a diagnosis from a build that fails I-2.
- **The pod run projects past $3 or 2 hours.** Re-plan the chunking first.
- **The readings fit none of the sheet's pre-registered readings.** Report them as unexplained, with the numbers, and claim no change.
- **The work needs a file outside the scope.** Stop, and widen the scope by its own PR.
