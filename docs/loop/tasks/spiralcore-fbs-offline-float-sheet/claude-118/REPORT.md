# REPORT: spiralcore-fbs-offline-float-sheet, claude-118 (scope decision)

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [`spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md) (antigravity-81, #3233).
References #2891.

## What did not work, first

- **The build could not start in this unit.** The loop guard reads an item's scope from `origin/main`, on purpose, so "a branch cannot widen its own scope" (`crates/uor-r4-harness/src/loopguard.rs`). The first write of a new kernel file under `crates/uor-r4-float-train/src/` was refused, because the restored scope was only on this branch. Under the ORDER's stop rule, this unit lands the scope decision alone, and the build and run move to the next claim.
- **No result yet.** The item's done-when (descent and tokens per second from the offline-float sheet, with a matched control) is not met, so this PR says References, not Closes.

## What changed

- **`docs/roadmap/items/spiralcore-fbs-offline-float-sheet.md`:** its `scope:` line gains `crates/uor-r4-float-train/**`, by a D68 decision. A fresh-context non-author decision reviewer returned APPROVE; the owner log is #2891 comment `6405361056`. The decision:
  - It restores what #3232 removed while #3231 held the crate. #3231 merged at 18:05Z on 25 Oct.
  - `crates/uor-r4-integer/**` stays out: float-train does not depend on it, and the Hamilton runner keeps its integer oracle inside float-train.
  - No open PR touches the crate.
  - No arm, bar or pin of the registered sheet changes. The three §5 kernel pins still match.
- **The ORDER** records the cancel probe, the D68 conditions and the build design for the next unit. The conditions:
  - the pinned `spiralcore_fbs.cu`, `spiralcore_fbs.rs` and `cuda_stack.cu` stay byte-identical;
  - the arm kernels go in a new `.cu` file, with its own PTX module;
  - no new dependency;
  - the run receipt records the new kernel hashes.

## Measurements

None. This is a scope decision, not a result.

## Checks run at the head

See the PR: `loop report --check`, `loop pr-check --local`, and the exact-head review.

## Skills used

- holding-the-goal: the guard's refusal was read as the process working as designed (scope lands on main first), not as an obstacle to route around.

## Still open

- **The build and run:**
  - the identity-transport and 2-atom arm kernels;
  - the frozen-query arm, which reuses the zero-credit hook and keeps the queries out of weight decay;
  - the sheet runner `spiralcore-fbs-sheet --mode synthetic-probe|throughput-probe|full`, reusing the Hamilton runner's loader, evaluator and Q12 export path;
  - a D67 pre-run review, then one RTX 5090.

  Kernel drafts and the design are kept outside the repository in the lab's local notes for the next claim.

## Plain summary

The registered test of SpiralCore/FBS projections in float training needs new code in the float training crate, which the item's scope had excluded while another lab worked there. That work has merged, so I had the scope change reviewed and approved, and this pull request makes it. The project's guard only lets a branch write to files that the scope on main already allows, so the code itself comes in the next pull request.
