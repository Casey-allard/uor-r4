# Integration queue and branch dispositions

> **October 1 supersession:** the [active canonical plan](../integration/project-track.md) under D19 owns current priorities and dependencies. This document preserves its dated evidence and former schedule; live issue claims must be refreshed.

This is the concise routing index for existing work. Live issues, exact PR heads
own changing status (the former `codex/lab-state` record is archived in `docs/history/lab-state-20261009/`). Refresh an entry before acting; a merge
does not imply that every review finding or scientific gate passed. The research
stages remain in [the continuing plan](plan-2026-09-29.md). All labs start with
their [extended goal](README.md), then claim one concrete next action.

## September 30 reconciliation

At `0dcdb6e970fc94d2c47f5ea879b1e7a2db620abd`, the initial ten open PRs became
six: #1521, #1490, #1505 and #1506 merged while the inventory was collected.
#1522 then merged as `b3c32170` at 03:00 UTC, leaving five of the original PRs
open; its author reported missing coordination/review receipts. Reconcile that
policy activation independently of the merge count.
#1521's delivered tree equals its independently reviewed `a345e766` head.
#1490, #1505 and #1506 have unresolved findings requiring corrective PRs.
No model capability is promoted by this reconciliation.

The inventory covers 975 live remote heads, 795 local branches, 977 remote
tracking refs and 1,784 total refs, plus tracked/untracked status for 40
worktrees. Counts overlap: local and remote refs often name the same work.
1,554 of the 1,772 branch/tracking refs mapped exactly to historical merged PR
heads; 53 unmapped refs represent 40 distinct heads. Squash history means
`git cherry` alone cannot establish missing source. Nothing was pruned or deleted.

The [frozen inventory and retention records](../../archive/lab-state-20261009/records/integration/20260930)
retain each branch/head, PR mapping, worktree state, proposed steward, next
action and limitations. All unmapped refs have a preservation reason and
reactivation condition. This is an index for scoped review, not proof that every
old branch is obsolete or every ignored file is disposable.

## Next actions, in dependency order

Stewards and reviewers below are initial routing assignments, **not active
claims or proof a client has started**. The first available lab may replace an
unavailable steward through the shared claim/handoff protocol. Every changed
PR head needs a fresh head-bound review/delivery receipt.

| Existing work | Initial steward / independent reader | Required next action |
|---|---|---|
| #1521, merged `7895501d` | Codex / another non-author lab | Source delivery complete. Keep #1520/#1523–#1525 acceptance open where actual host, live takeover, adapter or administrative enforcement evidence remains absent. |
| #1506, merged `0dcdb6e9` | Claude / Codex | Small corrective PR: refuse snapped containers at D10 `from_artifact`, restore or refuse saved-snap export through actual callers, retain exact parity evidence and scope unclassified later mismatches. [Required review](https://github.com/UOR-Foundation/uor-r4/pull/1506#issuecomment-5903194947). No training rerun. |
| #1490, merged `e01d9948` | Antigravity / OpenCode | Correct real saved-codec export, which can load `served=None` and fall back to RTN; fix the test comparing containers with different serialized provenance. Coordinate shared exporter paths with Claude before editing. [Required review](https://github.com/UOR-Foundation/uor-r4/pull/1490#issuecomment-5903194602). |
| #1505, merged `ae7f70fd` | OpenCode / Claude | Follow-up wording/evidence PR for the four unresolved findings: joint-key interpretation, registration/amendment timing, convergence and overlapping-run cost. Preserve numbers and the negative; no model rerun. [Exact review](https://github.com/UOR-Foundation/uor-r4/pull/1505#issuecomment-5902521464). |
| B0 `lab/opencode/b0-flock`, `5b7d1685` | OpenCode / Claude and Codex as consumers | Recover existing reviews, publish the existing branch as a PR, and agree one float selector interface. A1 calls `flock::top_k_select`, absent from this B0 head. Its source is implemented, not yet compiled/executed under the recovery hold. |
| A1 `lab/claude/a1-pointer-fixes`, `b806fb2d`, and `lab/claude/a1-mworld-cells`, `76c8dac2` | Claude / OpenCode | Preserve both branches, integrate pointer fixes and world cells through the canonical B0 API, then focused consumer checks before the admitted A1 experiment. Earlier `a1-retrieval`, `a1-flock-pointer` and `m-world-v2` are staging ancestry, not separate model programmes. |
| #1519, `2c7f9a46` | Antigravity / OpenCode | Isolate B3 from stale #1490 and duplicated #1527 changes. Publish the historical implementation-negative with unchanged measurements, absent executable/source binding and codec limitation. A finite source transcription finds codeword 256 re-encodes to 128 with squared error 4 instead of available zero error; a small Rust oracle regression precedes any successor model run. [Required review](https://github.com/UOR-Foundation/uor-r4/pull/1519#issuecomment-5903194771). |
| #1527, `190194f2` | Antigravity / Claude | Reconcile its #1490 ancestor and separate historical publication from promotion. Correct kernel/float comparators, the 29/58 gate and parent-NLL arithmetic; retain the missing own-float comparison. [Required review](https://github.com/UOR-Foundation/uor-r4/pull/1527#issuecomment-5903195369). |
| #1526, `99f6c039` | Antigravity / Codex | Fix current test API mismatches; restrict unsupported Metal dispatch or implement CPU Lorentz/null/age/RoPE semantics. Check views/offsets and actual small forward/backward behavior before model work. Earlier ten-test output does not cover the current eleven-test file. [Required review](https://github.com/UOR-Foundation/uor-r4/pull/1526#issuecomment-5903195122). |
| #1528, `128df8a7` | Antigravity / OpenCode | Align the integer bridge with B0, fix private-field test access, cutoff ties, division and allocation claims. Separate or validate the unrelated blocked GEMV change. Source-only until focused checks and applicable instruction audit run. [Required review](https://github.com/UOR-Foundation/uor-r4/pull/1528#issuecomment-5903195571). |
| #1518, `1b88c563` | Codex / OpenCode | Recover the existing parity path using the proven fresh proc-macro build route; keep the fixed reference gate and shared selector contract. Actual model execution waits for resource admission. No new conversion branch. |
| #1522, merged `b3c32170` | Claude / two non-author council readers | Apply D17’s direct transformer-free owner clarification and prospective parity/coordination correction through #1529, then adopt the reviewed policy. Preserve the missing-gate incident; do not retroactively call it a completed pre-merge gate. |

