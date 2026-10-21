# REPORT: infra-loop-pr03-render, claude-14 (loop PR 3: loop render for every view, --check at the stamp, determinism)

Lab: claude. Base: `32092fca6`. Order: [ORDER.md](ORDER.md), committed (`1c1f1a584`) before the work. The cancel probe found no `loop render`, and only `docs/roadmap/README.md` rendered today.

Fresh-context builders wrote the code, one per commit. The main session re-ran the gates and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **The item's done-when is met only in part, by design.** The ORDER's architecture calls narrowed it:
  - `docs/loop/baseline.json` is deferred to loop PR 5. Its contents are undefined, and its first consumer is `pr-check`.
  - No rendered view is committed into the real repository. Committing them is PR 8's fold-in. Committing STATE.md beside a hand-edited STATUS.md now would drift (L-001).

  So the PR says "References #2906", not "Closes".
- **HANDOFF went over the 24 KB read-path budget twice.**
  - First it was 29.5 KB, because held and decision items were listed twice.
  - Then it was 26.4 KB, because the blocked table repeated each done-when.
  - The blocked table is now `| Item | Title | Status | Owner |`. HANDOFF renders at 22.2 KB on the real repository, with all 87 items listed.
  - One commit message had claimed "within budget" too early. It was amended before any push.
- **The third commit's first builder ran out of context** with the code written but not committed. A second builder finished it.
- **Two fresh-context reviews of `8cf640dd9` asked for changes. Both are fixed in two commits.**
  - **Fix 1 (`3fc99d77a`):**
    - The AGENTS.md writer rebuilt the whole file from the committed copy. An in-place render would have dropped any uncommitted edit outside the markers; the reviewer reproduced this. The block is now spliced into the on-disk bytes between the on-disk markers, and nothing else is touched.
    - One marker missing, a duplicate marker, or reversed markers used to read as "no markers" and pass `--check`, so a deleted marker or a forged block passed. Markers are now classified as Absent, Ok or Malformed. Only Absent at the commit is a pass with a note. Committed markers that are missing or malformed on disk fail `--check`.
    - An xtask validation error was reported as "differs". Now, in render mode, any non-zero exit is REFUSED. In check mode, only exit 1 means "differs", and anything else is REFUSED.
    - The roadmap view reads the working tree. So with `--at`, or with `--check` at a stamp other than HEAD, it is skipped with a note, never compared against an older commit.
    - The renderer's git calls no longer inherit `GIT_*`. They use `-c core.quotepath=false` and `ls-tree -z`.
    - The README now says to use `--out` on the real repository until PR 8.
  - **Fix 2 (`5d73bc136`):** malformed input is refused (exit 2), never guessed at or dropped. This covers:
    - frontmatter without a closing `---`;
    - any `depends_on` that is not an inline `[...]` list;
    - a notebook with two `| Date |` tables, or without its `Number`, `Bar` and `Verdict` columns;
    - a non-recursive notebook listing;
    - a stray row on the mechanism board, and an empty board;
    - an `id: L-` that is not `L-NNN`, and duplicate lesson ids.

    In addition, YAML comment lines are never taken as a lesson title, and milestones sort numerically (`m10` after `m2`). The real repository passes every strict rule unchanged.
