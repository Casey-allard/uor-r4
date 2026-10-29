# ORDER: route-plus-memory, claude-161

Lab: claude. Item: [route-plus-memory](../../../../roadmap/items/route-plus-memory.md). Base: `e473455de`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "ok yes, my vote is to pivot to your diagnosed recommended new roadmap". This is that roadmap's decisive run, the [pipeline map](../../../../model/pipeline-map.md)'s R1. The item's done-when reads:

> 1. **A memory read,** float for training with an exact integer form:
>    - the key is the turn anchor's first `k` token ids, exact, or a learned code of the bag as a second arm;
>    - the table is built from TRAIN only: per key, its commonest next token, or a few with counts;
>    - at a reply row, the stored token's table point moves the readout point through a learned weight that starts at zero, as the Hamming readout does.
>
>    Host tests pin the key and the table build. CUDA tests pin that zero weight is no read and that the move equals the stored point's pull. With no new flag, every file a run writes is unchanged.
> 2. **A pre-registered run** (one sheet, a fresh FIT and pre-run review, CHARTER §4.5):
>    - (m) claude-152's (a) recipe plus the memory read;
>    - (m0) the same with an empty table, the no-memory control (L-010);
>    - (g) claude-155's bag, which holds the item `in-space-trunk-to-text`'s bar, as a second reference;
>    - two seeds each;
>    - openings read as the mean of the last three checkpoints;
>    - (m) barred against (a), (m0) and (g);
>    - the per-slot or per-key removal reading reported, as claude-154's I-5 did.
> 3. **The REPORT** leads with cost per token: training rate and the memory's bytes and operations per token. One notebook row, and the result on main.

## The read (this unit's design)

**The turn memory**, `crates/uor-r4-float-train/src/flock_read_memory.rs`:
- **Key.** At a row that predicts a reply's first token (an *opening row*), the key is the token ids at the turn anchor's first `k` slots (`opening_slots(ids, k, Turn)`, claude-154). claude-154 showed this equals the `--opening-bound` probe's first-`k` key at every DEV and TRAIN opening.
- **Table.** Built from the pinned TRAIN only: for each key, its commonest opening, ties to the smaller id (the probe's own rule). The table holds exact integer ids.
- **Move.** At an opening row whose key the table holds, the readout point n moves toward the stored token's table point c (detached): n + w ⊙ (c − n). Here w is a per-coordinate weight that starts at 0. An unseen key, and every other row, moves nothing. At w = 1, a hit decodes the stored token exactly.
- **Which rows.** The caller names the opening rows for each forward:
  - in training and in the DEV reading, from the response mask (`opens_reply`), as the evaluation already counts openings;
  - in serving, the first step of a reply.
- **Not in this unit:**
  - **The learned code of the bag,** the item's second arm. It runs only if the exact key fails.
  - **The move's integer export:** the integer runtime has no read path yet (the map's R5). The key and table are already integer-exact.

## Cancel probe (run first)

Run at `e473455de`:
- `route-plus-memory` is `ready`, and its scope covers every file the read touches (`flock_read*.rs`, `lib.rs`, `in_space_long.rs`, `train-in-space.rs`, `geometric_stack_native.rs`).
- No open PR touches those files.
- No other lab's ORDER names the item.

All three held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_memory.rs` (new; under `flock_read*.rs`)
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `docs/harness/prep/route-plus-memory*`
- `docs/loop/tasks/route-plus-memory/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/route-plus-memory.md`

## Done when

1. The read is built with the item's tests, and with no new flag every file a run writes is unchanged.
2. The sheet has a fresh FIT and a pre-run review. (m) and (m0) are run at seeds 1 and 2 and read against the bar. (a) and (g) are claude-152's and claude-155's sealed values, so they are not re-run.
3. The removal reading is reported, the outputs are sealed and read back, and the REPORT leads with cost per token.
4. The PR is merged and verified, with the owner log on #2028.

## Stop if

- **An instrument check fails:** for example, step 0 differs from (a)'s, or zero weight is not no read. Fix the instrument before any reading.
- **The caps would be passed.**

No step waits on Casey.
