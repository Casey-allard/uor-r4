# ORDER: infra-loop-pr03-render, claude-14 (loop PR 3: loop render for every view, --check at the stamp, determinism)

Lab: claude. Item: [infra-loop-pr03-render](../../../../roadmap/items/infra-loop-pr03-render.md) (#2906). Setup row: 3. Base: `32092fca6`.

## Goal

This is loop PR 3 from the build plan. Under D52 as amended (#2919), Claude builds every setup PR while it runs alone. The item's done-when is:

> `loop render` writes STATE, HANDOFF, LESSONS, the AGENTS.md top-ten block, `docs/roadmap/README.md` and `docs/loop/baseline.json` from committed inputs only, with no clock and no GitHub. `--check` re-renders at the stamp and needs an exact match, and two renders of one commit are byte-identical. The planted fault turns the check red: **a hand-edited STATE fails.**

## Cancel probe (run first)

Does a renderer for these views already exist?

Result at `32092fca6`:
- `uor-harness` has no `loop render`.
- `STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md` and `docs/loop/baseline.json` do not exist.
- Only `docs/roadmap/README.md` is rendered today, by `cargo xtask roadmap render`.

The task stands. Class: instrument.

## Architecture calls

These are Claude's calls as architecture lead under D52. They narrow the item, they are reversible, and the REPORT records them.

1. **Inputs come from the commit, not the working tree.** The renderer reads every input at a named commit (`git show <commit>:<path>`, or `git ls-tree`). That makes the output a pure function of that commit. The stamp line names the commit. `--check` reads the stamp, re-renders at that commit, and needs a byte-exact match.
2. **The real repository gets no new rendered files in this PR.** STATE "replaces STATUS.md", and the loop page says "The fold-in to STATE, HANDOFF and PLAN is PR 8". If STATE.md were committed beside a hand-edited STATUS.md, two status files would drift: lesson L-001. So PR 3 tests the renderer and `--check` on fixture repositories, and `loop render --out <dir>` writes a preview of the real repository's views for inspection. PR 8 commits the live views and retires the old files.
3. **The AGENTS.md top-ten block is written only between markers that already exist.** AGENTS.md is an owner path. PR 8 adds its markers with owner approval. Until then, the block is reported as "no markers" rather than written.
4. **The roadmap view reuses today's renderer.** `loop render` produces `docs/roadmap/README.md` by running `cargo run -q --locked -p xtask -- roadmap render`, in the same way the gates resolve cargo. A second roadmap renderer would drift.
5. **`docs/loop/baseline.json` is deferred to loop PR 5.** The loop page lists it as rendered but does not define its contents. Its first consumer is PR 5's `pr-check`, for example the link baseline that PR 4 found missing. So the item's done-when is met in part here, and the PR says "References #2906", not "Closes".
6. **LESSONS reads an `id: L-NNN` line from each mistakes file when one is present.** PR 8 assigns the ids. Until then, a file with no id is listed under a heading for files without an id, and is never given an invented id.

## Files in scope

- `crates/uor-r4-harness/src/**`: a new render module, and its wiring under `loop` in `main.rs`. If needed, the `doccheck.rs` tables for the new `render` action and its flags.
- `crates/uor-r4-harness/tests/**`
- `docs/harness/README.md`: usage for `loop render`.
- `docs/loop/README.md`: only the "What is rendered" paragraph, and only to say what is built.
- `docs/loop/tasks/infra-loop-harness/claude-14/**`

## Done when

- `uor-harness loop render [--check] [--out <dir>] [--at <commit>]` renders STATE, HANDOFF and LESSONS from committed inputs, plus the AGENTS block when its markers exist and the roadmap view through xtask. Each carries a stamp naming the input commit.
  - STATE: milestone bars and holds from `docs/harness/milestones.json`, item counts by status from `docs/roadmap/items`, the newest notebook row per research line, the mechanism status lines, and the lab registry.
  - HANDOFF: open items with id, title, done-when, cancel probe, estimate and status, plus a blocked table and a not-doing list, as far as today's items carry them.
  - LESSONS: from `docs/harness/mistakes/*.md`.
- The tool reads no clock and makes no GitHub call. Two renders of one commit are byte-identical, and a test pins this.
- `--check` re-renders at the stamp and fails on any byte difference. The planted fault holds: a hand-edited STATE fails `--check`, and a test pins it.
- The harness tests pass, including the full suite, along with claim-wording, links and the local copyright check.

## Stop and ask if

- An input the loop page names does not exist in a form the renderer can read without guessing.
- Any file outside scope needs to change.
