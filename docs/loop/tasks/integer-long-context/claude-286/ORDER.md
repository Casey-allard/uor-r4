# ORDER: integer-long-context, claude-286

Lab: claude. Item: [integer-long-context](../../../../roadmap/items/integer-long-context.md) (#3471). Base: `e06635e3a`.

## Goal

The item's done-when reads:

> 1. **The decision,** recorded by a decision review on this item's issue (`[claude] owner log:`): which rule, and why, read from the measurements in 2.
> 2. **The measurements,** under a prep sheet with a fresh FIT and a pre-run review: on one trained checkpoint, the float model's held-out loss and greedy reply agreement past 192 tokens under each rule, on multi-turn prompts whose context passes 192. A rule needing a new training arm gets that arm.
> 3. **The integer session serves the chosen rule,** with a test against the float forward's rule past 192, and a negative gate. A session within 192 tokens decodes byte for byte as at the base.
> 4. **Tokens per second against context** out to at least 2,048 tokens, with the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.** The pipeline map's gap 6 records the result.

**This unit lands the cancel probe, the scope the later units edit, and their plan.** The loop's scope rule reads an item's scope at the merge base (`loopprcheck.rs:431-446`, as claude-284 found), so the files the measurement and the serving units change must be in the item's `scope:` on main before those units start. The item's text also predates P2 and P2b: it still describes a 192-token world.

## Claim check

- **The step it advances:** P4, "Long context served (#3471)", in Casey's plan of 4 Nov ("please make all of those points your current plan and goal for next steps"), and step 5 of THE ORDER (#2028 6634218223). The bound is 2–3 units. This is unit 1; the plan below is 3 units in all.
- **What it changes in the model:** this unit, nothing; it unblocks claude-287 and claude-288.
  - claude-287 changes what the float scorer does past a checkpoint's window, under one option. The default stays today's slide, byte for byte.
  - claude-288 changes what the served integer session does once its window is full. Today it returns `StackError::ContextFull`; after claude-288 it applies the chosen rule, so a conversation goes on past the window.

## Cancel probe (run first)

**Question:** is the work still open, and what can its measurement decide?

**What was checked, at `e06635e3a`:**
1. **The integer session still stops at its artifact's `context`.**
   - `StackError::ContextFull` is returned at `stack/session.rs:2444, 2929, 3104, 3542, 3793` and at `stack/format.rs:2332, 2665`.
   - No non-test code outside `stack/` names it. One oracle test asserts it (`crates/uor-r4-training/tests/stack_d11_oracle.rs:394`, a context of 8). `uor-chat` has no rule of its own for a full window.
   - **A larger `context` already serves the carry rule.** claude-242 re-exported claude-240's s1 artifact at a context of 32,768: the snapshot's one `"context":192` changed, and the payload bytes stayed identical.
     - `fbs-toks --context-sweep` then decoded it past 192. The trunk's state runs on, and the Hamming read scans every earlier position.
     - The trunk alone stays flat at about 5,500 to 6,100 tok/s from 192 to 32,768. The read adds about 167 ns per earlier position: 4,647 tok/s with the read at 192, 1,851 at 2,112, 177 at 32,768.
     - Peak resident memory at 32,768 was 145,473,536 bytes.
   - So integer carry exists with its speed measured. What no one has is the float forward under carry to test it against, or any reading of what carry does to the model's predictions.
2. **The window is no longer 192 everywhere.**
   - `train-in-space --context N` trains at an N-token window, from 2 to 4,096 (claude-273). P2b's (L) trained at 2,048 on long chained conversations (claude-276).
   - Training cuts back-to-back windows, restarting at each document's `<|bos|>` and carrying nothing between them; a document's last, partial window reaches back to fill its input (`build_document_windows`, `hamilton_sheet.rs:1870`).
   - The float decode slides over the checkpoint's own window (claude-274): in the serial decode (`in_space_long.rs:5849, 5860`) and in the cached decode (`:6124`, `CacheRefusal::WindowFull` at `geometric_stack_native.rs:1569`). Every claude-281 pass's first cache refusal is that slide.
   - Scoring refuses a `--context` past the checkpoint's window (`in_space_long.rs:7165-7169`), and `--score-rows --context N` from 193 to 1,536, where the row stride is untested (`bin/train-in-space.rs:21`). So no float reading past a window exists yet.
3. **Every served artifact is a 192-token model, and no export at another window has been tested.**
   - The float `fbs-snapshot`, when it reads out on DEV (`--dev`), refuses any context but `SHEET_CONTEXT` (`src/bin/fbs-snapshot.rs:222, 237`). Without `--dev` it reads out on fixed sequences and does not check the context (`:218-220`, `:233-235`).
   - The point key's window is pinned at `POINT_KEY_WINDOW = 192` (`stack/point_key.rs:374`), which `fbs-export` uses (`bin/fbs-export.rs:106`).
   - The integer format itself holds any context up to `MAX_CONTEXT = 65,536` (`stack/format.rs:290`). That is what let claude-242 raise an artifact's context by hand (probe 1).
   - So the served artifacts are 192-token models: claude-243's `a7r-s{1,2,3}.uorlut`, which `uor-chat` serves (claude-259).
   - Exporting P2b's (L), trained at 2,048, needs more than its context: the point key would still be built from 192-token windows, and the DEV readout refuses it.
4. **The data past a window needs no generator change.**
   - `heldout-panel chain=N` (`docs/labs/one-model-end-to-end/instrument/src/main.rs:296-300`) and `m-world-recall-corpus chain=N` (`src/bin/m-world-recall-corpus.rs:161-164`) take any budget of at least twice `CONTEXT`; the corpus writer also needs `context=` at least `chain=` and `recall=off` (`:165-172`). `MWorld2::chained` draws turns up to it.
   - The 2 Nov owner log on #3471 (6585860957) named `milestone_world_v2.rs`'s `CONTEXT = 256` as the first blocker. That cap bounds one episode; the chained conversations P2 added (claude-271 to claude-275) pass it.
   - So `milestone_world_v2.rs` and `examples/m-world.rs` stay out of this item's scope.
   - p-long (claude-275, in the store) already holds 160 conversations of up to 2,048 tokens: past a 192 window. A `chain=4096` panel would pass (L)'s 2,048.
5. **The measurement cannot decide by recall yet.**
   - P2b's (L) states no far note even inside its window: 0 of q-tt's 283 and 0 of p-long's 306, at both seeds. claude-281 classified this as Mechanism, closing that read configuration.
   - So recall past the window would read 0 under every rule, on (L) and very likely on `a7r`.
   - The rule has to be chosen on held-out loss, greedy reply agreement and speed, with recall reported.
   - The decision is to be re-read on the first checkpoint that recalls far notes inside its window.
6. **Overlap.**
   - The only open PR is #3695 (this unit).
   - No other item sharing the files below has an open order: float-kernel-fusion, one-model-end-to-end, point-key-integer, turn-memory-integer, served-rounding-cost, training-step-cost and writer-admission.
   - **DeepSeek's lane overlaps this item already at the base.** `cpu-training-pilot` lists `stack/session.rs` and `stack/mod.rs`, both in this item's base scope, as well as `bin/uor-chat.rs`.
     - `loop claim` compares a new item's scope with each open order's item scope as it stands on origin/main (`loopclaim.rs:18, 680-710`).
     - So this unit's own order, open since 03:19Z, refuses a `cpu-training-pilot` claim. DeepSeek claims one about every hour (deepseek-147 to deepseek-152 merged from 00:50Z to 03:11Z).
     - claude-287's long GPU unit would refuse them too, though it never edits either file. The decision below fixes that.

**Result:** not cancelled. **Class: instrument.** The probe settles which files the later units edit and what their measurement can decide; it does not touch any mechanism.

## Decision: fourteen scope additions and two removals (claude's own decision review)

**The removals.** `crates/uor-r4-integer/src/stack/session.rs` and `crates/uor-r4-integer/src/stack/mod.rs` leave the scope at this unit's merge (probe 6).
- **Why:** they are this item's only overlap with `cpu-training-pilot`, and only claude-288 edits them. Without them, an open claude-287 order holds no lock on DeepSeek's lane.
- **When they return:** claude-287's last commit puts both back, so they are on main for claude-288 only. claude-288 is short laptop work, claimed only after a check that DeepSeek has no open order.

**The additions:**

| File | What edits it | Why |
|---|---|---|
| `crates/uor-r4-float-train/src/in_space_long.rs` | claude-287 | The scorer's three rules past the window: probe 2's refusal and the decodes. |
| `crates/uor-r4-float-train/src/bin/train-in-space.rs` | claude-287 | The option and its usage line. |
| `crates/uor-r4-float-train/src/hamilton_sheet.rs` | claude-287 | The blocks rule cuts as `build_document_windows` does; a carry training arm, if the decision needs one, changes the cut. |
| `crates/uor-r4-float-train/src/geometric_stack_native.rs` | claude-287 | The cached decode's `WindowFull`: blocks resets and carry does not slide. |
| `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs` | claude-288 | Probe 3's refusal, for an export at the trained window. |
| `crates/uor-r4-integer/src/stack/format.rs` | claude-288 | Its two `ContextFull` returns, and the rule recorded in the artifact, so `uor-chat` needs no change. |
| `crates/uor-r4-integer/src/stack/point_key.rs` | claude-288 | `POINT_KEY_WINDOW` follows the model's window. |
| `crates/uor-r4-integer/src/bin/fbs-export.rs` | claude-288 | It reads that window. |
| `crates/uor-r4-integer/src/fbs_snapshot.rs` | claude-288 | The snapshot reader's 192-token key window. |
| `crates/uor-r4-float-train/src/flock_read_hamming.rs` | claude-287 | The read's decode and row cuts name the window's positions (claude-279). Carry reads past W; a read bounded to the last W keys, if the sheet keeps one, cuts here. |
| `crates/uor-r4-float-train/src/flock_read_memory.rs` | claude-287 | The turn memory's reply table is read only while the prompt is inside the window. Each rule says what it answers past W. |
| `crates/uor-r4-integer/src/stack/hamming.rs` | claude-288 | The served read scans every earlier position (claude-242); a bounded read, if chosen, bounds it here. |
| `crates/uor-r4-integer/src/stack/memory.rs`, `stack/dialogue_memory.rs` | claude-288 | The served reply table and its key window, which must follow the float table's rule past W. |

- **None of the last five adds a lock.** Every item that lists them is done or held, or already overlaps this item through `session.rs` or `in_space_long.rs` (`one-model-end-to-end`, `turn-memory-integer`, `point-key-integer`, `integer-nearest-readout`).

- **Left out:**
  - `bin/uor-chat.rs`: `cpu-training-pilot` holds it, and the session applies the rule itself.
  - `milestone_world_v2.rs` and `examples/m-world.rs`: probe 4.
  - `docs/research/notebook/float-training.md`: the item's own notebook, `m4-serving.md`, takes every row.
- **Reversal cost:** low.
  - It is one scope line.
  - The lock it adds holds only while this item has an open order. While it does, a claim on an item that shares these files is refused (probe 6's list). All of those are ready and have no open order; the one next in Casey's plan is writer-admission (P6), which comes after P4 and P5.
  - If the return is forgotten, claude-288's own scope check fails on its first edit to `session.rs`, which shows at once.

## Files in scope (this unit)

- `docs/loop/tasks/integer-long-context/claude-286/`: this ORDER and the REPORT.
- `docs/roadmap/items/integer-long-context.md`: the scope line, *What is true now* and *How to start* brought up to the base, and a *Where it stands* section with the plan.

## Done when (this unit)

1. The cancel probe and the scope decision above, recorded, with the decision review's record posted on #3471 as a `[claude] owner log:`.
2. The item's scope adds the fourteen files and drops `stack/session.rs` and `stack/mod.rs` until claude-287's last commit. Its text describes the base: training's `--context`, the slide in both decodes, the 192-only export, and claude-281's recall reading. Its title and done-whens stay as they are: they still hold for the served 192-token model.
3. The REPORT, negatives first. No notebook row: nothing is measured.

## Stop and ask if (this unit; stop and decide, never wait)

- **The scope edit is refused at the head** (`loop pr-check`). Then the scope goes through whichever unit the harness admits, and this unit closes on the probe alone.
- **The decision review finds a reason to keep a file out, or one missing.** Fix the line, and re-review the delta.

## The plan for claude-287 (done-whens 1 and 2: the rule, measured in float)

### Files in scope

`in_space_long.rs`, `bin/train-in-space.rs`, `hamilton_sheet.rs`, `geometric_stack_native.rs`, `flock_read_hamming.rs` and `flock_read_memory.rs` in the float crate, `docs/harness/prep/integer-long-context*`, the item file, its task folder, and `docs/research/notebook/m4-serving.md`. Its last commit returns `stack/session.rs` and `stack/mod.rs` to the item's scope, for claude-288.

### Done when

1. **The rules in the scorer.** `--score-rows --past-window slide|blocks|carry` scores each panel conversation whole, at its own length, under one rule for both the loss and the greedy decode. Without the option, every output is byte for byte as at the base. W is the checkpoint's own window.
   - **slide:** each target and each decoded token sees the last W tokens. This is today's decode, serial and cached.
   - **blocks:** the window restarts at each multiple of W from the conversation's `<|bos|>`, as `build_document_windows` cuts training, including its last, partial window, which reaches back to fill W inputs.
   - **carry:** one pass over the whole conversation. The trunk's state runs on past W, and the Hamming read reads every earlier position. The sheet decides whether a read bounded to the last W keys is a fourth arm.

   Within the first W tokens the three rules are byte-identical, which a test checks. Every rule keeps the memory-on and memory-off pair, the no-memory control (L-010).
2. **The panels.**
   - **p-long** (claude-275, in the store) for the 192-token checkpoints.
   - **A `chain=4096` panel** for (L): `heldout-panel` at a new seed; `check-queries` 0 failed; a census against the long TRAIN showing no shared prompt; the panel's user turns excluded as before. It goes in the store with a `SHA256SUMS`, read back.
3. **The sheet,** `docs/harness/prep/integer-long-context-<date>.md`, with a fresh FIT and a pre-run review.
   - **Checkpoints:** `a7r` seeds 1 and 2 (W = 192, the served model) on p-long, and P2b's (L) seeds 1 and 2 (W = 2,048) on the new panel. Each is pinned by the sha256s of its sealed run.
   - **Readings,** for each rule and memory setting:
     - held-out loss by distance past W, with the band within W as the identity check;
     - greedy reply agreement with the reference;
     - recall by distance, reported and not deciding (probe 5);
     - seconds per pass and decode tok/s.
   - **The loss** is the held-out loss the scorer already reports: the checkpoint's own objective, with no softmax or cross-entropy form (§3.2).
   - **The decision table** names what each outcome chooses.
     - **Blocks is the forgetting control.** It drops everything before its block, so a rule that does no better than blocks past W keeps nothing useful from the earlier context.
     - If carry's loss and agreement past W hold within the sheet's paired band of slide's, carry wins. It costs one trunk step and one read per token, and its integer speed is measured (probe 1). Slide costs a W-token prefill per token.
     - **A tie does not choose blocks.** If all three rules tie on loss and agreement past W, which is likely on templated filler with recall at 0, the panel has not tested what the earlier context is for. Blocks is the cheapest to serve, its read and memory bounded at W, but it drops every earlier turn by construction, which a 4,096-token conversation cannot accept. So a tie chooses carry over slide, on serving cost: carry's integer cost is measured (probe 1, 1,851 tok/s at 2,112), and slide's is estimated from the same figures as a W-token prefill per token, under 30 tok/s at W = 192. The integer session runs neither slide nor blocks today, so neither is measured. A choice made on a tie is provisional, and is re-read on the first checkpoint that recalls far notes inside its window.
     - Blocks wins only if both other rules are worse than it past W.
     - If carry is worse, slide wins, unless a carry training arm is needed. That arm is the 2 Nov owner log's: consecutive chunks with the trunk's state and the read's earlier keys carried, gradients inside a chunk. If the decision needs it, it is the next unit, and P4's bound is checked then.
4. **One RTX 5090 pod,** a $3 bound, a watcher on every job, and the results sealed in the store before `down`.
5. **The decision review** on #3471 (`[claude] owner log:`), one notebook row in `m4-serving.md`, and a REPORT with the negatives first.

## The plan for claude-288 (done-whens 3 to 5: the rule, served)

### Files in scope

`stack/session.rs`, `stack/format.rs`, `stack/mod.rs`, `stack/tests.rs`, `stack/synthetic.rs`, `stack/point_key.rs`, `stack/hamming.rs`, `stack/memory.rs`, `stack/dialogue_memory.rs`, `fbs_snapshot.rs`, `bin/fbs-export.rs` and `bin/fbs-toks.rs` in the integer crate; the float crate's `src/bin/fbs-snapshot.rs`; `docs/model/pipeline-map.md`, the item file, its task folder and `m4-serving.md`.

### Done when

1. **The session serves the chosen rule** in place of `ContextFull`. The artifact records the rule, so `uor-chat` needs no change.
   - **An artifact that records no rule keeps today's `ContextFull`.** That keeps every existing artifact, the oracle test at `stack_d11_oracle.rs:394` (outside this scope) and "within W, byte for byte" unchanged.
   - **The step cache's format.** THE ORDER (#2028 6634218223) says the float cache's "format is defined once so P4's integer session reuses it". If the rule needs a carried state, the session uses claude-278/279's cache format, not a second one.
   - **If carry is chosen,** the session already computes it at a larger `context` (probe 1). The unit then makes that the recorded rule rather than a hand edit, and tests it against claude-287's float carry.
   - **If the decision picks a checkpoint trained past 192,** the export follows its window: the float `fbs-snapshot` refusal lifts, and `POINT_KEY_WINDOW` comes from the model.
2. **Tests:**
   - against the float forward's rule past W, on a synthetic stack;
   - with a negative gate: the rule set aside, the test fails;
   - a session within W decodes byte for byte as at the base.
3. **Tokens per second and resident memory against context** out to 8,192 (the 2 Nov owner target; the done-when asks for at least 2,048). `fbs-toks` runs on the laptop, which is product serving (§3.6).
4. **The pipeline map's gap 6,** a notebook row, a REPORT, and the item set `done`.

### Stop and ask if (stop and decide; never wait)

- **DeepSeek has an open order** when claude-288 would claim. Then take other work and claim after it closes.
- **Carry changes what a saved session stores**, for example keys kept past `context`. `session_codec.rs` is in `cpu-training-pilot`'s scope, DeepSeek's lane, and is never edited here. Then the rule is not saved in a session, and a restored session starts its window afresh; or a note goes to DeepSeek on #2028, and the work waits for its own unit.
