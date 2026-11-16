# ORDER: m2-donor-corpus-generation, claude-405 (the D87 corpus recorded as a protected asset)

Lab: claude. Item: [m2-donor-corpus-generation](../../../../roadmap/items/m2-donor-corpus-generation.md).
Base: `origin/main` at `a454a250a`.

## Why this unit

The D87 donor corpus was sealed and merged on 16 Nov 2026 (claude-400, PR #4028). Generating it cost about $854
corpus-only. Casey asked the same day, in claude's chat, verbatim: "Since this cost us almost 1000$ to generate can
you please make sure you note exactly how valuable it is for the future". This unit writes that record. It runs
nothing, moves no file and touches no store.

## Files in scope

- `docs/model/data-and-tokenizer.md`, which gets:
  - a new section, "The D87 donor corpus: a protected asset";
  - its row in the corpora table and in the status table;
  - `distill-d1` marked superseded.
- `docs/loop/tasks/m2-donor-corpus-generation/claude-405/**`

## Done when

1. The section states, each with its source:
   - what the corpus is;
   - what it cost;
   - why it matters;
   - where both verified copies live;
   - the rules for keeping it (never delete, move or rewrite; derived streams are caches; how to verify a copy).
2. Every figure matches the claude-400 or claude-403 REPORT, or the TRAIN pins it is compared with.
3. `loop pr-check --local` passes.
