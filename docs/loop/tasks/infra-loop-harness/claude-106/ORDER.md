# ORDER: infra-loop-harness, claude-106 (restore the discrete crate to carrier-credit-hard-selection's scope, briefly)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-104 and -105). Base: `8440a02ce`.

## Goal

One item-file fix, a D68 scope decision. It puts `crates/uor-r4-discrete/**` back into `carrier-credit-hard-selection`'s `scope:`, as claude-105 (#3222) said it would if #3203's review asked for a code fix.

**Why.**
- #3203's exact-head review, slice 4, found a Required defect. `CarrierRelease::validate_for` counted and de-duplicated a release's arms without checking that each was registered, so seven registered arms plus one stranger would pass.
- The fix is committed on #3203. Its negative test, and one advisory (no panel-fit label beside an "instrument" credit result), are in `crates/uor-r4-discrete/src/carrier_arms.rs`.
- The guard now refuses those edits, because the item's scope on `origin/main` no longer holds the crate.

**Keeping the lock short.** #3203 now lands as the code and design delivery: the kernel, the schedules, the runner, the grader, the sheet, the release and the plan. Its open order, and so the crate lock, ends on that merge. The GPU arms' results follow in a docs-only unit after the controls finish, which projects to about 18:30Z.

**Rejected alternatives.**
- **Ship the fix without its test:** a correctness fix needs a test that fails without it.
- **Keep #3203 open until the results:** that holds the lock about three more hours.

**Why the infra-loop-harness item.** It is the same as claude-104 and -105: Codex's idle #3169 holds the umbrella, with no overlap, so this ORDER is hand-written.

## Cancel probe (run first)

Class: records. Run at `8440a02ce`; all must hold:
- The item's `scope:` lacks the crate. **Holds.**
- The guard refused the two remaining edits on claude-103's worktree. **Holds:** "REFUSED: crates/uor-r4-discrete/src/carrier_arms.rs is outside active ORDER … scope [docs/…]".
- No open PR touches the item file. **Holds.**

## Files in scope

- `docs/roadmap/items/carrier-credit-hard-selection.md` (the `scope:` line and one note)
- this task folder

## Done when

- The edit is made, and `roadmap check` passes.
- A fresh decision and exact-head review approves it.
- The checks pass, and the REPORT says `References #2904` and `References #2819`.

## Stop and ask if

Stop and decide, never stop and wait.

- The review shows a way to land the test without restoring the glob.
