# ORDER: one-model-end-to-end, claude-263 (served training through export: I-3 at the repaired head, then the full-length served run)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `e2470ec4d`. This ORDER was written by hand under the run file's umbrella exception; cancel probe 2 records why.

## Goal

Casey, 2 Nov 2026: "your goal is to get us to a working chat model using the training and serving methods we have described - you need to figure out how to stitch all of that together". Casey, 3 Nov, on the path claude proposed (fix served training, close claude-261, the batched scorer, the quantizer's speed, then the full-length one-model run): "Yes, please proceed on your recommended path, trying to fix these problems".

claude-262 (`fac0131bc`) repaired served training, which was missing PRODUCT_EXP. Its 256-step served runs learn again, at 1.79 and 1.84 times plain. Nothing has yet shown that a model trained as served *serves* as trained: claude-261's exports missed I-3 on 295 to 305 of 310 prompts against a bar of 8, before the repair. This unit has two parts.

- **Part A (I-3):** score claude-262's served step-256 checkpoints in float on the pod, then export them snapped and serve them in integers on the laptop. Then compare first tokens on DEV's 310 prompts.
- **Part B, only if part A passes at both seeds:** the full-length served run. Arms (b) and (k2), 8,192 steps, seeds 1 to 3, each arm alone on the card in turn. Then I-3 on every export, then R1 to R7. This is the first model on the path trained as served at full length, exported, served and scored end to end with tok/s.

**Plan step (the claim check):** this unit advances order step 4, "one model trained with all of them, exported, served and scored end to end, with tok/s".
- Part A changes no model code. It decides whether the trained-as-served model is exact through export, and every later served reading rests on that.
- Part B trains the full-length served model, (b) against (k2), and serves it.

## Cancel probe (run first; results)

1. **Already done?** No float-against-integer check of a trained served export exists at or after `fac0131bc`. claude-262's REPORT lists I-3 as not run. Not cancelled.
2. **The lock, and why this ORDER is hand-written.**
   - **The refusal**, at 09:20Z and again at 09:50:46Z on 3 Nov: `uor-harness loop claim one-model-end-to-end --lab-topic i3-repaired` exited 2 with "loop claim: refused: item one-model-end-to-end is already claimed by PR #3583 (docs/loop/tasks/one-model-end-to-end/claude-261/ORDER.md)".
   - **Its source is a merged file.** `claude-261/ORDER.md` merged with #3579 at 07:05Z. GitHub's file list for #3583 is stale: the API reports `base.sha` `75086c1d3`, DeepSeek's original base, so it lists 95 files.
   - **The no-overlap check.** `git diff --name-only $(git merge-base origin/main origin/deepseek/batched-scorer) origin/deepseek/batched-scorer` gives merge base `e2470ec4d` (main's tip) and 6 files: `crates/uor-r4-float-train/src/bin/train-in-space.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`, `docs/harness/claims.json`, `deepseek-9/ORDER.md`, `deepseek-9/REPORT.md` and `docs/research/notebook/float-training.md`. The only file in this ORDER's scope is the notebook, which the claim's overlap check skips. With a correct list, `deepseek-9/ORDER.md` would still lock the item, so this is an item-level lock with no files at stake.
   - **#3583's state.** Draft, head `90bfc731aa4be6c059988cdc0fc32ee25e47454d`, last push 08:38Z. Its own blocker, a 5090 parity run, was answered by claude at 08:40Z (#3583 comment 6599121141: PASS) and confirmed for that head at 08:41Z (6599138824). DeepSeek has pushed nothing since.
   - **The item is an umbrella.** origin/main holds 29 unit folders under it, 20 from claude and 9 from DeepSeek. deepseek-9's own ORDER was opened while #3579 held the item, and its cancel probe names #3579's shared files. That is the cross-lab precedent. claude-262 is a weaker one: its refusal named claude's own worktree.
   - **The decision.** The run file names infra-loop-harness as its example of an umbrella item. Reading `one-model-end-to-end` the same way is claude's own call, made on a fresh decision review (ALLOW with six conditions, 3 Nov 09:48Z) and logged for Casey on #2028, with a proposal to name this item in the rule.
   - **Hands off.** claude never touches #3583's branch, worktree or pods, or `docs/harness/claims.json`.
   - **E dropped.** Part A's support reading E was the batched scorer's parity. claude ran that run outside any unit as review evidence for #3583: PASS, 5 of 5 files byte-identical, 2.62 to 3.53 times faster (store `results/claude/pr-3583-parity-2026-11-03`). It decides nothing here.
3. **Inputs.**
   - claude-262's seal `results/claude/one-model-end-to-end-served-repair-2026-11-03/claude-262.tar` (SHA-256 `d04505ac64af7c9f37c60c1e26b00943cad16b3bd36aebde37541da8cabb00e3`) is in the HF store, read back on 3 Nov.
   - DEV, TRAIN and the tokenizer are at `w2-inputs/2026-10-25/driver-data`, by the pins in both sheets.
   - The panels P-w and P-v are by claude-261's pins.

   Not cancelled.

Classed mechanism-instrument: a mismatch would be a parity fault in export or serving (§4.2), not a model property.

## Coexistence with #3583

- Before every push and before going ready, the no-overlap check in cancel probe 2 is re-run. If #3583's files grow into this ORDER's scope, stop and decide in the unit; never wait.
- The notebook takes added rows only. Whichever PR merges second merges main forward and keeps both sets of rows.
- Part B leases its own cards inside the caps, never a card DeepSeek holds.

## Pre-registration

- Part A: `docs/harness/prep/one-model-end-to-end-i3-repaired-2026-11-03.md`. TEST FITNESS FIT (round 2); pod job pre-run PASS with finding 1 fixed.
- Part B: `docs/harness/prep/one-model-end-to-end-full-served-2026-11-03.md`. TEST FITNESS FIT (round 5, minors fixed); PRE-RUN PASS (`job-full.sh` `162386e0`, `watch-full.sh`).

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/claude-263/**`
- `docs/labs/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

No code changes. A fault found here is repaired in the next unit, at a named commit, as both sheets say.

## Done when

1. Part A ran on one 5090 pod at the pinned scoring head `0073cd07bd5d16021639041aa30c55132d2c6dc9`. I-0 passed, with the half-order test named, and seeds 1 and 2 were scored and snapshotted. The seal is in the store, read back, and the pod is down.
2. On the laptop, the snapped exports were made with `fbs-export --run-config` and served with `fbs-toks`. Each seed's I-3 reading was recorded: the exit code, order mismatches, first-token misses and whole-reply misses. The sheet's bar was applied: a pass needs exit 0, 0 order mismatches and at most 8 misses of 310.
3. If both seeds pass, part B ran by its sheet, with three pods and arms in turn, sealed and read back. I-3 ran on every export. The table was applied: R1 to R5 are held unread if any I-3 fails, and R6 and R7 are read only on exports that pass I-3. Decode tok/s is reported first.
4. If part A fails at either seed, part B does not run in this unit. The REPORT names the seed, the readings and the next unit, which localizes the fault with the existing tools.
5. REPORT with negatives first; one notebook row; the item's "What is missing" line updated.

## Stop and ask if

Each is decided in the unit, never waited on:
- **An I-3 failure.** Follow the sheet's row: no repair in this unit.
- **Part B projects past its sheet's cost** (about $15, at most $2.40/h, 6 h leases). Re-scope in a new ORDER rather than trim arms.
- **A pod or lease fault.** Replace the pod and record it. A healthy run is never cancelled for a self-set budget.
- **#3583's files grow into this scope.** Decide in the unit per "Coexistence" above.
