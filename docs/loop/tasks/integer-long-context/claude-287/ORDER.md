# ORDER: integer-long-context, claude-287

Lab: claude. Item: [integer-long-context](../../../../roadmap/items/integer-long-context.md) (#3471). Base: `d6c2c3ecd`. Plan step: P4, unit 2 of 3 ([claude-286's ORDER](../claude-286/ORDER.md)).

## Goal

The item's done-when reads:

> 1. **The decision,** recorded by a decision review on this item's issue (`[claude] owner log:`): which rule, and why, read from the measurements in 2.
> 2. **The measurements,** under a prep sheet with a fresh FIT and a pre-run review: on one trained checkpoint, the float model's held-out loss and greedy reply agreement past 192 tokens under each rule, on multi-turn prompts whose context passes 192. A rule needing a new training arm gets that arm.

This unit delivers done-whens 1 and 2. Done-whens 3 to 5 are claude-288's.

## Cancel probe (run first)

Read at `d6c2c3ecd` (`origin/main`) by two read-only scouts and checked here.

1. **No rule past the window exists yet.**
   - `git grep -n -e past_window -e past-window d6c2c3ecd -- crates` finds only `QueryPrompts::past_window` (`in_space_long.rs:5144`), the queries whose reply is longer than the window. Those are counted, not decoded.
   - `score_rows` refuses a `--context` past the checkpoint's window (`:7165-7170`).
   - The new code names its rule `PastWindowRule`, so that it does not clash with that field.
2. **The scorer counts hits; it reports no loss.**
   - Each scored row records the whole table's nearest token and the served writer's pick (`ScoredRows`, `in_space_long.rs:6797-6815`).
   - `batch_loss` → `query_loss` (`:1116-1150`), the checkpoint's own objective, is called only in training.
   - So "held-out loss" in done-when 2 needs one new reading: `query_loss` on the scored rows, grouped by distance. It adds no new arithmetic.
3. **Blocks is what the scorer does today.**
   - A panel's conversations are cut by `build_document_windows` at the scored `context` (`in_space_long.rs:7090-7148`; `hamilton_sheet.rs:1870-1923`).
   - Each cut is a back-to-back chunk of W targets, with no state carried and no `<|bos|>` at the cut.
   - So the blocks rule is the base path, run at W.
4. **Carry needs no new forward.**
   - No weight is sized by the window. `StackModel::set_context` changes only the field, and refuses any change that would reshape a weight (`geometric_stack_native.rs:1189-1198`).
   - The trunk's scan runs over the input's length (`geometric_stack_native_cuda.rs:604-633`), and the Hamming read's causal set is built from that length (`flock_read_hamming.rs:41`).
   - So a checkpoint trained at W scores a whole conversation in one forward after `set_context(C)`, as claude-242 did by hand for the integer artifact. The cap is C ≤ 4,096 (`geometric_stack_native.rs:953`).
   - The Hamming read's memory grows with the square of the length: (batch × time, time). It refuses at 2³² elements (`flock_read_hamming.rs:1545-1561`); one row at 4,096 holds 16.8 M.
   - One shift from training: the type-spin frame runs from the start of the window (`flock_read_hamming.rs:1150-1155`). Under carry that is the start of the conversation. This is what carry means, and it is recorded, not fixed.
5. **Slide exists only in the decode.**
   - `query_prompts` (`in_space_long.rs:5152-5201`) cuts a W-token prompt ending at each query's reply, capped at the conversation's `<|bos|>`.
   - The greedy decode slides over W (`:5849, 5860, 6124`).
   - No teacher-forced slide exists.
6. **The window gates.**
   - `row_stride_window` refuses 193 to 1,536 (`in_space_long.rs:5080-5093`), because more than one window would share a forward there.
   - Every arm here scores at 192 (16 windows per forward) or at 2,048 and over (one per forward), so none meets the gate.
7. **The inputs exist.**
   - **a7r, the served model:** the R arm of `training-460fe46e` (`results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/sN/r-drop-sN/`, claude-243). It was trained at 192.
   - **P2b's (L):** trained at 2,048 (claude-276, `results/claude/one-model-end-to-end-long-context-2026-11-04/sN`).
   - **p-long:** 160 conversations up to 2,048 tokens, 609 queries at distances up to 1,994 (claude-275, `…-long-context-2026-11-04/inputs/p-long`).
   - **The long TRAIN** for the census (`…/inputs/train-long`).

**Result:** not cancelled. The class is **instrument**: the float scorer cannot read past a checkpoint's window, so no rule has been measured. Nothing here is a reading of the model.

## Files in scope

- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/point_key.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/memory.rs`
- `crates/uor-r4-integer/src/stack/dialogue_memory.rs`
- `docs/model/pipeline-map.md`
- `docs/harness/prep/integer-long-context*`
- `docs/roadmap/items/integer-long-context.md`
- `docs/loop/tasks/integer-long-context/**`
- `docs/research/notebook/m4-serving.md`

This unit edits only these files:
- **`in_space_long.rs`:** the `--past-window` option in `score_rows`, the distance bands and the per-rule query prompts.
- **`hamilton_sheet.rs`:** the sliding windower, beside `build_document_windows`.
- **`bin/train-in-space.rs`:** the flag and its header line.
- **`geometric_stack_native.rs`:** only if `set_context` needs a test.
- **The prep sheet.**
- **The item file:** its last commit returns `crates/uor-r4-integer/src/stack/session.rs` and `stack/mod.rs` to the scope, for claude-288.
- **The task folder and `m4-serving.md`.**

The integer files stay untouched until claude-288.

## Done when

1. **`--score-rows --past-window slide|blocks|carry`.**
   - It cuts panels only, and is refused with Fresh V2. DEV, which every `--score-rows` run scores, keeps its 192-token windows as the run's identity check: under slide and blocks its rows are the base's, and under carry they run on the model at C. (Decided at the wiring: a refusal on DEV would refuse every run.)
   - It is also refused with the opening keys, the served and keyed arms, and the decision traces and forces, which this measurement does not read.
   - W is the checkpoint's own window, read from its `config.json`; a `--context` other than W is refused. Without the option, every output and every refusal is byte for byte as at the base.
   - **blocks:** `build_document_windows` at W, as today.
   - **slide:** a new windower over each conversation:
     - the first window scores targets 1 to W;
     - each later window holds W inputs and scores only its last s targets, so every target past W sees from W − s + 1 to W tokens;
     - s is a flag, recorded in the output.
   - **carry:** `set_context(C)` after load, with C the panel's longest conversation rounded up to 2,048 or 4,096. Then `build_document_windows` at C, which gives one window per conversation.
   - **Queries,** for greedy replies and recall. The prompt ends at the reply, and its start depends on the rule:
     - slide: today's `query_prompts`, the W positions ending at the reply's last target, cut at the conversation's `<|bos|>`;
     - blocks: the start of the reply's block, a multiple of W from `<|bos|>`;
     - carry: the conversation's `<|bos|>`.

     The decode runs at the model's context: W for slide and blocks, C for carry. A blocks reply that crosses its block's end slides from there. This is recorded as the one way the decode departs from blocks.
   - **A new output,** `past-window-{split}-{memory|no-memory}.json`, records:
     - the rule, W, C and s;
     - for each band of distance from the conversation's `<|bos|>` (`[0,W)` and then bands of width W), the targets, the nearest hits, the writer hits, and the summed and mean `query_loss` under the checkpoint's objective;
     - for each band of query distance, the replies, the token agreement with the reference and the recall grade.
2. **Tests, each with a negative gate where it pins a fix:**
   - within the first W targets, the three windowers give identical rows;
   - the slide windower covers every target once, and each sees at least W − s + 1 tokens;
   - carry's windows are one per conversation;
   - the per-rule query prompts start where the rule says;
   - the option is refused with Fresh V2 and without a panel, and W and the objective are read from the saved configs;
   - the bands count rows, hits and losses by offset from each conversation's BOS, and queries by their note's distance;
   - without the option, `windows_of` and the scored files are unchanged.

   The float crate's tests pass, and the stub-nvcc check is clean.
3. **The panel.** `heldout-panel chain=4096` at a new seed, for (L), with:
   - `check-queries` 0 failed;
   - a census against the long TRAIN showing no shared prompt;
   - the user turns excluded as for p-long;
   - storage in the store with a `SHA256SUMS`, read back.
4. **The sheet,** `docs/harness/prep/integer-long-context-2026-11-05.md`, with a fresh non-author FIT and a pre-run review.
   - **Arms:**
     - a7r at seeds 1 and 2 on p-long (W = 192, C = 2,048);
     - (L) at seeds 1 and 2 on the new panel (W = 2,048, C = 4,096);
     - each rule with memory on and off (L-010).
   - **The decision table,** as claude-286's ORDER set it:
     - blocks is the forgetting control;
     - carry wins if it holds within slide's paired band past W;
     - a tie never chooses blocks, chooses carry, and is provisional;
     - if carry is worse, slide wins, unless a carry training arm is needed, which is then the next unit.
5. **The run.** One RTX 5090 on Vast within a $3 bound, a watcher on every job, results sealed in the store and read back before `down`.
6. **The decision review** on #3471 (`[claude] owner log:`), one row in `m4-serving.md`, and a REPORT with the negatives first. The last commit returns `session.rs` and `mod.rs` to the item's scope.

## Stop and ask if

Stop and decide; never stop and wait.
- **Carry past 4,096 would need the cap raised or chunked forwards.** Decide: keep C at 4,096 and say so; the panel is cut to 4,096.
- **The Hamming read refuses, or memory runs out, at 4,096.** Decide: chunk the batch to one conversation per forward. If it still fails, carry at 4,096 is an instrument limit, recorded as such, and (L) runs carry at 2,048 only.
- **`query_loss` needs arithmetic the scorer does not have** (an objective missing from the checkpoint's config). Decide: report hits and agreement only, and say done-when 2's loss is missing and why.
- **A rule's run would pass the $3 bound.** Decide: cut s for slide (a coarser stride) before cutting an arm, and record it.
- **DeepSeek has an open `cpu-training-pilot` order when this unit's last commit returns the two files.** The return does not lock while this order is open, because the item's scope at the merge base is what `loop claim` reads. Nothing waits.
