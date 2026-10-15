# #1787 — lab-host: a storage WARNING must not act as the admission gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:39:05Z
- **Updated:** 2026-10-09T13:21:39Z
- **Closed:** 2026-10-06T04:40:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1787
- **Merged:** 2026-10-06T04:40:10Z
- **Merge commit:** 7c53c39431f16af21e6abb33301e67c3d0907d9b
- **Base / Head:** main / UOR-Foundation:deepseek/policy-watermarks
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes a live, programme-wide block: **a soft storage goal was acting as a permanent admission gate.**

## The defect

The repo policy already declares three levels per volume class (`agent-execution-policy.json` → `execution.machine_budget.storage_watermarks_gib`):

```
internal: target 60 | warning 40 | stop 25 GiB
```

The local runner `host-policy.json` had set `reserve_bytes = 40 GiB` — **the warning level** — as the **hard** admission gate. `host::check_admission` errors when free space is below `reserve_bytes`, which calls `host::hold()`, which writes `admissions-held.json` — and **nothing anywhere removes that file** (8+ writes, zero removals; contrast `admission-blocked.json`, which is cleared at `daemon.rs:1028`). So a soft goal became a permanent latch.

**It was live:** 39.89 GiB free against a 40.12 GiB requirement, short by **0.23 GiB**, holding admission for every lab. It blocked the very cleanup a lab needs to run to get back above the goal.

## Changes

- Local `host-policy.json` `reserve_bytes` → the policy's **stop** level, **25 GiB** (backup kept as `host-policy.json.bak-reserve40`). The hard gate is the stop level; **40 GiB remains the goal.**
- `host-policy-check` **warns** below the repo's warning level but does **not** fail the run — *WARN ONLY, must not block*.
- Section [4] compares the local reserve against the repo's **stop** level, not its warning. Comparing it to the warning was the original defect, restated inside the checker.
- Section [4]: a repo watermark with **no corresponding local volume is now informational, not drift**. The repo policy is host-agnostic — a watermark is a statement about a *class* of storage, and other hosts may still have that volume. A missing *local* volume is a fact about this machine. This removes two false findings for `inner_workspace` and `outer_ssd`, superseded when the drive failed and the workflow moved to iCloud.

## Verified

Guard reports **6 → 4** conditions. The free-space line now reads:

```
internal: 39.4 GiB free >= 25.0 GiB reserve, but below the 40 GiB soft goal — WARN ONLY, must not block
```

No admission hold present; the runner admits again.

## Still needed, separately

- **Other hosts may have the same mis-set number.** This corrects the checker and this host's policy; a host whose `reserve_bytes` was also a warning level should be re-checked.
- **`admissions-held.json` still has no removal path in the runner.** Making a re-evaluable hold self-clear is in progress.

References #820, #1747.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

