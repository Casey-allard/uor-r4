# ORDER: infra-loop-harness, claude-96 (m3-nonrelation-prose-panel: its notebook in scope; the discriminator gate's superseding note)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-88, claude-90 and claude-93). Base: `89d8cea20`.

## Goal

claude-94 (draft #3188) measures `m3-nonrelation-prose-panel` (#2856) on a lawful compiler, after a decision review (owner log on #2856, comment `6394013509`). Two item-file changes that unit needs sit outside its scope, and `loop guard` reads scope from `origin/main`, so they land here first.

1. **The item's notebook.** Its done-when asks for one row in `docs/research/notebook/m3-durable-memory.md` (step 7 of its "How to start"). The item's `scope:` does not list that file, so the unit cannot write it. Add it.
2. **The discriminator gate's note.** The decision review required a note on `m3-memory-turn-discriminator-gate` (#2855) saying that its `SavedCompiler` references are superseded: that compiler's heads are `SparseSoftmax` fits (CHARTER §3.2), and the sieve compiler's own abstention is never a pass of that gate. Add the note.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. `loop claim` refused it on 25 Oct ("already claimed by PR #3169"), so this ORDER is written by hand, as claude-88, claude-90 and claude-93 were.

## Cancel probe (run first)

Class: docs (host only). At `89d8cea20`, all three must hold, or the unit is cancelled:
- `m3-nonrelation-prose-panel.md`'s `scope:` does not list `docs/research/notebook/m3-durable-memory.md`;
- no open PR other than #3188 has an ORDER that lists that notebook, `m3-nonrelation-prose-panel.md` or `m3-memory-turn-discriminator-gate.md`;
- `m3-memory-turn-discriminator-gate.md` has no superseding note.

## Files in scope

- `docs/roadmap/items/m3-nonrelation-prose-panel.md` (the `scope:` line only)
- `docs/roadmap/items/m3-memory-turn-discriminator-gate.md` (one note paragraph)
- `docs/roadmap/README.md` (re-rendered; shared file)
- this task folder

## Done when

- The prose-panel item's `scope:` adds exactly `docs/research/notebook/m3-durable-memory.md`.
- The discriminator item carries the note. Its bar, status, dependencies and done-when are unchanged.
- `cargo xtask roadmap check` passes, and `render --check` matches.
- `loop report --check` and `loop pr-check --local` pass.
- A fresh-context review counts at the exact head. It also serves as the decision review for the one-file scope addition (D66, D68).
- The REPORT says `References #2904`, `References #2856` and `References #2855`.

## Stop and ask if

- The scope addition would reach any file other than that one notebook.
- The note would change the discriminator gate's bar, status, dependencies or done-when.
