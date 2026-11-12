# ORDER: one-model-end-to-end, claude-353

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `1966da68b3463c4a21840ca15a2f177782dc0239`.

## Goal

**Make the integer reader's ask heads bring back what the trained model's ask heads bring back, and say what that does to the laptop copies of the four stage 3 models.**

**Why now.** [claude-352](../claude-352/REPORT.md) read NOT_RUN: on every model the laptop copy's replies differed from its trained model's on more than half the rows (I-f), against a bar of 0.10. On 12 Nov 2026 (UTC), in chat, the lab asked "Start the export repair with a laptop-only code read, no pod?" and the owner answered **"Yes, start"**. After the read found a cause the lab asked "Deliver the reader fix as a PR now, laptop only?" and the owner answered **"Yes, deliver"**. That approves this laptop-only unit. It approves no pod, no float run and no new arm.

**The fault.** In the float read (`crates/uor-r4-float-train/src/flock_read_hamming.rs`, claude-330) an ask head brings back the table's point of the kept position's own token, as a copy head does, and at the readout its mean mixes in the copy form, `w_y ⊙ (y − n)` where an ask head found a candidate. The integer reader (`crates/uor-r4-integer/src/stack/hamming.rs`, claude-336) asks as the float read does, but brings back the hook's input through the value rotation and adds `w_y ⊙ y`, as a plain type head. claude-336 showed parity on the heads' choices; nothing compared what they bring back.

## What runs

Laptop only. No pod, no training, no float forward.

| Step | What |
|---|---|
| 1 | The reader's change: an asking read's heads bring back table points, its mean is at the copy mean's exponent, and the readout mixes it in the copy form. A read that does not ask computes as before. |
| 2 | A test of the stream form and the readout form against the sums written out, which fails without the change. |
| 3 | The four stage 3 artifacts of claude-352's kit re-read with the changed reader on the same 192 and 256 prompts: the replies against the pod's saved float replies at 192 (I-f's count), and the stated values counted with claude-352's pinned scorer. |
| 4 | Two replies of each of two models word for word: trained model, laptop copy before, laptop copy after. |

## Decisions of this unit (claude's own decision review; logged for Casey, who can reverse them)

1. **claude-352's registered reading stays NOT_RUN.** Its sheet registers one run and that run is spent. The counts here describe the same sealed models under a repaired instrument, and no row of the sheet is read from them.
2. **claude-352's REPORT is not edited.** The correction to its laptop numbers is written in this unit's REPORT and in the notebook row.
3. **No float-side change.** The float read is the trained definition; the reader follows it.
4. **The difference that remains is reported, not chased here.** If rows still differ past 0.10 after the change, the unit says so with the counts and what is ruled out. A second cause is its own unit.
5. **The change was tried before this ORDER.** To test the cause, the reader's change was made uncommitted in this worktree during the approved code read, and two models were re-read with it. It is committed after this ORDER, and every count in the REPORT is retaken at the committed head.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The new test passes at the head and fails with the reader's change set aside. | the test run both ways |
| G2 | The integer crate's stack tests pass at the head. | `cargo test -p uor-r4-integer --features serving-wasm --lib` |
| G3 | The counts of step 3 are in the REPORT for all four models, before and after. | the REPORT |
| G4 | The correction to claude-352's laptop numbers is stated plainly. | the REPORT |

## Not established by this unit

- Whether the difference that remains is rounding or a second fault.
- Anything about a model not among the four, open text, or conversations longer than the context.
- No milestone moves.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a reader on `main` whose asking heads already bring back table points.
- **Result (at claim):** none. At the base, `ServedHamming::type_mean` writes `TypeHeads::payload` for every read with type heads, asking or not.

## Claim

Written by hand, as claude-347's to claude-352's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-353/**`
- `crates/uor-r4-integer/src/stack/hamming.rs` and `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/research/notebook/**` (one row a result)

## Inputs

- claude-352's kit on the laptop, `local/claude/claude-352/`: the four context-8256 artifacts, the 192 and 256 prompts, the pod's float replies at 192, the kit of places and the pinned scorer.
