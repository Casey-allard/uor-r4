# ORDER: one-model-end-to-end, claude-363

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `b5c5804ab09ff495b14aa2096b78796459b02979`.

## Goal

**A reply keeps the model's own wording, and is served again from the found value only when the value was never offered.** [claude-362](../claude-362/REPORT.md) opened every reply's first row: the told value was said on 543 of 582 recall questions on all three models (501, 489, 489 before), but no reply kept the reference's sentence (182, 178, 176 before, 0 after). This unit serves the reply with the counted table as merged; when the asks set a place and no row of the reply was offered it, the reader starts the reply over with its first row open.

## What is built

1. Reader: a session switch, off by default, that opens the first row of a reply when the asks set a place.
2. `StackChat` and `uor-chat --place-serve-again 0|1`: serve; if the register held a place at the reply's first step and no reply row was offered it, start the reply over with the switch on.
3. `raw-id-replies --serve-again 0|1`: the same rule per row; a row served again is marked and the receipt counts them.
4. Tests for each. With every switch off, replies are byte for byte today's. No training, no pod, no artifact change.

## The bar, written before the new panel exists

A fresh panel: the reading panel's writer at `seed=19`, written after this order is pushed, read once, on claude-358's p-s2, p-s1 and p-s3. V is the recall questions whose reply holds the told value as a whole word, by the pinned scorer; E is the recall replies equal to the reference. Arms: *merged* (the table as merged) and *again* (this unit).

- **Adds:** on all three models, V(again) − V(merged) is at least 5% of the recall questions, E(again) is at least 90% of E(merged), and the exact replies on the other rows are not down by more than 1% of those rows.
- **Does not add at this configuration:** the V gain is under 2% on two or more models, or E(again) is under 50% of E(merged) on two or more.
- **Otherwise not called.**

Prediction: V +7% to +10%; E(again) within 3 of E(merged); other rows' exact count unchanged.

Reported with no bar: claude-362's first-row-open arm on the same panel; how many replies were served again and how many of those already held the value; never-told rows; replies that never end; the nine-turn conversation word for word (turn 4's false find is expected to be spoken); the seed 18 panel, already read.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | Every switch off: replies byte for byte today's. | tests; the seed 18 replies re-read equal claude-362's files |
| G2 | The switch opens a reply's first row only, and only when the asks set a place. | tests in the integer crate |
| G3 | A reply never offered its place is served again; one that was offered is left alone; the history holds the reply served last. | tests |
| G4 | The seed 19 panel is written after this order and read once; the row is called by the bar. | the kit's logs with times |
| G5 | The REPORT gives replies word for word and what did not work first; one notebook row. | the REPORT, the notebook |

## Not established by this unit

- Whether a count-fitted table may be served in the product (§3.1, the owner's call).
- The table's false finds and misses; anything trained ([claude-361](../claude-361/ORDER.md) is running); general replies.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a reader that already starts a reply over, or a first-row rule that keeps the model's wording.
- **Result (at claim):** none. `StackChat` serves each reply once, and claude-362's switch opens every reply's first row.

## Claim

Written by hand: claude-361 (PR #3951, a draft with pods running) holds the item for `uor-harness loop claim`.

## Files in scope

- `crates/uor-r4-integer/src/fbs_snapshot.rs`, `crates/uor-r4-integer/src/stack/session.rs`, `crates/uor-r4-integer/src/stack/chat.rs`, `crates/uor-r4-integer/src/bin/uor-chat.rs`, the crate's tests
- `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-363/**`, `docs/research/notebook/**` (one row)

## Inputs

- The three saved models: `local/claude/claude-358/`. The table: claude-359's fit (`local/claude/claude-362/asks-off.json`).
- The scorer: `local/claude/claude-361/bin-pin/whole-word-recall`.
