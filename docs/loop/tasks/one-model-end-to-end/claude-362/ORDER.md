# ORDER: one-model-end-to-end, claude-362

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `870d8b1d48c06a9c320e3a21d707c708c22b69b8`.

## Goal

**When the counted asks find the place of a told value, the reply starts on the value.** The fitter gets a switch that marks the reply's first row as an opening row. The result is read as replies on three saved models, on both existing panels and on a panel nothing has read, and in the nine-turn conversation.

**Why now.** With the counted table of [claude-359](../claude-359/REPORT.md) the place is found on 560 of 578 recall questions of the reading panel, but the reply states the value on 513, 505 and 491 (three seeds). Read word for word, most of the lost replies never reach an opening the table counted: the model echoes the name with a question mark ("Stasgrust?"), loops, or answers something else, so the found value is never said. The nine-turn conversation shows the same at turn 2 ("What is mibtorn?" → "Mibtorn?", with the place held).

**What was already seen, on the laptop, before this order** (kit `local/claude/claude-361/first-row/`, 13 Nov): the table with one pair changed by hand, the assistant marker's last two tokens set to open. Reply states the told value, three seeds:

| Panel | Own read | Counted table as merged | First row open |
|---|---|---|---|
| reading panel (seed 16), of 578 | 314, 320, 325 | 513, 505, 491 | 555, 556, 554 |
| scoring panel (seed 14), of 562 | 316, 300, 311 | 493, 471, 476 | 536, 538, 538 |

Seed 2, reading panel: 44 replies gained, 2 lost; of 403 other rows 19 changed, all of them never-told questions the table had already answered with a wrong value; exact replies on those rows 88 before and after. Conversation: turns 2, 5, 7 and 9 right; turn 4 ("Say exactly this: …") now answers "18.", a false find the table already made that is now spoken.

Both panels were read before the idea existed, and the idea came from reading the reading panel's wrong replies. So this order's bar is on a panel written after it.

## What is built

1. `place-asks`: a switch, off by default, that writes the opening pair for the reply's first row (the assistant marker's last two tokens) into the table's `opens`, replacing a counted entry for that pair. The fit log names it. With the switch off the table is byte for byte what the fitter writes today.
2. Tests: the table with and without the switch; the reader's first reply row offers and speaks the place's token when the asks set the register, and does nothing when they did not.
3. No change to the reader's program and none to an artifact. No training, no pod.

## The bar, written before the new panel exists

A fresh panel: the reading panel's writer at `seed=18`, written after this order is pushed. On it, for each of the three saved models (claude-358's p-s2, p-s1, p-s3), V is the recall questions whose reply holds the told value as a whole word, by the pinned scorer.

- **Adds:** V with the switch is higher than V without by at least 5% of the panel's recall questions on all three models, and the exact replies on the other rows are not down by more than 1% of those rows on any model.
- **Does not add at this configuration:** the gain is under 2% on two or more models.
- **Otherwise not called.**

Prediction: +6% to +9% on each model; other rows' exact count unchanged; never-told rows unchanged.

Reported with no bar: both old panels; never-told rows that say so; replies that never end; the nine-turn conversation word for word; the false finds that are now spoken.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The switch off writes today's table; on, it changes only that pair. | a test, and a byte compare on the fitted table |
| G2 | The reader offers and speaks the place's token at the reply's first row only when the asks set the register. | tests in the integer crate |
| G3 | The seed 18 panel is written after this order and read once; the row is called by the bar above. | the kit's logs with times |
| G4 | The REPORT gives the replies word for word and what did not work first. | the REPORT |
| G5 | One notebook row. | the notebook |

## Not established by this unit

- Whether a count-fitted table may be served in the product. That is the owner's reading of §3.1 and is open; this unit serves the table only through the reader's existing `--place-asks` switch.
- Anything trained. The trained key is [claude-361](../claude-361/ORDER.md), running now.
- Conversation quality beyond the nine turns read; the table's false finds; general replies.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a table or reader that already opens the reply's first row.
- **Result (at claim):** none. The merged table holds the pair as counted, not open, and the reader has no such rule.

## Claim

Written by hand: claude-361 (PR #3951, a draft with pods running) holds the item for `uor-harness loop claim`.

## Files in scope

- `crates/uor-r4-integer/src/place_asks.rs`, `crates/uor-r4-integer/src/bin/place-asks.rs`, the crate's tests
- `docs/loop/tasks/one-model-end-to-end/claude-362/**`
- `docs/research/notebook/**` (one row)

## Inputs

- The three saved models and both panels: `local/claude/claude-358/`, `local/claude/claude-357/`, `local/claude/claude-359/`.
- The pinned reader and scorer: `local/claude/claude-361/bin-pin/`.
