# ORDER: one-model-end-to-end, claude-316

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `084a7f92e`.

## Goal

**Make `served-reply-panel`'s speed figure measure decoding.** Today each row calls `CpuFbsForward::from_artifact` inside the timed loop, so `tokens_per_second` includes one full artifact parse per row (claude-311; still open in claude-313's and claude-314's REPORTs).

The fix:
- build the forward once, and give each row a clone of that untouched forward. `CpuFbsForward` derives `Clone`, so each row starts from the same state as before;
- time `generate` alone, and report `decode_milliseconds` and `decode_tokens_per_second` beside the existing wall figures.

Every row the scorer writes must stay byte-identical.

Claim check: (a) the step is "pipeline first" (#2028, 6709947526), with the owner's metric rule that tokens/s leads every result. (b) What it changes is the served model's measured decode speed: no model, artifact or score changes.

## Harness exception, recorded

`loop claim` refuses the item: deepseek-25's #3838 holds its claim. This ORDER is written by hand, under Casey's "i approve you to make harness exceptions as needed". #3838 does not touch the instrument crate. No open PR edits `served-reply-panel.rs` or `docs/labs/one-model-end-to-end/instrument/`; each open PR's file list was checked on 8 Nov.

## Cancel probe (run first)

1. No open PR edits the scorer's file or crate.
2. `CpuFbsForward` derives `Clone` at the base (`crates/uor-r4-cpu-train/src/fbs.rs:231-232`).

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-316/**`

## Done when

1. Scoring claude-313's seed-1 artifact (`1c9ae43c…`) gives rows equal to claude-313's `score/s1.json` (`5175c29b…`), field by field, both before and after the change.
2. The report carries `decode_milliseconds` and `decode_tokens_per_second`, and the decode time is at most the wall time.
3. The crate's tests pass, the binary builds with `--locked`, and the REPORT records the old and new speed figures on the same artifact.

## Stop and ask if

Nothing waits on Casey (§5.1). If the rows differ after the change, the change is wrong: revert it and record why. Bound: one unit.
