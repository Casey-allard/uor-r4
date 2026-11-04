# ORDER: one-model-end-to-end, claude-273 (training at a set window)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `f42159e43`. Draft PR #3646.

## Goal

Training cuts every TRAIN document into windows of `SHEET_CONTEXT = 192` tokens (`hamilton_sheet.rs:66`), and builds its model at that window (`build_hamilton_model_at`, `config.context = SHEET_CONTEXT`). No flag sets either. claude-271's conversations run to 2,048 tokens, and claude-272 made the Hamming read fit at 2,048. So the next limit is the window itself.

This unit makes the training window a setting, `--context N`, with 192 as the default. A run at 192 writes the same `config.json` and trains the same model as before. It then times one training step at 2,048 against 192 on one GPU.

- **Plan step:** Casey's 4 November plan, P2 unit 3 ("the window as a setting"). It also gives P2b ("time a training step at 2,048 against 192") its number before that run's sheet is written.
- **What it changes in the model:** the window the model trains at, and so how far back each row can read.
- **No prereg** (§4.5): the GPU part is a finite throughput test, a few steps per window, with no result claimed beyond time, memory and a finite loss.

**Moved out of this unit (P2's bound is 2–3 units; this is unit 3):**
- Scoring and decoding at a set window. `--score-rows` and its decodes keep a 192-token window, `decode_*`'s sliding window among them. A model trained at N still scores 192-token windows, since `embed` refuses only `time > context`.
- The long TRAIN corpus and the long held-out panel.

Both go to P2b's sheet unit, which must name the data by sha256 anyway. The overrun goes in PLACE.md and in one note to Casey.

## Cancel probe (run first)

1. Does any flag or `LongConfig` field already set the training window?
2. Is `SHEET_CONTEXT` still the only window on the training path: `load_train`, `start_memory`, `assemble`, `trained_points`, `cut_census`, `gradient_norms`, and `run_long`'s memory rows and throughput line?
3. Does anything that `build_fbs_model_at` builds depend on `config.context`? `read.age` is `[heads, context]`, but only on `a` layers, and the sheet's pattern is `rrrrrr`.
4. Does any open order hold these files? The claim's overlap check passed.

**Result, at `f42159e43`: not cancelled.** The finding is about the instrument, not the mechanism: the training window is a constant.
1. No flag or field sets the window. `train-in-space.rs` has no `--context`, and `LongConfig` has no window field. `recall_window`'s `context` argument belongs to the recall mode, not training.
2. Yes: the training path reads `SHEET_CONTEXT` at `in_space_long.rs`:
   - `load_train` (:9660, :9665) and `start_memory` (:772);
   - `assemble` (:1998, :2003) and `trained_points` (:1064, :1066);
   - `cut_census` (:1047, :1048) and `gradient_norms` (:9746);
   - `run_long`'s memory rows (:10294, :10308), its whole-window check (:10315) and its throughput line (:10174).

   The model is built at `config.context = SHEET_CONTEXT` (`hamilton_sheet.rs:2205`).
3. No. The only weight sized by the window is `read.age` (`geometric_stack_native.rs:789`), and only `a` layers have it; the sheet's pattern is `rrrrrr` (`hamilton_sheet.rs:2201`). `embed` refuses only `time > config.context` (:1165).
   - **Out of this unit:** `fbs_export.rs:293` exports `config.context`. A model trained at 2,048 therefore exports a 2,048 context, while the integer session's point key is pinned at 192 (`fbs_snapshot.rs`). That is P4's to reconcile.
4. The claim passed with no overlap.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-273/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **The window is a setting.**
   - `train-in-space --context N` sets `LongConfig.context`. It accepts 2 to 4,096, the model's own range, and refuses anything else.
   - The field is left out of `config.json` at 192, so a run without the flag writes the same file as before.
   - `run_long` builds its model at N, through a new `StackModel::set_context`. That setter refuses a model with any weight sized by the window.
   - `load_train` and `start_memory` cut TRAIN at N.
   - Every row index on the training path uses the window's own length:
     - `assemble`;
     - `trained_points`;
     - `cut_census`;
     - `gradient_norms`;
     - the turn memory's rows and its whole-window check;
     - the throughput line.
   - DEV's evaluation during training keeps its 192-token windows.
2. **The launch-size guard.** `choose_sorted` returns an error, not a partial launch, when any tensor it builds would reach 2³² elements. claude-272 found that candle 0.9.2 casts launch sizes to `u32`. The tensors checked are the `(batch · time, time)` distance and its steps, and one position's `(batch, 1, time, width)` difference.
3. **Host tests on the laptop.** No tensor maths, per D24/D29.
   - The rows `assemble` lays out at a 2,048 window are `b · 2,048 + offset`, and at 192 they are the old `b · 192 + offset`.
   - A `LongConfig` at 192 serializes without `context`.
   - `--context` refuses 1 and 4,097.
   - The guard's arithmetic is right at 2³² − 1 and at 2³².
   - **Negative gates:** set aside the stride fix, and the `assemble` test fails; set aside the serde skip, and the `config.json` test fails.
4. **One GPU check:** the smallest RTX 5090, one lease, under an hour.
   - It runs a few `run_long` steps at `--context 192` and at `--context 2048` on pinned TRAIN, with the recipe P2b will train (batch 4).
   - It records the mean step time, the memory pool's peak and a finite loss for each window.
   - On the same pod, the crate's CUDA library suite passes.
5. **A REPORT, negatives first,** and one notebook row.

## Stop and decide

- **A step at 2,048 fails** (a launch, a kernel limit, out of memory): classify it as instrument or mechanism. Fix it in this unit if it is the window's code. Otherwise record it as P2b's first blocker.
- **A step at 2,048 is more than about 20× a step at 192** (the window is 10.7× longer): record it. The packed-bit read, or a smaller training batch at 2,048, is then P2b's first decision.
- **Any 192 behaviour changes:** this is a bug. Fix it; never loosen a test.
