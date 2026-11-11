# ORDER: one-model-end-to-end, claude-348

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `0f177b51adbc245c2d99d9e4b4aa0c4410ed8b89`.

## Goal

**Make the served model read its prompt faster without changing one reply.**

**Why now.** The outside review Casey passed on (10 Nov) named two serving costs, and Casey's answer to "Start the seven no-pod fixes now?" was "Yes, start". [claude-347](../claude-347/REPORT.md) delivered the five readings; these are the two that change serving code. The project's lead metric is tokens per second (owner, 12 Oct).

**The steps, in order. Each measurement comes before the code it would justify.**

1. **Measure.** On arm r's artifact, the share of one served step spent choosing the next token (the whole-table nearest scan: 4,096 rows of 256 values), with the stage clocks `main` already has (`fbs-toks --stages`).
2. **Stop choosing a token nobody reads.** The chat steps every prompt token through `serve_step` and keeps only the last pick (`StackChat::feed`). A reading step runs the same kernels, leaves the same session state and skips the scan. Built only if step 1 shows the scan is a material share.
3. **The exact cell search.** Count the share of the table an exact pruned search would visit on real queries. Build it only if it prunes; report the count either way.
4. **`uor-chat --prompt-ids` and the reply mark.** Decide whether the raw-id path takes a mark, now that `raw-id-replies` exists.

**Proof for step 2.** Replies byte for byte equal to `main`'s: the 1,141 panel prompts at three seeds through `raw-id-replies` (marked and unmarked) against claude-347's checksums, and the registered scripts through `uor-chat`. Tokens per second before and after on arm r, same machine, same run. The step's multiply audit still passes.

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`): the model is opened on a laptop, and a turn's wait grows with the conversation because every turn reads the whole history.
- (b) **What it changes in the model.** Nothing. Same artifact, same replies; less work per prompt token.

**Rules check** (CHARTER §3). Integer serving only; no multiply, float or squashing function is added; no training; laptop only (host checks and finite kernel tests, §4.5). No neural comparator.

**Not in scope.** Any pod. Any mechanism arm: the hold of 10 Nov stands. The artifact format. DeepSeek's contracts under `conformance/stack/`. `crates/uor-r4-cpu-train`. The timed copy of the step (`stack_step_fbs_timed`) stays a full step.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a step on `main` that reads a token without choosing the next one, or a pruned nearest search.
- **Result (at claim):** none. `git grep -n -E "fn read_step|fn serve_read" origin/main -- crates` finds nothing; `stack_whole_table_nearest` is the only nearest search and it scans every row.

## Claim

Written by hand, as claude-347's was: `uor-harness loop claim` refused this item on 10 Nov because claude-346 (PR #3913, a draft under the hold) holds it. claude-346's worktree was removed today to stay inside the disk floor; its branch `claude/e8-address` is unchanged on origin.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-348/**`
- `crates/uor-r4-integer/src/stack/{session,chat,tests}.rs`
- `crates/uor-r4-integer/src/bin/{uor-chat,fbs-toks}.rs` (only if step 4 or a reading-step clock needs it)
- `docs/labs/one-model-end-to-end/instrument/**` (only if the proof needs a reader change)
- `docs/research/notebook/**` (one row a result)

## Inputs, all on the laptop

- Integer artifacts: `local/claude/claude-345r/art/r-s{1,2,3}-c8256.uorlut`; arm rt's under `local/claude/claude-345t/`.
- claude-347's reader outputs and checksums: `local/claude/claude-347/{fi2,fi3}/`.