- **Unknown values are shown as `not recorded`.** No mistakes file has an `L-NNN` id yet, and no item has `Cancel if` or `size:`. LESSONS lists all 23 files under "Without a lesson id yet". HANDOFF shows `not recorded` and invents neither ids nor items.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/looprender.rs` (new), `lib.rs`, `loopgates.rs`, `main.rs`, `doccheck.rs` | code | 1,071 | `uor-harness loop render [--check] [--out <dir>] [--at <commit>]`. Doccheck registers `render`, plus `--check`, `--out` and `--at`; the last two are value flags. |
| `crates/uor-r4-harness/tests/loop_render.rs` (new) | tests | 934 | 31 tests over fixture repositories. `GIT_*` is stripped, and no test reaches a real cargo. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 17 | Usage, including "use `--out` until PR 8", and the loop page's "What is rendered" paragraph. |

That comes to about 2,020 lines, excluding this ORDER and REPORT.

**Design calls.**
- **Inputs at a commit.** Every input is read with `git show <commit>:<path>` or `git ls-tree -z`, with `GIT_*` stripped. No working tree, clock or network is involved. Each view's first line is the stamp `<!-- rendered by uor-harness loop render from <40-hex>; do not edit -->`.
- **`--check` exit codes.**
  - 1 when a view differs from its re-render at the stamp. The first differing line is printed.
  - 2 for a missing or garbled stamp, and for `--check --at`.
- **STATE.** Milestones (numeric order), item counts, the newest dated row per research line, the mechanism board, and the lab registry.
- **HANDOFF.**
  - Open items, with the first sentence of their done-when and cancel probe, size, and status with dependencies.
  - Blocked items listed once, with their owner.
  - Not doing.
- **LESSONS.** Only `id: L-NNN` counts as an id.
- **AGENTS block.** Written only between `<!-- loop:lessons-top-ten:start -->` and `<!-- loop:lessons-top-ten:end -->`, spliced into the on-disk file. PR 8 adds the markers, with owner approval.
- **Roadmap view.** Produced through xtask. It is skipped with `--out`, with `--at`, and when the stamp is not HEAD.

## Measurements

None. This is a host-only tool.

## Checks run at the head

The main session re-ran these at `5d73bc136`. Later commits change only this REPORT.

- `cargo test -q --locked -p uor-r4-harness --test loop_render`: 31 passed.
- **Negative gates.** Each set aside only its own lines and was restored byte-identical. In each case the named test FAILED with the lines set aside and passed once they were restored.

  | Lines set aside | Test that failed |
  |---|---|
  | `--check`'s comparison (the planted fault) | `a_hand_edited_state_fails_check` |
  | LESSONS in the view set | `a_hand_edited_lessons_fails_check` |
  | HANDOFF in the view set (run by the main session) | `a_hand_edited_handoff_fails_check` |
  | The AGENTS comparison | the AGENTS hand-edit test |
  | The old whole-file AGENTS write restored | `in_place_render_keeps_uncommitted_bytes_outside_the_markers` |
  | "Malformed means no markers" restored | `a_missing_marker_on_disk_fails_check` |
  | The old frontmatter split restored | `frontmatter_without_a_closing_line_is_refused` |
- **The real repository.** `loop render --out <tmp>` gave STATE at 4,802 bytes, HANDOFF at 22,200 bytes and LESSONS at 3,906 bytes. A second render was byte-identical, `--check --out` exited 0, and AGENTS.md was unchanged.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 730 passed, 0 failed.
- `cargo fmt --check` is clean. Clippy reports no warnings in the touched files.
- The PR body lists the claim-wording, `xtask links` and local copyright checks.

## Skills used

None of the private kit's skills.

## Still open

- **`docs/loop/baseline.json`** comes with loop PR 5.
- **PR 8:**
  - commit the live views;
  - retire STATUS.md and NOW.md;
  - add the AGENTS.md markers;
  - assign `L-NNN` ids;
  - pin `eol=lf` for the views, because a CRLF checkout would fail `--check`.
- **The policy gate**, in a later PR: a lab PR that touches a rendered view fails, and so does a freshness check.
- **`--check` accepts any reachable commit in its stamp.** That matches the spec. PR 8 may require the stamp to be an ancestor of HEAD with no input change in between.
- **Writes are not atomic.** A failure part-way through can leave a mix of views.
- **HANDOFF has about 2 KB of headroom.** It grows as items gain `Cancel if` and `size`.

## Plain summary

`uor-harness loop render` writes the project's status pages from what is committed at one named commit, and stamps that commit on each page. `--check` re-renders at the stamp and fails on any difference, so a hand edit is caught. Malformed inputs are refused, not guessed at. The real repository switches to these pages in loop PR 8.
