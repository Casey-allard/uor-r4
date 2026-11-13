# ORDER: one-model-end-to-end, claude-361 (the trained key on the two tokens before the link word)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `6329952e329de1b59f83864519dc5d8dc6b673e9`.

## Goal

**Move the trained place key onto the two tokens before the link word and let the question read more than two tokens, so the model's own geometry finds a told value where today only the counted table does.**

**Why now.** Three counts say where the trained find loses.

- claude-350 sized the read that was then built (claude-351): a learned target for the asking token at the token before the place, and the token before the asking row matched with the token two before the place. By count it reaches 436 of 562 places at best. The 126 it cannot reach are questions asked in other words than the statement.
- claude-359 served a counted table in its place: the three saved models state a told value on 491 to 513 of 578 fresh questions, against 314 to 325 by their own read. The table is counts, not geometry (CHARTER §3.1).
- claude-360 cut the table's input: the two tokens before the link word (offsets 2 and 3) find 546 of 562 and 565 of 578; the cut nearest the trained read (offsets 1 and 2, the question's last two tokens) finds 242 and 251. The place read refuses the question carry, so its question is two tokens and no more.

**The steps, in one unit.**

1. **Size the trained read by count, on the laptop, before any pod.** A trained question gives each question token one code a head, where the table gives it a weight for every frame token. The count form of that: for each question token and offset, keep only its `k` heaviest fitted frame tokens (`place-asks ablate --targets k`), with and without the rows that do not read the question (`--shared 0`). Read on both panels for offsets 2 and 3 at `k` = 1, 2 and all, with the question's last 3 tokens, last 8 and all of it. This says how many targets a token and how much of the question the trained read needs.
2. **Build it.** `place_find` keys a place by the tokens two and three before it; the place read takes the question carry; the place pull moves the question toward the new key; what keeps the find to rows after a link token is fixed here, from step 1's counts. The artifact says which key it was trained with, and an artifact of today reads as it does today, byte for byte. Integer tests on the laptop, float tests on CUDA.
3. **Train and read the replies**, under a prep sheet with a non-author FIT: the saved models' recipe with the new key, three seeds, both panels, by kind, against the saved models.

Step 2 goes ahead when step 1's count with at most two targets a token is above the count ceiling of the read that was built (436 of 562, 0.776) on the first panel. Under it, the unit reports the count and changes the design before a pod is rented; it does not stop.

**Claim check.**
- (a) **Step.** The owner direction's "one model end to end" (`PLAN.md`): a told fact read back by the model's own parts.
- (b) **What it changes in the model.** Which tokens the ask heads' key names at a candidate place, how much of the question their learned code reads, and what the place pull trains it toward.

**Rules check** (CHARTER §3). Sign codes of token points under the key's rotation, a sum over earlier rows, a Hamming compare inside the head's margin: the project's geometry, already in the trainer (§3.1). No squashing function, no dense product (§3.2, §3.3). Step 1 is counts and integer adds on the laptop. Float tests and training run on a pod inside the caps (§3.5); the served artifact is integers (§3.6). No comparator.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a count or a build of a learned place key on offsets 2 and 3, or of a place question that reads more than two tokens.
- **Result (at base):** none. claude-350's rows C1 to C3w count a learned target at offset 1 and a same-token match at offset 2. claude-360 counts the table with every pairing kept. `--hamming-place` refuses `--hamming-carry-question` in code (`HammingReads::place_vocab`). The task stands.

## Claim

`uor-harness loop claim` refuses while this lab's worktree `.worktrees/claude-346` holds the item (its pod is training). The ORDER is written by hand, as claude-359's and claude-360's were; the units share no file.

## Files in scope

- `crates/uor-r4-integer/src/place_asks.rs`, `crates/uor-r4-integer/src/bin/place-asks.rs` (step 1)
- `crates/uor-r4-integer/src/fbs_snapshot.rs`, `crates/uor-r4-integer/src/stack/hamming.rs`, `crates/uor-r4-integer/src/stack/format.rs` (the key's offsets in the read and the artifact)
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`, `crates/uor-r4-float-train/src/fbs_export.rs` (the key, the carry and the pull in training and export)
- `docs/harness/prep/**` (the sheet), `docs/loop/tasks/one-model-end-to-end/claude-361/**`, `docs/research/notebook/float-training.md`

## Done when

1. Step 1's counts are in the REPORT for both panels, with tests that fail without the rule they pin, and the fit with no cut still writes table v4's bytes (SHA-256 `6e5e985cedb7d94305d8dd6d5abb7ef247a7f58787f6ea1997f19745a8de1417`).
2. The new key is in the trainer and the integer reader with tests: an artifact of today reads byte for byte as today; the new key reads the offsets it names and no other; the float and integer finds agree row by row.
3. The REPORT gives the retrained models' replies on both panels by kind, against the saved models, negatives first.

## Stop rules

- No bar is moved after a count or a reply is read.
- A pod goes up only after the float tests pass and the sheet has its FIT, inside the caps, with its watcher armed first.
