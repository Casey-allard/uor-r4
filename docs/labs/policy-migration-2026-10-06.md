# Policy migration record — coordination store policy pointer

**Status:** proposed · **Date:** 2026-10-06 · **Lab:** OpenCode-DeepSeek
**Action:** `AdoptPolicy` — `previous_policy_sha` → `policy_sha`
**Issue:** #820

## What is being migrated

| | |
|---|---|
| `previous_policy_sha` | `ba266ad44f21c9df2dafa9d56a1e2051c1565d91` — *"Restore lab integration ownership and reconcile transformer-free policy delivery"* (#1529) |
| `policy_sha` (proposed) | `e445ee94657311ae158edfab292f2c58cd4f5b24` — *"Shared GPU pod protocol and uor-pod tool"* (#1758), 2026-10-05 21:13 |

The store is pinned **215 commits behind `main`**.

## Why this is required, and why it is blocking

`Action::Claim` calls `require_current_policy` (`tools/lab-runner/src/coord.rs:1398-1401`), which validates
`compare/<policy_sha>...main` through `validate_policy_comparison` (`:226-250`). The guard rejects when the
comparison lists **≥300 files** *or* touches any guarded path (`AGENTS.md`,
`docs/integration/DECISIONS.md`, `docs/integration/agent-execution-policy.*`, `docs/labs/protocol.md`,
`docs/labs/operations.md`).

Measured for the current pin:

```
compare/ba266ad4...main  ->  status=ahead  files=300  guarded=AGENTS.md  ->  FAIL
error: shared policy changed or comparison incomplete; hold admissions and
       perform reviewed policy migration
```

**Every `Claim` from every lab is refused.** This is not degradation; claims are the entry point for all
work, so coordination is stopped.

## Why `e445ee94` is the correct target

It is **the last commit touching any guarded policy file**, so it is the revision at which the current
policy took effect:

```
git log -1 --format=%H origin/main -- AGENTS.md docs/integration/DECISIONS.md \
  docs/integration/agent-execution-policy.json docs/integration/agent-execution-policy.md \
  docs/labs/protocol.md docs/labs/operations.md
-> e445ee94657311ae158edfab292f2c58cd4f5b24
```

Measured against the guard:

| candidate | status | files | guarded | verdict |
|---|---|---|---|---|
| `ba266ad4` (current pin) | ahead | 300 | `AGENTS.md` | **FAIL** |
| `86bec270` (D20 itself) | ahead | 67 | `AGENTS.md` | **FAIL** |
| `12f5caa5` | ahead | 16 | none | PASS |
| `2b21c9a3` | ahead | 15 | none | PASS |
| **`e445ee94`** | ahead | **49** | **none** | **PASS** |
| `d2cae5ba` (then-head) | ahead | 5 | none | PASS |

**Note `86bec270` fails.** D20 — *"geometry stays first; suspend the no-loss rule only after exhaustion"* —
changed `docs/integration/DECISIONS.md`, and `AGENTS.md` changed *after* it, so adopting D20's own commit
would not clear the guard. The correct target is the newest guarded-file commit, not the newest decision.

## Preconditions for `AdoptPolicy` (`coord.rs:410-430`)

1. `previous_policy_sha` equals the store's current `policy_sha`, and differs from `policy_sha`. ✓
2. No task is `claimed` with an unexpired lease. ✓ — measured: **0 live claims**; 5 tasks sit `claimed`
   but expired **113–141 hours** ago.
3. **No attempt is non-finalized.** ✗ — **ONE blocks:** `opencode-b0-flock-g1a2-20260930-g2`
   (issue 1512, session `opencode-deepseek-20260930`, epoch 6, `phase: reserved`). It was reserved against
   `/Volumes/UOR-Workspace`, destroyed with the drive; the job was **cancelled in the queue and never
   started**.
4. A `decision_receipt: FileEvidence` (this record, path + sha256).

## The deadlock, stated exactly

```
adopt policy      -> blocked by the unreconciled attempt
finalize attempt  -> Action::FinishAttempt requires owned(): a live, unexpired, same-session claim
claim task 1512   -> requires require_current_policy, which fails
                  -> CYCLE. No self-recovery path.
```

The existing reconciliation paths cannot reach this attempt either:

- `reconciliation.json` (`uor-r4.stopped-reconciliation/1`) needs `evidence_kind` of
  `pre_spawn_failure` (requires a runner-produced `preflight-failure.json`) or `owned_process` (requires
  `process.json`). **A queued cancel produces neither.**
- The durable `spec.json` does not match the reserved `spec_sha256`
  (`4147ddc92f...8d031`); `verify_exit_evidence` rejects with *"durable job spec differs from reserved
  spec"*. **Confirmed by applying a real `FinishAttempt`** and observing exactly that error.

So the attempt is an **orphan**: its evidence is positive and honest
(`process_state: never_started`, `outcome: cancelled`, `exit_status: null`, `elapsed_ms: 0`,
`cancel.json` present, not queued, not running, no `attempt.json`/`process.json`) but no path accepts it.

## What this record authorises, and what it does not

**Authorises:** migrating the store's policy pointer from `ba266ad4` to `e445ee94`, once the blocking
attempt is reconciled.

**Does not authorise:** altering any decision, watermark, contract or mechanism. This is a pointer
migration to the revision at which current policy already took effect. **No policy content changes.**

## Companion requirement

The attempt must be reconciled first. An opt-in `Action::Abandon` — finalizing a `reserved` attempt whose
exit evidence positively proves `never_started`, requiring the reserving session and **no** live claim —
is in preparation with tests. `FinishAttempt` is deliberately **not** weakened for live work.

## Two further gaps found alongside this

1. **`admissions-held.json` has no removal path** (8+ writes, zero removals; contrast
   `admission-blocked.json`, cleared at `daemon.rs:1028`). A transient condition therefore latches the
   daemon off permanently. That was live: a free-space hold at 39.89 GiB against a 40.12 GiB requirement,
   where 40 GiB is the policy's **warning** level and the **stop** level is 25 GiB. Fixed for this host
   and in the checker (#1787); making a re-evaluable hold self-clear is in preparation.
2. **DONE dirs can go unreconciled without blocking, and therefore unnoticed.** Measured: of 13 DONE
   dirs, **three** have exit evidence never finalized in `coord` — `claude-1541-p1-e4-a1` and
   `opencode-b0-flock-unit-20260930-a` are **absent from `coord` entirely**, and one is **another lab's**.
   Only tracked non-finalized attempts block, so these are silent. Jobs produce durable evidence in
   `done/` and nothing guarantees the coordinator is told.
