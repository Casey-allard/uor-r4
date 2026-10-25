# ORDER: infra-loop-harness, claude-108 (carrier-credit-hard-selection's scope for its docs-only results unit)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-104, -105 and -106). Base: `cada129ea`.

## Goal

One item-file fix, a scope decision under D68, in `carrier-credit-hard-selection`'s `scope:`:
- remove `crates/uor-r4-discrete/**`;
- add the two docs pages its results unit writes: `docs/research/notebook/m1-discrete-attention.md` (the item's `notebook:`) and `docs/mechanisms/carrier-credit.md` (the mechanism page's tried table).

**Why.**
- claude-106 (#3224) restored the crate glob for #3203's review fixes, and said the glob could go again before the results unit is claimed.
- #3203 merged as `cada129ea`. The crate work is complete: the kernel, the schedules, the runner, the grader and their tests.
- What remains is docs only:
  - grading the eight arms with the merged `carrier-arms grade`, on the Mac;
  - the REPORT, the notebook row, the mechanism page's tried table and the lab README.
- `loop claim` checks a new claim's scope against every open order's item scope on `origin/main` (`crates/uor-r4-harness/src/loopclaim.rs`, 680–725).
  - With the glob, the results order would lock every discrete item against every lab until the controls finish, about 18:30Z.
  - claude-107 (#3226) now holds `m1-bvg-k-stage-t-training`, whose scope holds the crate. So the results claim would be refused against claude's own Stage T order.
- The two added pages lock nothing:
  - `loop claim` skips notebook paths when it counts overlap (`loopclaim.rs:637-638`), and only `m1-bvg-k-block-apply-c2` (done) holds that notebook anyway;
  - no other item's scope holds `docs/mechanisms/carrier-credit.md`, or a glob that covers it.

**Rejected alternatives.**
- **Claim the results unit with the glob:** that locks the crate for hours, and `loop claim` refuses it against claude-107.
- **Write the notebook row and the mechanism page outside the scope:** scope is advisory (D76), but the guard refuses the edits, and the scope should say where the unit writes.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. Codex's idle #3169 still holds an order on it. Its files (`PLAN.md`, `docs/integration/DECISIONS.md` and its task folder) do not overlap this unit's, so this ORDER is hand-written under the run file's umbrella exception, as in claude-104 to -106.

## Cancel probe (run first)

Class: records. Run at `cada129ea`; all must hold, or the unit is cancelled:
- The item's `scope:` holds `crates/uor-r4-discrete/**`. **Holds** (line 11).
- #3203 is merged, and claude-103's worktree is removed. **Holds:** MERGED as `cada129ea`; the worktree path no longer exists.
- The item's results are still owed: the item `status:` is not `done`. **Holds:** `status: ready`.
- No open PR touches the item file. **Holds:** the open-PR file search returns `[]`.

Result: all hold, so the unit goes ahead.

## Files in scope

- `docs/roadmap/items/carrier-credit-hard-selection.md` (the `scope:` line and one note)
- this task folder

## Done when

- The edit is made, and `roadmap check` passes.
- A fresh decision review and exact-head review approve it.
- `loop report --check` and `loop pr-check --local` pass.
- After the merge, `loop claim carrier-credit-hard-selection` is not refused for a crate overlap.
- The REPORT says `References #2904` and `References #2819`.

## Stop and ask if

Stop and decide, never stop and wait.

- The review finds that the results unit needs a crate edit, for example a grader defect found while grading. Then the glob stays, and the results unit is claimed after claude-107 releases the crate.
