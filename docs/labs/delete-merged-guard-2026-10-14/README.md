# `git delete-merged`, ported with its whole four-step guard chain

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the second half of the objective's "only with their guards"

## What landed

**`uor-harness git delete-merged` is Rust, and all four guards came with it.**

`git switch` landed last unit; **this is the other half of the pair, and the one with the recorded
data loss behind it.**

## The mistake this guard exists for

> I merged a pull request with `gh pr merge --auto`, polled its state, and then ran the remote delete
> **unconditionally**. The PR had auto-closed as DIRTY midway through the poll; the loop exited without
> noticing, the delete ran anyway, and **the only remote copy of unmerged work was removed.**

**The work survived in a local commit and was recovered — but nothing in the command sequence prevented
the loss.**

## The four steps, each against a specific failure

| # | guard | the failure it prevents |
|---|---|---|
| 1 | **a live claim, FIRST, needing no network** | *"a merged PR is not proof a branch is finished"* — and gating on the merge first meant **an unreachable GitHub returned before this check ran, so a claimed branch could be deleted whenever the network was down: the protection failed exactly when the evidence was weakest** |
| 2 | a merged PR whose merge commit **is an ancestor of `origin/main`** | a PR marked merged whose commit is **not** on main |
| 3 | **`git cherry`** — does the branch introduce anything upstream lacks? | see below |
| 4 | **`safe_to_delete` is the ONLY flag the caller acts on** | *"an earlier version used `merged` and a guard returned early without clearing it, so a REFUSED branch still printed 'would delete'"* |

**Step 3 is where two earlier attempts were wrong in opposite directions**, which is why the port keeps
the reasoning in the code: **`rev-list --count` (ancestry) refused every squash-merged branch**, because
a squash leaves no ancestor; **`diff --stat` (whole tree) refused a branch merely BEHIND main**,
*"flagging five files belonging to other labs that this branch never touched."*

## Three guard tests, and the first is the whole point

| test | asserted |
|---|---|
| **a live claim refuses the delete WITHOUT a network** | refusal, the reason, **and the release command the owner needs** |
| **`--apply` does not bypass the claim guard** | the flag decides whether to act, **not whether to check** |
| **an unprovable merge clears `safe_to_delete`** | the other early return clears the acting flag too |

**The first test involves no `gh` at all** — that is the property: **it would fail if the claim check
moved below the PR lookup**, which is the exact arrangement the docstring calls *"the worst"*.

## Two divergences, both stated

- **The branch is read from `--branch` first, then positionally.** Python requires the flag; **the port
  accepts either**, because a positional form is what the sibling commands use (`git switch <target>`,
  `clean-merged`) and requiring the flag would make this one command different for no reason a lab can
  see.
- **Rust refuses at the claim guard with only `claims.json` present**, while Python cannot reach that
  guard without `docs/harness/milestones.json` — it fails loading the registry first. **That is the
  guard's own doctrine working**: *"a claim needs no network to read"*, and this port needs no
  **registry** either.

## Verification

| check | result |
|---|---|
| the claim guard, end to end, on a scratch repo | **REFUSED**, with the release command |
| three guard tests | pass |
| `cargo test -p uor-r4-harness` | **398 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **`git install-hooks` and `git status` still delegate**, as do `mistakes show|capture`,
  `multilab sync-live|migrate-ledger` and `team roster|dispatch`. **Six `delegate_to_python` sites
  remain.**
- **`deleted` is ABSENT from the JSON on a guard refusal, not `false`** — Python returns the same, and
  the port keeps it. **My first test asserted `false` and failed.** A caller reading `deleted` must
  treat missing and false alike, **which is why `safe_to_delete` is the one flag that is always
  present.**
- **`apply` deletes LOCAL first, then remote**, so a failed remote delete leaves the local copy for the
  next attempt. **A local delete that succeeds and a remote delete that fails is reported as two
  separate sides** — *"a partial delete that reads as a success is how you end up believing a branch is
  gone when it is not."*
- **The remote delete runs `git push origin --delete` unconditionally once `safe_to_delete` is set.**
  **The guard proves the merge; it does not re-verify between the local and remote deletes** — so a
  force-push landing in that window is not caught. Python has the same window.
