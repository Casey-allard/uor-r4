# ORDER: one-model-end-to-end, claude-339

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `fbf15e56f`.

## Goal

**Train (b64L) on claude-322's TRAIN with the ask heads, the question carry and the found-copy pull, serve it, and score whether replies start to carry the asked value.**

- **Why now.** Recall is the gap after stopping: no trunk puts a rare value from the prompt into its reply.
  - On the recall task, the ask heads with carry 4 answered at both seeds when pulled at both placements (claude-335, B2-aqC).
  - claude-337 gave the language trainer the pull at its scored rows. claude-338 made the scorer serve the ask heads and the carry (`--engine integer`).
- **The risk, named first.** A language window has no question row, so only the scored placement is open to it. On the recall task that placement (B2-aC) passed at seed 2 and failed at seed 1. There it was undertrained and still rising at 16,384 steps (`R_ask` 0.04 to 0.24 in the last 2,048). The prep sheet must say how this run answers that: more steps, a third seed, or a reading that separates "not learned yet" from "not learned".

**The work:**
1. **A prep sheet:** `docs/harness/prep/one-model-end-to-end-language-carry-2026-11-09.md`. It covers question, arms, readings, bars, the outcome map, instrument checks, steps, cost and stop rule, with a non-author FIT before any pod.
2. **Arms, on claude-322's (b64L) recipe and TRAIN d3mwcap6:**
   - **L-aC:** `--hamming-ask 1 --hamming-carry-question 32 --hamming-attract 1`, the pull at the scored rows. The reach is 32, not 4: the sheet's FIT measured the question and header at up to 19 rows before the reply (the sheet's Arms section).
   - **L-a:** `--hamming-ask 1 --hamming-attract 1`, no carry. This is the no-carry control (L-010).
   - claude-322's sealed b64L-d3mwcap6 serves as the no-ask line.
   - Two seeds per arm, unless the sheet's FIT says otherwise.
3. **The −C cut:** L-aC served without its carry. If no export path drops the carry, the sheet names that as an instrument gap and the cut moves to the float scorer.
4. **Serving:** `fbs-snapshot`, then `fbs-export --run-config`, then `served-reply-panel --engine integer` at the pin. Also the held-out human-text panels claude-322 read.
5. **The pre-run review (D67)** before the pods, and a probe ladder: smoke, then the full run.

**Claim check:**
- **(a) Step:** step 4, one model trained, served and scored end to end, for the recall gap.
- **(b) What it unblocks:** whether the ask heads and the carry move recall in served replies. That is the outcome map's next step once stopping holds.

**Not in scope, named here:**
- New mechanisms or new training signals.
- The CPU port (DeepSeek's lane).
- A content-readout or turn-memory artifact (claude-338's named limits).

## Cancel probe (run first)

1. No open PR touches the item's files.
2. claude-338 is merged and verified.
3. Disk: at least 20 GiB free.
4. `uor-pod status`: room inside the caps (4 pods, $8/h).

**Result (at claim):** pass.
- No open PR touches the item's files.
- claude-338 merged as `fbf15e56f3ab1961b30610893392e7a97fd0ab18` and was verified on fresh `main` (`served-reply-panel` 23 / 0).
- 27 GiB free.
- 0 pods running. RTX 5090 offers from $0.563/h.
- `loop next` offers only DeepSeek's CPU-repair direction, so this ORDER is hand-written under Casey's harness exception, as claude-331 to claude-338 were.

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/labs/one-model-end-to-end/**` (only if an instrument check needs a change)
- `crates/uor-r4-float-train/src/{fbs_export.rs,in_space_long.rs}` (added 9 Nov for the ask-range repair, the sheet's row 0a)

## Done when

1. The sheet has a FIT and the pre-run review passed.
2. Every arm ran at both seeds, or the stop rule ended it and the REPORT says where.
3. Each arm is served and scored on m2-fresh256 with `--engine integer`. The REPORT reads each against the sheet's outcome map, negatives first, as one notebook row.
4. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). A failed instrument check stops the run before the full pods, and the REPORT classifies it as an instrument fault or a mechanism result (L-007).
