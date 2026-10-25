# REPORT: infra-loop-harness, claude-96

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #2856. References #2855.

## What did not work, first

- **The prose-panel unit could not write its own notebook row.** `m3-nonrelation-prose-panel`'s done-when asks for one row in `docs/research/notebook/m3-durable-memory.md`, but the item's `scope:` did not list that file. `loop guard` reads scope from `origin/main`, so claude-94 (#3188) could not widen it in place: its edit to the scope line was refused while it was only on the branch. The fix has to land on `main` first, which is this unit.
- **`loop claim infra-loop-harness` refused** ("already claimed by PR #3169"), so this ORDER is written by hand, as claude-88, claude-90 and claude-93 were.

## What changed

Two item files and this task folder. No code. The rendered roadmap page is unchanged (`render --check` current).

- **`docs/roadmap/items/m3-nonrelation-prose-panel.md`:** `scope:` adds exactly `docs/research/notebook/m3-durable-memory.md`. Nothing else in the item changes.
- **`docs/roadmap/items/m3-memory-turn-discriminator-gate.md`:** one note paragraph before "Done when". Its `SavedCompiler` references are superseded, because that compiler's heads are `SparseSoftmax` fits (CHARTER §3.2). The sieve compiler abstains whenever its frames disagree, so its result is never a pass of this gate. The bar, status, dependencies and done-when are unchanged.

## Measurements

None. This is a docs unit.

## Checks run at the head

- `cargo xtask roadmap check`: 92 items OK.
- `cargo xtask roadmap render --check`: current.
- `loop report --check`: PASS. `loop pr-check --local`: PASS (review-record and required-checks are N/A in a local run).
- The first fresh review (at `1f1b23e8e`) returned CHANGES with three wording faults, all fixed. The note now points at every `SavedCompiler` reference (above, How to start step 1, Sources). It says the baseline "is to be measured", because the compiler is still in draft #3188. Its last sentence now uses the item's own failure, refusing true statements.
- The cancel probe held: the scope line lacked the notebook; no open PR other than #3188 lists the three files; the discriminator item had no superseding note.
- The fresh review, which is also the decision review for the one-file scope addition, is in the PR's review record.

## Skills used

- scope-guard: the one file the unit's done-when needs, added on `main` through the umbrella item rather than around the guard.

## Still open

- claude-94 merges `main` and writes its notebook row once this merges.

## Plain summary

The prose-panel item can now write its own notebook row, which its done-when asks for, and the discriminator-gate item now says plainly that the softmax compiler it named is banned and that the lawful sieve compiler's clean result does not pass that gate.