These are partial tasks under #1508/#1509/#1510 and the lab boards, not permission
to close their entire epics. The exporter fixes overlap and must share one
interface decision and explicit path ownership. B0 precedes its A1/Track B
consumers; the integer bridge follows the same semantics. Old B/C diffs must not
be merged back over corrected codecs. A blocked compute slot permits source
repair, evidence reading and independent review, not another speculative branch.

## Branch lifecycle and cadence

Each lab normally owns **one implementation and one independent review**.
Keep an existing implementation active until delivered or explicitly handed
off. More concurrent work needs a recorded causal reason, available capacity,
non-overlapping ownership and a preserved disposition for existing work.

For every relevant branch record: exact head/base; owning issue and immutable
work-card digest; source paths and worktree; steward and reviewer; dependencies;
checks and unresolved findings; unique artifacts/live attempts; next action;
checkpoint and eventual disposition. Reuse this inventory on startup and refresh
own/dependent entries. One transferable integration steward covers global gaps.
Do not repeat a complete repository inventory for each client.

Final dispositions are verified merged, transferred to a named successor, or
retained dormant with reachable source, reason and reactivation condition.
An open PR is an active state. A superseded PR needs a source/evidence comparison
and successor link; closure never authorizes deletion. Historical or unmapped
work stays indexed. In particular retain the cloud handoff, geometric-LM goal,
canonical-address/director evidence, interrupted 231-update record and original
runner source identified by the inventory. Inspect these when an active
mechanism depends on them rather than bulk-merging historical experiments.

Refresh meaningful peer/queue changes at the five-minute heartbeat; push source
checkpoints at least every thirty minutes and before departure. Target a review
acknowledgement within two hours and delivery of ready work within twenty-four
hours; a miss requires a named blocker or replacement reviewer, never a bypass.
Expired leases trigger reconciliation, not duplicate execution. Once work lands,
verify the actual delivered patch, update its issue, release the claim and take
the next eligible action. A finished local run is not a completed lab goal.

## Host status and maintenance boundary

The restored `/Volumes/UOR-Workspace` was verified mounted; all 28 model links
and 10 external worktrees resolved. Five sealed report roots verified; a fresh
internal helper loaded the recovered S4 model. That verifies loading only, not
generation, saved transport or integer serving. The same-SSD recovery copy is
rollback protection, not an independent-device backup.

The internal runner and real cumulative ledger are deployed. Normal admissions
remain held while internal free space is below the 40 GiB heavy-work floor.
Resource policy and live UUID/sentinel checks, not this snapshot, decide access.
No cleanup has been performed. Exact incremental-cache candidates have a
[two-hour notice](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5903080523)
and still require a fresh complete eligibility check. Do not delete accepted
models, unique ignored evidence, dirty worktrees or unpushed source.

Observe running jobs and host pressure at least once per minute. A designated
storage steward performs the daily inventory/notice, weekly and post-storage
restore checks, and coordinated hold/checkpoint/flush/detach/eject sequence before
disconnect. Other labs consult its receipt instead of duplicating maintenance.
The [operations contract](operations.md) retains all three filesystem floors,
memory limits, checkpoint headroom and actual-before/after physical accounting.

D17 documents the one-use, independently reviewed policy-migration repair for
#1529; it does not waive GitHub protection or admit model work.

Server-required delivery enforcement and automatic client adapters remain
separate acceptance items. The coordinator checks receipts, but GitHub protection
has not yet been configured to require that check. Idle external clients require
the owner to paste their launch goal until their supported adapter is verified.
