# #1793 — lab-runner: distinguish recheckable holds from terminal ones so a transient condition cannot latch admission forever

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:57:23Z
- **Updated:** 2026-10-09T13:19:01Z
- **Closed:** 2026-10-06T04:58:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1793
- **Merged:** 2026-10-06T04:58:27Z
- **Merge commit:** c2414f91eaa9f58553f6ddbf700256929726f47a
- **Base / Head:** main / UOR-Foundation:deepseek/admission-latch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes a live bug where **a transient condition could latch admission off permanently** — and the runner-side counterpart to #1787.

## The bug

`host::hold()` writes `admissions-held.json`; **nothing anywhere removes it** (8+ writes, zero removals — contrast `admission-blocked.json`, cleared at `daemon.rs:1028`). The daemon gates on **existence only** (`daemon.rs:954`, `:599`). So a condition that is a pure function of current machine state — free space — writes a permanent latch. It was live: a hold at **39.89 GiB against a 40.12 GiB requirement**, where 40 GiB is the policy's **warning** and the **stop** is 25 GiB, blocking the very cleanup needed to get back above the goal.

## What it does

- `HostFailure { terminal(reason) | recheckable(check: HoldCheck, reason) }` — the classification lives in the **data**, not in string-matching a reason.
- `HoldCheck` — the re-evaluable predicate, so re-evaluation runs **the same function** that produced the hold, not a re-implementation (`check_volume_reserve` is factored out of `check_storage`, which is now built from it).
- The shortfall is a **typed** `RunnerError::VolumeReserve`, never string-inferred; its `Display` is **byte-identical** to the old message, so operator logs and receipts are unchanged.
- `HoldRecord` persists `kind` + `check`. `admit_pending` loads it: **Terminal → return early, never cleared**; **Recheckable → re-run the recorded predicate**, and on pass remove the file and **fall through to the ordinary admission path**, which still re-validates every candidate — clearing the latch skips nothing. A failed removal just defers to the next tick.
- `:599` now blocks only **Terminal** holds.

## All 13 `host::hold` sites explicitly classified

Kind is decided **at every call site** — no default, nothing inferred from the reason. Ten are Terminal (unrecoverable exit status, contradictory durable evidence, missing identity, policy refusal), two are Recheckable (`:679` iff typed shortfall, `:1319` per predicate — **the site that produced the live latch**), and **one is flagged AMBIGUOUS** (`:1219` launch failure — it records a charged attempt finalized UNKNOWN, and could embed a transient storage error; Terminal was chosen because a wrongly-Recheckable site could release an unreconciled charge). That site is flagged rather than silently classified.

## Tests — 8 added, all pass

`cargo test -p lab-runner --offline` on this rebased head (`bdafd091`) over `c5282a62`:

```
lib 98 | cli 6 | delivery 7 | host_recovery 7 | maintenance 4  =  122 passed, 0 failed
```

The four required cases plus: the `:599` gate, a legacy unclassified record (reads back **Terminal** by design — a guess could release an unreconciled process), the message contract, and call-site classification. Existing terminal-hold tests pass **byte-unchanged**.

## Three honest findings from the author

1. **The live latch came from the monitor site (`:1146`→`:1174`), not input verification** — the live reason is bare, while `:654` prefixes `"input verification admission failed: "`. Arithmetic confirms `43083890688 = 42949672960 + 134217728 + 0` in-flight storage.
2. **This change alone would NOT have released that specific record.** It has no `kind`, so it reads back Terminal. **The manual clear was necessary**, and legacy→Terminal is the safe direction — proven by test.
3. **Two flaky tests are pre-existing, not introduced here.** Proven by running the unmodified baseline at `81de3ca5`: **3 of 6 full-suite runs failed identically** in `unknown_monitor_preserves_paths_child_and_charge_until_stopped_reconciliation` (existence-then-content race on `payload.exit`) and `cancel_finalizes_a_running_job` (`cli.rs:339`). Two of this change's own new tests had races of the same class; both were fixed and 10/10 stable lib runs verified.

References #820, #1747, #1787.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

