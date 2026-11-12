# ORDER: one-model-end-to-end, claude-354

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `17b7b86399ad065d636e0fb13e0537b8d4e0f14d`.

## Goal

**Find why the laptop copies still give other replies than their trained models on about a quarter of the rows, and repair the reader if the cause is in it.**

**Why now.** [claude-353](../claude-353/REPORT.md) repaired one reader fault and left 0.18 to 0.27 of the replies differing on every model (arm r's export 0.21), against a bar of 0.10. On 12 Nov 2026 (UTC), in chat, the lab asked "Hunt the remaining difference on the laptop next, no pod?" and the owner answered **"Yes, hunt it"**. That approves this laptop-only unit. It approves no pod, no float run and no new arm.

## What runs

Laptop only: saved reply files, the integer reader, the code.

| Step | What |
|---|---|
| 1 | For the four stage 3 models and arm r: the reply step at which the laptop copy first leaves the trained model, and what the trained model says at that step with its turn memory off. |
| 2 | The code the counts of step 1 point to, read on both sides. |
| 3 | If a reader fault is found: the change, a test that fails without it, and the models re-read on the same prompts with the counts before and after. |
| 4 | If none is found: what is ruled out, with counts, and what a float trace would have to record. |

## Decisions of this unit (claude's own decision review; logged for Casey, who can reverse them)

1. **claude-352's registered reading stays NOT_RUN.** No row of its sheet is read here.
2. **The float read is the definition.** If the cause is on the float side, it is reported, not changed: a float change cannot be tested on the laptop.
3. **The reader program may gain a reporting option** (for a given prefix, the rank and score gap of a given token). It serves integer artifacts only.
4. **No count is taken before this ORDER.** claude-353's kit and reply files are the inputs.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The table of step 1 is in the REPORT for all five models. | the REPORT |
| G2 | With a fault found: its test fails without the change, each part set aside on its own, and the integer crate's stack tests pass at the head. | the test runs |
| G3 | With a fault found: the counts of step 3, before and after, for every model re-read. | the REPORT |
| G4 | The REPORT says plainly how far each model is from 0.10 at the end, and what is still not explained. | the REPORT |

## Not established by this unit

- Anything a float trace would show; none is made.
- Anything about open text or conversations longer than the context. No milestone moves.

## Cancel probe (run first)

- **What would make this unit unnecessary:** laptop replies within 0.10 of the trained models' at `main`.
- **Result (at claim):** none. At the base they differ on 275, 197, 245 and 300 of 1,108 (claude-353).

## Claim

Written by hand, as claude-347's to claude-353's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-354/**`
- `crates/uor-r4-integer/src/stack/**`
- `docs/labs/one-model-end-to-end/instrument/**`
- `docs/research/notebook/**` (one row a result)

## Inputs

- claude-352's kit and claude-353's reply files on the laptop: `local/claude/claude-352/`, `local/claude/export-repair/`.
