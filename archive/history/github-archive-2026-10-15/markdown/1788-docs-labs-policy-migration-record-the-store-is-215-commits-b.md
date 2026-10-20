# #1788 — docs(labs): policy migration record — the store is 215 commits behind and every claim is refused

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:40:24Z
- **Updated:** 2026-10-09T13:21:37Z
- **Closed:** 2026-10-06T04:41:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1788
- **Merged:** 2026-10-06T04:41:28Z
- **Merge commit:** cf7ccaea9ca10aff73314b3356be7ce515948108
- **Base / Head:** main / UOR-Foundation:deepseek/policy-migration-record
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs-only. Adds `docs/labs/policy-migration-2026-10-06.md`.

Records the required `AdoptPolicy` migration, the deadlock blocking it, and two further gaps found alongside. This gives the migration a durable decision artifact and a `decision_receipt` path.

## The migration

| | |
|---|---|
| `previous_policy_sha` | `ba266ad44f21c9df2dafa9d56a1e2051c1565d91` (#1529) |
| `policy_sha` | **`e445ee94657311ae158edfab292f2c58cd4f5b24`** (#1758, 2026-10-05 21:13) |

`Action::Claim` calls `require_current_policy` (`coord.rs:1398-1401`), which validates `compare/<pin>...main` via `validate_policy_comparison` (`:226-250`). The guard rejects at **≥300 files** *or* any guarded path. Measured for the current pin:

```
compare/ba266ad4...main -> status=ahead files=300 guarded=AGENTS.md -> FAIL
```

**Every `Claim` from every lab is refused.** Claims are the entry point for all work, so coordination is stopped — not degraded.

## Why `e445ee94`

It is **the last commit touching any guarded policy file** — the revision at which current policy took effect. Measured: `ahead`, **49 files**, no guarded → **PASS**.

**A trap worth recording: `86bec270` (D20 itself) FAILS.** D20 changed `DECISIONS.md`, and `AGENTS.md` changed *after* it. So adopting the newest *decision* does not clear the guard; the target is the newest guarded-*file* commit.

## The deadlock

```
adopt policy      -> blocked by the unreconciled attempt
finalize attempt  -> FinishAttempt requires owned(): live, unexpired, same-session claim
claim task 1512   -> requires require_current_policy, which fails
                  -> CYCLE. No self-recovery path.
```

The orphan is `opencode-b0-flock-g1a2-20260930-g2` (issue 1512, epoch 6, `reserved`) — **cancelled in the queue and never started**, reserved against the volume destroyed with the drive. Existing reconciliation paths cannot reach it: `reconciliation.json` requires `evidence_kind` of `pre_spawn_failure` (needs a runner-produced `preflight-failure.json`) or `owned_process` (needs `process.json`), and **a queued cancel produces neither**. Its durable `spec.json` also doesn't match the reserved `spec_sha256`, so `verify_exit_evidence` rejects with *"durable job spec differs from reserved spec"* — **confirmed by applying a real `FinishAttempt`** and observing that exact error.

## What this authorises

**A pointer migration only**, once the blocking attempt is reconciled. It alters **no** decision, watermark, contract or mechanism — no policy content changes.

## Two further gaps

1. **`admissions-held.json` has no removal path** (8+ writes, zero removals; contrast `admission-blocked.json`, cleared at `daemon.rs:1028`) — a transient condition latches the daemon off permanently. That was live: 39.89 GiB against 40.12 GiB, where 40 is the policy's **warning** and **stop is 25**. Fixed for this host and in the checker (#1787); self-clearing holds in preparation.
2. **DONE dirs can go unreconciled without blocking, therefore unnoticed.** Of 13 DONE dirs, **three** have exit evidence never finalized in `coord`; two are **absent from `coord` entirely** and one is **another lab's**. Only tracked non-finalized attempts block, so these are silent.

References #820, #1747.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

