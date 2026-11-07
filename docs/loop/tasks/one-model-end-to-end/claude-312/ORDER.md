# ORDER: one-model-end-to-end, claude-312

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `bb76798a5`.

## Goal

**The prep sheet for (b64L) on `mw-balp/train`, its FIT, and its pre-run review.** claude-310 (#3829) and claude-311 (#3830) left one confound. (b64L), trained on D3 (EOS 756 per million), never stops on `m2-fresh256`. claude-259's (b) and (h), trained on `mw-balp/train` (87,788 per million), stop on 227–256 of 256. But (b) differs from (b64L) in four ways: objective `cell` against `nearest`, batch 16 against 64, turn memory 8 against none, and 8,192 steps against 16,384.

The sheet registers the clean test: claude-301's (b64L) recipe at its pin `ff5156bcc`, byte-for-byte, with only `--train-split` changed from D3 to `mw-balp/train`. Seeds 1 and 2. The served integer artifact is then scored by `served-reply-panel`, and DEV top-1 is read in-run.

Claim check: (a) the step is "pipeline first" (#2028, 6709947526): the next training run is judged by the served-reply score. (b) What it changes is TRAIN alone, so a change in stopping can be credited to the corpus.

## Cancel probe (run first)

1. No open PR or claim trains (b64L) on `mw-balp/train`.
2. `mw-balp/train` in the store hashes to `3dee858f…` (tokens) and its mask to claude-273's `b5e7352d…`.
3. claude-301's pin `ff5156bcc` and its job scripts are reachable.

## The sheet must register, before any run

- **Arms:** one arm, (b64L-mw), at seeds 1 and 2. The comparison is claude-301's sealed (b64L-D3) and claude-309's scores.
- **Bars, fixed in advance:**
  - **Stopping:** EOS stops on `m2-fresh256` at each seed, against (b64L-D3)'s 0 of 256. "Corpus makes it stop" holds if both seeds stop on EOS on at least 128 of 256.
  - **The panel's own bar** (104 rows, 52 pairs), reported.
  - **DEV top-1 and the no-memory panel,** reported against (b64L-D3) as the cost of the swap.
- **Repetition:** about 27 passes over `mw-balp/train`'s 7.5M tokens at 201M positions. The sheet says what that implies, and whether a stock-line collapse (claude-311) counts against the stopping bar.
- **Cost:** a lease inside the caps, about $6 for two RTX 5090 runs (claude-301's per-seed I-t of 0.63–0.70 s/step at 16,384 steps).

## Files in scope

- `docs/harness/prep/one-model-end-to-end-b64l-mwbalp-2026-11-07.md`
- `docs/loop/tasks/one-model-end-to-end/claude-312/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The sheet is written, a non-author FIT is recorded on it, and its findings are applied.
2. The pre-run review (§5.10) is recorded, GO or NO-GO.
3. The REPORT, negatives first, and one notebook row.

The run itself is the next unit.

## Stop and ask if

Nothing waits on Casey (§5.1). No spend in this unit. Bound: one unit.
