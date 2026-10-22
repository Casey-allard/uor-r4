# ORDER: infra-loop-harness, claude-43 (a notebook row added inside the table passes the scope rule)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 5. Base: `8d488457e`.

## Goal

`loop pr-check`'s `scope` rule allows "an append to the item's `notebook:` page" through `append_only` (`loopprcheck.rs:1191`): the head must start with the base text. Every notebook page has prose after its results table. In `m1-discrete-attention.md` the last row is line 64 of 85, and the other five pages are alike. So a row added in the table, where the notebook README says rows go ("Append at the bottom" of the table), always fails as "outside scope". It failed the #2829 close (claude-42), and it would fail the #2857 close Casey just decided ("Add notebook: field").

The planted fault: **a row inserted before a notebook's trailing prose is refused.**

## Cancel probe (run first)

At `8d488457e`, does `pr-check --local` refuse a notebook row added before the page's trailing prose? If it allows it, the task is void. Class: instrument.

## Calls

- Judge the declared notebook with the existing `keeps_base_lines` (every base line kept, in order, so rows are only added), as `docs/start/FILEMAP.md` already is. Remove the now-unused `append_only`.
- The test fixture's notebook gains trailing prose, like the real pages. A new pass case inserts a row before it. The fail case, a rewritten row, still fails.
- `docs/harness/README.md` (shared) says so.

## Files in scope

- `crates/uor-r4-harness/src/loopprcheck.rs` and `crates/uor-r4-harness/tests/loop_prcheck.rs`
- `docs/harness/README.md` (shared)
- This task folder

## Done when

1. The new pass case passes, and fails with the fix set aside.
2. The rewritten-row case still fails.
3. The full checks pass: `loop_prcheck` tests, clippy `-D warnings`, `loop gates --full`, `report --check` and `pr-check --local`.

## Stop and ask if

- Notebooks must stay strictly end-appended. That would be a format change to every notebook page, which is the PM's call.
