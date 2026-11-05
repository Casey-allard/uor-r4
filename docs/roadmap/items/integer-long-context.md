---
id: integer-long-context
title: Serve prompts and conversations past the 192-token window
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [integer-hamming-read]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/point_key.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/hamilton_sheet.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, crates/uor-r4-float-train/src/bin/fbs-snapshot.rs, crates/uor-r4-float-train/src/flock_read_hamming.rs, crates/uor-r4-float-train/src/flock_read_memory.rs, crates/uor-r4-integer/src/stack/hamming.rs, crates/uor-r4-integer/src/stack/memory.rs, crates/uor-r4-integer/src/stack/dialogue_memory.rs, docs/model/pipeline-map.md, docs/harness/prep/integer-long-context*, docs/roadmap/items/integer-long-context.md, docs/loop/tasks/integer-long-context/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

The pipeline map's gap 6: "The serving session cannot slide its 192-token window ... Long context, the geometry's main claim, cannot be served yet." A conversation of a few turns passes 192 tokens, so a served model can hold no real conversation until this is solved.

**What is true now** (read at `e06635e3a` by claude-286):
- **The integer session stops at its artifact's `context`.** Every served step returns `StackError::ContextFull { context }` once the session holds `context` tokens (`stack/session.rs`, and `stack/format.rs:2332, 2665`). No non-test code outside `stack/` names it; one oracle test asserts it (`crates/uor-r4-training/tests/stack_d11_oracle.rs:394`).
- **A larger `context` already serves the carry rule (c).** claude-242 re-exported a 192-trained artifact at a context of 32,768 by changing the snapshot's one `context` field; the payload bytes were identical. `fbs-toks --context-sweep` then decoded it past 192, with the trunk's state running on and the Hamming read scanning every earlier position:
  - 4,647 tok/s at 192, 1,851 at 2,112, 177 at 32,768;
  - the trunk alone flat at about 5,500 to 6,100 tok/s;
  - peak memory 145 MB at 32,768.

  No float forward under carry exists to test it against, and nothing has read what carry does to the model's predictions.
- **Every served artifact is a 192-token model,** claude-243's `a7r-s{1,2,3}.uorlut` in `uor-chat`, and no export at another window has been tested. The float `fbs-snapshot` refuses any other context when it reads out on DEV (`--dev`, `src/bin/fbs-snapshot.rs:222, 237`), and the point key's window is `POINT_KEY_WINDOW = 192` (`stack/point_key.rs:374`, read by `bin/fbs-export.rs:106`).
- **Training cuts back-to-back windows of `--context`**, 192 by default and 2 to 4,096 since claude-273, restarting at each document's `<|bos|>` and carrying nothing between windows; a document's last, partial window reaches back to fill its input (`build_document_windows`, `hamilton_sheet.rs:1870`). P2b's (L) trained at 2,048 on long chained conversations (claude-276).
- **The float greedy decode slides** over the checkpoint's own window (claude-274), in the serial decode (`in_space_long.rs:5849, 5860`) and the cached one (`:6124`; `CacheRefusal::WindowFull`, `geometric_stack_native.rs:1569`). It reads no reply table once the prompt has slid out (`turn-memory-integer`). Scoring refuses a `--context` past the checkpoint's window (`in_space_long.rs:7165-7169`), and `--score-rows --context N` from 193 to 1,536, where the row stride is untested (`bin/train-in-space.rs:21`).
- **Recall cannot decide the rule yet.** P2b's (L) states no far note even inside its window: 0 of 283 on trained phrasings and 0 of 306 on p-long, at both seeds (claude-281, Mechanism). Recall past the window is reported, and the choice rests on held-out loss, reply agreement and speed.
- So the trained model, its float decode and its integer session follow three different rules past the window.

The map's "the integer runtime has no look-back" is out of date: the Hamming read's copy and type heads are served (claude-194, claude-195). Their parity with the float read is [`integer-hamming-read`](integer-hamming-read.md)'s, which this item depends on.

## What is missing

**A rule for what comes after 192, chosen by measurement.** Each candidate changes cost or the model:
- **(a) Slide.** Re-run the last 192 tokens for each new token, as the float decode does. Exact against it, but it costs a 192-token prefill per token.
- **(b) Blocks, as training cuts them.** Reset at each 192 from `<|bos|>`. Exact against training, but the reply loses everything before its block.
- **(c) Carry the state.** The FBS trunk is a recurrence, so its state can run on. The Hamming read looks back over a bounded set of positions. The integer session already computes it at a larger `context` (claude-242), but the float model has never been trained or scored this way, so the rule may need a training arm that sees longer windows.

