# ORDER: one-model-end-to-end, claude-360 (what a key must see: the find by frame slot)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `8805b704344bab9cb40bba9c9f9b9f62774f952d`.

## Goal

**Size the context a trained key read needs, by census, so the next unit builds that read from the project's geometry and not from counts.**

**Why now.** claude-359 (PR 3949) showed the gap in replies: with an exact find the three saved models state a told value on 491 to 513 of 578 fresh questions; by their own trained find, 314 to 325. The exact find is a counted table and is not geometry (CHARTER §3.1), so the model does not own it. The trained find (claude-351) asks on one code of the row that asks and one code of the token before each place (`place_codes` in the float trainer). The counted table reads every token of the question and eight frame slots around each place. How much of that the find needs is not known, and it decides what the trained key is built over.

**The measurement.** The counted find is fitted again on TRAIN with part of its input cut away, and its census is read on both panels: frame slot 1 alone (the token before the place); slots 1 to 2; 1 to 3; 1 to 5; the turn's last three tokens alone; slot 1 with those; all eight; and, for the question, its last one, two and three tokens against all of it. No model is read and nothing is trained.

**The mechanism.** A cut names which frame slots and how much of the question a fit and its reader use. It lives in the program only: a table file and a session never carry one, so every table and artifact is byte-identical to today. `place-asks ablate` fits with a cut and prints the census of held splits in one run, and writes no table.

**Claim check.**
- (a) **Step.** The owner direction's "one model end to end" (`PLAN.md`): a told fact read back by the model's own parts.
- (b) **What it changes in the model.** Nothing here. It fixes the input of the next unit's trained key.

**Rules check** (CHARTER §3). Counts and integer adds on the laptop; no float, no GPU, no model, no comparator (§3.2, §3.3, §3.5, §3.6).

## Cancel probe (run first)

- **What would make this unit unnecessary:** a census that already cuts the frame slot by slot.
- **Result (at base):** none. claude-359 has two points only: five slots before the place with one floor (513 of 562) and all eight with two floors (539 of 562). The task stands.

## Claim

`uor-harness loop claim` refuses while this lab's worktree `.worktrees/claude-346` holds the item (its pods are training). The ORDER is written by hand, as claude-359's was; the units share no file.

## Files in scope

- `crates/uor-r4-integer/src/place_asks.rs` (the cut, in the fit and the reader)
- `crates/uor-r4-integer/src/bin/place-asks.rs` (the `ablate` mode)
- `docs/loop/tasks/one-model-end-to-end/claude-360/**`, `docs/research/notebook/float-training.md`

## Done when

1. The cut is in the fit and the reader, with tests that fail without it, and the uncut fit on TRAIN still writes table v4's bytes (SHA-256 `6e5e985cedb7d94305d8dd6d5abb7ef247a7f58787f6ea1997f19745a8de1417`).
2. The REPORT gives, for each cut, the true, wrong, "not told" and nothing counts of the recall questions and the places set on never-told questions, on the first panel and the second.
3. The REPORT says, from those counts, what the trained key is to read, and names the mechanism from the project's geometry the next unit builds it with.
4. A fresh review of the code, and the delivery path through merge.

## Stop and ask if

- A §3 rule would have to bend: tell Casey with the evidence and a proposed change.
- The cut would have to be written into a table file or read by a session.
