# ORDER: one-model-end-to-end, claude-338

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `f75725d30`.

## Goal

**Make the reply-panel scorer serve the ask heads and the question carry, so the language run (claude-339) can be scored on what it learned.**

- **Re-scoped at the cancel probe.** This ORDER first named the language run itself. The probe's instrument check found that the scorer cannot see the mechanism that run trains, so the run moves to claude-339.
- **The fault (instrument, L-007):**
  - `served-reply-panel` decodes through `CpuFbsForward::from_artifact` (`served-reply-panel.rs:307`), the CPU port in `crates/uor-r4-cpu-train` (DeepSeek's crate).
  - Its Hamming reads (`hamming.rs`, 750 lines) never read `ask` or `carry`. claude-336 named this and sent it to DeepSeek on #2028.
  - So an asking artifact would be scored as if it had no ask table and no carry, silently.
- **The integer session serves both exactly:**
  - [claude-336](../claude-336/REPORT.md) matched the float trace, rounded as exported, on every decision.
  - It decodes with the same shape the scorer uses (`mark_reply_start`, `step_native`, `fbs-toks.rs:872-874`).

**The change, in the lab instrument `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:**
1. **The scorer refuses an artifact whose Hamming image asks or carries** on the CPU-port path, by name. It never scores one silently.
2. **`--engine integer`** decodes through the integer session. The CPU port stays the default.
3. **Parity, the instrument check:** on claude-322's sealed seed-1 artifact (`86e51d5e…`, no ask table), `--engine integer` and the default give identical output JSON on all 256 rows of `m2-fresh256`, once the four timing fields (`wall_milliseconds`, `tokens_per_second`, `decode_milliseconds`, `decode_tokens_per_second`) and the integer run's `settings.engine` are removed.
4. **Each new path has a test, and each test fails with its fix set aside.**

**Claim check:**
- **(a) Step:** step 4, one model trained, served and scored end to end, for the recall gap.
- **(b) What it unblocks:** the scorer for claude-339's language run of the ask heads and the carry. Without it, that run's panel score would measure a model without them.

**Not in scope, named here:**
- the language run (claude-339, with its own sheet, FIT and pre-run review);
- DeepSeek's CPU port (its lane).

## Cancel probe (run first)

1. No open PR touches the item's files.
2. claude-337 is merged and verified.
3. Disk: at least 20 GiB free.

**Result (instrument, at claim):** pass.
- PRs #3889 (DeepSeek) and #3890 (Kimi) touch none of the files.
- claude-337 merged as `f75725d3071abb9488e058ac158a04d8d093014d` and was verified on fresh `main` (lib 340/0, bin 11/0).
- 29 GiB free.
- `loop next` offers only DeepSeek's CPU-repair direction, so this ORDER is hand-written under Casey's harness exception, as claude-331 to claude-337 were.

## Files in scope

- `docs/labs/one-model-end-to-end/**` (the instrument crate)
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`, `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. Both paths behave as above, each with its test and a failing mutation.
2. The parity check is byte-identical, or its first difference is classified.
3. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). If the integer session cannot decode a reply the way the scorer needs, the REPORT names the gap, and claude-339 decides how to score.
