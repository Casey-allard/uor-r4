# ORDER: infra-loop-harness, claude-105 (release the discrete crate from carrier-credit-hard-selection's scope)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-101 and -104). Base: `77c241c70`.

## Goal

One item-file fix, a scope decision under D68. It removes `crates/uor-r4-discrete/**` from the `scope:` of `carrier-credit-hard-selection` (#2819).

**Why.**
- `loop claim` checks a new claim's scope against the item scope, on `origin/main`, of every open order: open worktrees, plus ORDER files in open PRs (`crates/uor-r4-harness/src/loopclaim.rs`:12–22, 680–725).
- claude-104 (#3206) added the crate glob so claude-103 (#3203) could build its schedule. claude-103 is still open while its GPU arms run, which projects to about 18:30Z.
- So the glob now locks every discrete item against every lab. On 25 Oct at 15:4xZ, `loop next` listed nine ready discrete items as unclaimable for that reason, among them the M1 path's `m1-bvg-k-stage-t-training`, `m1-position-mechanism`, `m1-normalization-mechanism`, `m1-address-key-full-vector` and `m1-faster-writer`.
- Every glob of a specific file overlaps those items' whole-crate globs. So narrowing to named files would not release them; only removing the crate does.

**Why it is safe.**
- claude-103's crate work is complete and committed on #3203: the kernel, the schedules, the runner, the grader and their tests.
- What remains is the arms' results and docs, all inside the item's docs scope.
- #3203's crate edits will show as an advisory `scope` WARN in `pr-check`. Scope is advisory under D76; the hard rules are order, banned-presence, report, owner-paths, review-record and required-checks.
- `loop guard` will then refuse further crate edits on claude-103's worktree. If #3203's exact-head review asks for a code fix, the glob is restored first, through this umbrella, and the fix follows.

**Rejected alternatives.**
- **Keep the lock until the arms finish:** that blocks the M1 path for hours over a research item.
- **Narrow to the files #3203 touches:** they still overlap every whole-crate glob.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. Codex's idle #3169 still holds an order on it, and #3169's five files (`PLAN.md`, `docs/integration/DECISIONS.md` and its task folder) do not overlap this unit's. So this ORDER is hand-written, under the run file's umbrella exception.

## Cancel probe (run first)

Class: records. Run at `77c241c70`; all must hold, or the unit is cancelled:
- The item's `scope:` holds `crates/uor-r4-discrete/**`. **Holds.**
- claude-103's order is open. **Holds:** #3203 is an open draft.
- `loop next` lists discrete items as blocked by it. **Holds:** nine items.
- No open PR touches the item file. **Holds:** #3203 changes only claude-103's task folder, the lab folder, the sheet and the crate.

## Files in scope

- `docs/roadmap/items/carrier-credit-hard-selection.md` (the `scope:` line and one note)
- this task folder

## Done when

- The edit is made, and `roadmap check` passes.
- A fresh decision review and exact-head review approve it.
- `loop report --check` and `loop pr-check --local` pass.
- After the merge, `loop next` no longer lists discrete items as blocked by `carrier-credit-hard-selection`.
- The REPORT says `References #2904` and `References #2819`.

## Stop and ask if

Stop and decide, never stop and wait.

- The review finds another lock path that this edit does not release.