The trunk's recurrence makes (c) the geometry's own answer, which is why long context is "the geometry's main claim". The item measures before choosing.

## Done when

1. **The decision,** recorded by a decision review on this item's issue (`[claude] owner log:`): which rule, and why, read from the measurements in 2.
2. **The measurements,** under a prep sheet with a fresh FIT and a pre-run review: on one trained checkpoint, the float model's held-out loss and greedy reply agreement past 192 tokens under each rule, on multi-turn prompts whose context passes 192. A rule needing a new training arm gets that arm.
3. **The integer session serves the chosen rule,** with a test against the float forward's rule past 192, and a negative gate. A session within 192 tokens decodes byte for byte as at the base.
4. **Tokens per second against context** out to at least 2,048 tokens, with the resident memory.
5. **One notebook row, and a REPORT with the negatives first.** The pipeline map's gap 6 records the result.

## How to start

- **Read:**
  - `stack/session.rs` and `stack/format.rs` where `ContextFull` is returned;
  - `decode_reply_with_mode` (`in_space_long.rs:5789`) and the cached decode (`:6124`), for the slide;
  - `build_document_windows` (`hamilton_sheet.rs:1870`), for training's windows;
  - claude-242's REPORT (`docs/loop/tasks/integer-hamming-read/claude-242/`), for the integer carry's speed;
  - [`turn-memory-integer`](turn-memory-integer.md)'s notes on the window, and the pipeline map's stages 5 and 8.
- **Before claiming:** check that no open order holds an item sharing this scope.
  - `integer-hamming-read`, `turn-memory-integer`, `point-key-integer` and `integer-nearest-readout` edit the integer stack.
  - `one-model-end-to-end`, `float-kernel-fusion`, `training-step-cost`, `served-rounding-cost` and `writer-admission` edit the float crate's files here.
  - `docs-status-refresh`'s scope covers `docs/model/*.md`, the pipeline map included.
- **Kept out of scope:**
  - `bin/uor-chat.rs`: `cpu-training-pilot` holds it, and the session applies the rule itself.
  - `stack/session.rs` and `stack/mod.rs`, until claude-287's last commit returns them for claude-288. They are this item's only overlap with `cpu-training-pilot` (DeepSeek's lane), and `loop claim` refuses an overlapping claim while either item has an open order. So the float units hold no lock on that lane.
  - `milestone_world_v2.rs`: its `CONTEXT = 256` caps one episode. `chain=` in `heldout-panel` and `m-world-recall-corpus` writes conversations past any window (claude-271, claude-275).

## Where it stands (5 Nov, claude-286)

The plan, recorded in [claude-286's ORDER](../../loop/tasks/integer-long-context/claude-286/ORDER.md), is three units, P4 in Casey's plan of 4 Nov. The 2 Nov owner log on #3471 sets the target: a served conversation of 4,096 tokens, with tok/s and resident memory out to 8,192.
1. **claude-286:** the cancel probe, and this item's scope and text.
2. **claude-287, done-whens 1 and 2:**
   - `--score-rows --past-window slide|blocks|carry` in the float scorer;
   - a `chain=4096` held-out panel;
   - a prep sheet with FIT and a pre-run review, on `a7r` (window 192) over p-long and P2b's (L) (window 2,048) over the new panel, each at seeds 1 and 2, memory on and off;
   - one RTX 5090 pod, and the decision review.
3. **claude-288, done-whens 3 to 5:** the session serves the chosen rule, recorded in the artifact. Then the tests, tok/s and resident memory to 8,192, and gap 6.

## Rules that apply

- **GPU-only research compute** (CHARTER §3.5): the float measurements run on the GPU. Integer decodes on the laptop are product serving (§3.6).
- **Serving stays integer** (§3.6, as amended by D80 and D81).
- **Lead with tokens per second** against context.
- **A negative closes a configuration, never a mechanism** (§4.1).

## Sources

- `docs/model/pipeline-map.md`, gap 6 and stages 5 and 8.
- [`turn-memory-integer`](turn-memory-integer.md), its notes on training's blocks and the float decode's slide.
- `crates/uor-r4-integer/src/stack/session.rs`, `StackError::ContextFull`.
