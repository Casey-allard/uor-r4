# #2443 — Port git delete-merged with its whole four-step guard chain (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T05:07:13Z
- **Updated:** 2026-10-14T05:10:04Z
- **Closed:** 2026-10-14T05:09:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2443
- **Merged:** 2026-10-14T05:09:39Z
- **Merge commit:** 7a41c8352f3cc36511d06a02fe2fe2d8871b8c3d
- **Base / Head:** main / UOR-Foundation:deepseek/delete-merged
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`git delete-merged`, ported with its whole four-step guard chain.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`uor-harness git delete-merged` is Rust, and all four guards came with it.** `git switch` landed last unit; **this is the other half of the objective's "only with their guards" — and the one with the recorded data loss behind it.**

## The mistake this guard exists for

> I merged a pull request with `gh pr merge --auto`, polled its state, and then ran the remote delete **unconditionally**. The PR had auto-closed as DIRTY midway through the poll; the loop exited without noticing, the delete ran anyway, and **the only remote copy of unmerged work was removed.**

**The work survived in a local commit and was recovered — but nothing in the command sequence prevented the loss.**

## The four steps, each against a specific failure

| # | guard | the failure it prevents |
|---|---|---|
| 1 | **a live claim, FIRST, needing no network** | *"a merged PR is not proof a branch is finished"* — gating on the merge first meant **an unreachable GitHub returned before this check ran, so a claimed branch could be deleted whenever the network was down: the protection failed exactly when the evidence was weakest** |
| 2 | a merged PR whose merge commit **is an ancestor of `origin/main`** | a PR marked merged whose commit is **not** on main |
| 3 | **`git cherry`** | **two earlier attempts were wrong in opposite directions**: `rev-list --count` refused every squash merge; `diff --stat` refused a branch merely *behind* main, *"flagging five files belonging to other labs that this branch never touched"* |
| 4 | **`safe_to_delete` is the ONLY flag the caller acts on** | *"an earlier version used `merged` and a guard returned early without clearing it, so a REFUSED branch still printed 'would delete'"* |

## Three guard tests — the first is the whole point

| test | asserted |
|---|---|
| **a live claim refuses WITHOUT a network** | the reason, **and the release command the owner needs** |
| **`--apply` does not bypass the claim guard** | the flag decides whether to act, **not whether to check** |
| **an unprovable merge clears `safe_to_delete`** | the other early return clears the acting flag too |

**The first test involves no `gh` at all** — that is the property: **it would fail if the claim check moved below the PR lookup**, the exact arrangement the docstring calls *"the worst"*.

## Two divergences, both stated

- **The branch is read from `--branch` first, then positionally.** Python requires the flag; **the port accepts either**, because a positional form is what the sibling commands use.
- **Rust refuses with only `claims.json` present**, while Python cannot reach that guard without `milestones.json` — it fails loading the registry first. **That is the guard's own doctrine working**: *"a claim needs no network to read"* — and this port needs no registry either.

```
cargo test -p uor-r4-harness                    -> 398 passed, 4 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/delete-merged-guard-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`git install-hooks`/`status` still delegate**, as do `mistakes show|capture`, `multilab sync-live|migrate-ledger` and `team roster|dispatch` — **six `delegate_to_python` sites remain.**
- **`deleted` is ABSENT on a guard refusal, not `false`** — Python returns the same. **My first test asserted `false` and failed.** A caller must treat missing and false alike, **which is why `safe_to_delete` is the one flag always present.**
- **`apply` deletes local first, then remote**, and **reports the two sides separately** — *"a partial delete that reads as a success is how you end up believing a branch is gone when it is not."*
- **The remote delete runs once `safe_to_delete` is set; the guard does not re-verify between the local and remote deletes** — so a force-push landing in that window is not caught. **Python has the same window.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

