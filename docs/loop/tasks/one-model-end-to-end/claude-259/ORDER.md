# ORDER: one-model-end-to-end, claude-259 (a model that replies: the capacity sweep, trained as served)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `22fecde5a`.

## Goal

Casey, 2 Nov 2026: "Stop Q5 now (Recommended)", to "Stop the Q5 track now and move to one model trained on the exact values it is served with?". Then: "your goal is to get us to a working chat model using the training and serving methods we have described - you need to figure out how to stitch all of that together. I feel like we have gone on far enough that you probably need to do another general broad look at the whole project again".

**Plan step (the claim check):** step 2 of the path re-ordered by my decision review on 2 Nov (#2028 comment 6580712580; the lab's place record), "a model that replies". Step 1, chat on the trained model, is done (claude-259 of `m4-export-uorlut01-cli`, PR #3559, `0fa4c5fce`). **What it changes in the model:** its size. The unit trains the served recipe as served at four sizes and ships the size that replies best as the base of the next run. It is the first unit that changes what the model can hold since the turn memory.

## Revision, 2 Nov 2026 (this commit replaces the first version of this ORDER)

The first version fixed (r-drop) at its own size, trained as served. The broad look found that size, not the training values, is where replies are lost:
- **More steps buy nothing on DEV.** The sealed (r-drop) seed-1 run's `progress.jsonl` (`results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s1/r-drop-s1/`): DEV nearest top-1 is 11,770 of 14,287 at step 2,048 and 11,841 at step 16,384. Openings stay 602 → 602 of 1,226, copies 3,085 → 3,060 of 4,130. Over the same steps the train loss falls 0.967 → 0.559 and the DEV loss rises 2.576 → 2.673. Seven eighths of the run memorize TRAIN.
- **A reply is many tokens.** At the final stratum rates (openings 49%, copies 74%, the rest 91%) a reply of about 20 tokens is rarely right end to end. claude-218: even a forced right first token makes only about a fifth of replies exact. Copies are the weakest stratum inside a reply.
- **Width was tried once, before the memory** (claude-153, 28 Oct): width 512 with 8 heads raised copies at both seeds (2,394 / 3,265 against 1,517 / 2,933), not openings; its registered verdict was negative on one seed's miss. Nothing since then has varied the model's size.
- **The served engine takes other sizes.** The exporter's refusals (`fbs_export.rs`, `check_fbs_export_config`) name no width; the integer head serves widths up to 1,024 and head widths up to 256 (`format.rs`). `--train-as-served` refuses no width.
- **claude-155** recommended routing plus exact memory over a wider trunk. That was before the memory existed. The sweep tests the question directly and cheaply, and a no-gain outcome supports claude-155's reading (row 3 below).

This merges the old unit 2 (sheet) and unit 3 (run) into one unit, so the run is not a separate loop cycle. The old unit-3 bound stands: back to Casey after this run, with transcripts and numbers.

## Cancel probe (run first; results)

1. **A trained-as-served sheet already?** `grep -rln "train.as.served" docs/harness/prep/` finds none. Not cancelled.
2. **A size sweep since the memory?** The notebooks' only width row is claude-153's (28 Oct, no turn memory). None with `--hamming-read 8` or 16 since. Not cancelled.
3. **Inputs in the HF store:** `w2-inputs/2026-10-25/driver-data/` holds `mw-balp/train`, `ft/dev` and `tokenizer.json`; `inputs/claude/heldout-panels-2026-10-29/` holds P-dev. The (r-drop) recipe is claude-211's, which claude-220 and claude-243 served. Not cancelled.

Classed instrument (no model ran).

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/labs/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **The sheet**, `docs/harness/prep/one-model-end-to-end-capacity-2026-11-02.md`, one page:
   - the question;
   - the arms, each exact: (r-drop) with `--train-as-served 1` at width 256 / 4 heads (the base), 256 / 8, 512 / 8 and 1,024 / 16, seeds 1–3;
   - a decision table where every outcome changes a decision;
   - instrument checks: a smoke per arm at the run's head that runs `run_long` and exports, and the integer session's parity on each shipped checkpoint;
   - the data with sha256s, the cost and the stop rule (an export refusal mid-run is an instrument fault).
2. **A fresh FIT** on #3407: "TEST FITNESS: FIT".
3. **A pre-run review (D67)** by a fresh reviewer, its five points, on #3407: PASS.
4. **The run** on Vast RTX 5090s inside the caps; outputs sealed in the HF store with `SHA256SUMS`.
5. **Served and scored:** each arm's checkpoints exported to `uorlut` and served by `uor-chat`, on conv-a, conv-b and conv-c, with transcripts and decode tok/s; the decision table's reading.
6. **The REPORT**, negatives first, and one notebook row.

## Stop and ask if

Each of these is decided in the unit, never waited on:
- **The run needs a new instrument, trace or audit.** That is outside the track's bound: bring it to Casey.
- **An arm's smoke fails in the trainer or the exporter.** That is code outside this scope: drop the arm from the run, record why, and leave the repair to its own unit.
- **The projected cost passes the caps**, or an arm projects past about an hour per run: cut its steps or drop it, and record why.
