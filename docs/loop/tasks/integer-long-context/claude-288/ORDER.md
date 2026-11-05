# ORDER: integer-long-context, claude-288

Lab: claude. Item: [integer-long-context](../../../../roadmap/items/integer-long-context.md) (#3471). Base: `03f220688`. Plan step: P4, unit 3 of 3 ([claude-286's ORDER](../claude-286/ORDER.md)).

**Claim check.**
- **(a) The step:** P4 in Casey's 4 Nov plan, long context served (#3471).
- **(b) What it changes in the model that serves:** today the integer session refuses every token past 192 (`StackError::ContextFull`), because every served a7r artifact has `context = 192`. After this unit, an a7r artifact serves a conversation past 192 under **carry**, the rule claude-287's decision review chose (#3471 6653674839), with the turn memory keyed as the float model keys it under carry.

## Goal

The item's done-when reads:

> 1. **The decision,** recorded by a decision review on this item's issue (`[claude] owner log:`): which rule, and why, read from the measurements in 2.
> 2. **The measurements,** under a prep sheet with a fresh FIT and a pre-run review: on one trained checkpoint, the float model's held-out loss and greedy reply agreement past 192 tokens under each rule, on multi-turn prompts whose context passes 192. A rule needing a new training arm gets that arm.
> 3. **The integer session serves the chosen rule,** with a test against the float forward's rule past 192, and a negative gate. A session within 192 tokens decodes byte for byte as at the base.
> 4. **Tokens per second against context** out to at least 2,048 tokens, with the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.** The pipeline map's gap 6 records the result.

Done-whens 1 and 2 were met by claude-287 (#3696, `f9807de88`). This unit delivers 3, 4 and 5. The 2 Nov owner log on #3471 sets the target: a served conversation of 4,096 tokens, with tok/s and resident memory out to 8,192.

## Cancel probe (run first)

Read at `03f220688` (`origin/main`) and `f9807de88` by two read-only scouts, and checked here.

1. **The session stops at 192.**
   - `ContextFull` is returned once `position >= s.context` on every step path (`stack/session.rs:2928, 3103, 3541, 3792`), and by the in-crate float reference (`stack/format.rs:2331, 2664`).
   - The field is `StackShape.context` (`format.rs:313`), capped at `MAX_CONTEXT = 1 << 16` (`:289, 575`).
   - claude-243's a7r artifacts were exported at the snapshot's 192 (`docs/loop/tasks/integer-nearest-readout/claude-243/REPORT.md:76`).
2. **The trunk and the Hamming read already carry.**
   - claude-242 changed only the snapshot's `context` (192 to 32,768) and re-ran `fbs-export`. The payload was byte-identical, and replies matched the 192 artifact (`docs/loop/tasks/integer-hamming-read/claude-242/REPORT.md:68-76`).
   - The Hamming read scans every earlier position (`session.rs:3642-3643`, `stack/hamming.rs:332-334`). a7r has no rank read, so `MAX_FLOCK_CONTEXT = 4,096` (`session.rs:752-758`) does not apply. a7r has no point key either (schema `/7`, claude-243's ORDER).
   - No export path sets a serving context; claude-242 edited JSON with `sed`. No field records the window the model was trained at.
3. **The turn memory does not follow the float model's carry rule.**
   - **Integer:** keys cut into `key_window`-token blocks counted from the conversation's start (`stack/memory.rs:274-275`, `block_start`). The anchor search stops at its block's start (`:281-294`). The reply key's turn is `[]` once the prompt leaves the row's block (`:303-305`). a7r's `key_window` is 192 (`fbs-snapshot.rs:627-628`), and editing `context` leaves it at 192.
   - **Float, under carry:** `set_context(C)` (`in_space_long.rs:7576-7577`). The decode reads `ids[len - context..]` (`:6190`), and the reply key comes from `opening_slots` over the whole window (`:6208`, `:4235-4241`; `flock_read_opening.rs:77`). Those keys start at 0 and move only at `<|bos|>` and `<|eos|>`.
   - So past 192, the integer and float keys differ wherever a turn's anchor or a reply crosses a 192 boundary. claude-286's ORDER (`:90`) gave this to claude-288.
4. **No test reads an integer session past 192 against the float rule.** The synthetic trunks run at contexts 6 to 64 (`stack/tests.rs:2579`, and others). The one session past 192, `turn_memory_keys_follow_the_item_rule` (`tests.rs:5567`), writes its 192-block keys out by hand.
5. **The float reference exists.**
   - claude-287 sealed it in `results/claude/integer-long-context-2026-11-05/c287/` (`SHA256SUMS.files` sha256 `bd18638e…f68e4`, 378 files).
   - The files are `aN-sK-carry/samples-p-long-queries-{memory,no-memory}.json`. Each holds 609 queries: the prompt from the conversation's start (up to 1,994 tokens), the float model's 32-token greedy reply, and the reference.
   - a7r ran at W = 192 with carry at C = 2,048, on the R arm `r-drop-sK` step 16,384 of `training-460fe46e`, the checkpoint claude-243 exported.
6. **The tools exist.**
   - `fbs-toks ARTIFACT --replies PROMPTS_JSON --out FILE [--reply-tokens N]` prefills each prompt in a fresh session and decodes greedily (`fbs-toks.rs:113, 756-773`).
   - `--context-sweep --tokens` times prefills ending at 192, 576, 2,112, 8,256 and 32,768, and refuses an artifact whose context cannot hold the last (`:118-120, 702-707`).
   - RSS is read with `/usr/bin/time -l`, as claude-242 did; `fbs-toks` measures none.

**Result:** not cancelled. The class is **serving**: the session cannot serve past 192, and its memory would key differently from the float model if it did. No reading of the model is made here except the laptop parity and speed below.

## Plan

1. **The export, `fbs-export --context C`** (slice 1).
   - It writes a carry artifact: `shape.context = C`, the memory's `key_window = C` (the float rule under carry: one block, the whole served window), and the trained window W (the snapshot's own context) recorded in the header.
   - It refuses C < W, C above `MAX_CONTEXT`, and a rank read past `MAX_FLOCK_CONTEXT`.
   - Without the flag the output is byte for byte today's: a7r's base export reproduces claude-243's sha `f86a7856…` at seed 1.
   - The recorded W is serialized only when set, so older artifacts round-trip unchanged. It changes no serving arithmetic, so an older engine that ignores it serves the same tokens; no schema bump.
2. **The tests** (slice 2, CPU).
   - **Keys:** a float-crate test over a token stream past 192, with several turns and replies crossing 192. The integer session's opening and reply keys under a carry artifact must equal the float carry rule (`opening_slots`, `greedy_reply_key`) at every row. Negative gate: with `key_window` left at the trained 192, the test fails.
   - **Within W:** on a synthetic model with memory, a carry artifact (C > W) decodes byte for byte as the base artifact for every position below W.
   - **Past W:** the carry session's trunk and read past W equal the in-crate float reference over the whole sequence.
   - **Header:** W round-trips, is absent on base exports, and W > context is refused.
3. **The laptop readings, a7r seeds 1 and 2** (integer serving, §3.6).
   - Fetch each seed's snapshot and run config from the store (claude-227's `results/claude/integer-retention-rates-parity-2026-10-31/`). Export the base (check its sha against claude-243's) and a carry artifact at C = 8,256, then read:
   - **P-1, parity past 192:** the integer greedy reply under carry against claude-287's float carry reply, on p-long's 609 queries with the memory on, compared up to and including the first `<|eos|>`. Report exact replies, the first differing token's offset, and the same count on the queries within 192 as the base rate. This is a reading, not a bar.
   - **P-2, within W:** on the DEV samples, the carry artifact's replies are byte-identical to the base artifact's.
   - **P-3, speed and memory:** `--context-sweep` tok/s at each context out to 8,256, decode and prefill, with peak RSS from `/usr/bin/time -l`, on the C = 8,256 artifact. `--context-sweep` today refuses an artifact that cannot hold 32,768, so slice 1 makes it time every point the artifact's context holds and print each point it skips. One more export at 32,768 reads the last point. Hardware stated.
4. **The records:** the REPORT with negatives first, one row in `docs/research/notebook/m4-serving.md`, gap 6 in `docs/model/pipeline-map.md`, and the item's status.

## Done when

- G1: `fbs-export --context C` writes a carry artifact as in plan step 1, its refusals are tested, and a base export is unchanged.
- G2: the key test passes, and fails with `key_window` at 192 (the negative gate, run and recorded).
- G3: the within-W and past-W tests pass.
- G4: P-1, P-2 and P-3 are read for both seeds and sealed with a `SHA256SUMS` in the store, under `results/claude/integer-long-context-2026-11-05/c288/`.
- G5: the integer and float crates' suites are green, and the stub-nvcc check of the float crate is clean.
- G6: REPORT, notebook row, pipeline map gap 6 and the item's status, with a fresh exact-head review.

## Stop and ask if

- The base export no longer reproduces claude-243's sha. Stop and find why before any reading: it would mean the export changed the served model.
- P-2 is not byte-identical. That is a carry bug, not a reading: classify it as instrument and fix it before P-1.
- Serving needs anything under `bin/uor-chat.rs` (held by `cpu-training-pilot`, DeepSeek's lane). Leave it, and record what the chat needs.

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
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
