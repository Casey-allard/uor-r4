# #1784 — tools(lab-host): heartbeat automation and host-policy drift guard

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:22:49Z
- **Updated:** 2026-10-09T13:20:47Z
- **Closed:** 2026-10-06T04:23:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1784
- **Merged:** 2026-10-06T04:23:53Z
- **Merge commit:** 72b02b7f62d16811aeead6ce5f0a847a7fba9538
- **Base / Head:** main / UOR-Foundation:deepseek/lab-host-tools
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two host-local operational tools plus a README. Not part of the Rust workspace, not built by CI, no production code touched.

## Why these exist — both failures were invisible for days

**1. Labs expire silently.** `tools/lab-runner/src/coord.rs:14` sets `TTL = 1200` — **twenty minutes** — and `:492` expires a lab whose heartbeat is stale. **No MANUAL client can renew inside that window**, and every lab adapter says so in its own words (`"MANUAL client; no heartbeat automation"`).

Measured 2026-10-06 with the guard below — **four of five labs had EXPIRED heartbeats**, stale by 3.7 to 5.9 days, while every lab believed it was coordinating:

```
✗ antigravity-lab-20260930:     EXPIRED 8550m ago (~5.9d)
✗ claude-lab-20260930:          EXPIRED 5365m ago (~3.7d)
✗ codex-durable-20260929-root:  EXPIRED 7684m ago (~5.3d)
✗ kimi-coordinator-20260930:    EXPIRED 8365m ago (~5.8d)
✓ opencode-deepseek-20260930:   19m remaining  (after installing this automation)
```

**2. Local policy drifts unobserved.** `runner/host-policy.json` still declared two volumes destroyed when a drive failed and the workflow moved to iCloud, and `admissions-held.json` was holding admission on a **stale** free-space measurement — claiming 25.5 GiB free against a 40.0 GiB reserve while the machine measured **45.3 GiB**. Both were findable only by reading three files nothing compared.

## `lab-heartbeat <session> [coord.git]`

Renews one lab's heartbeat. Unique event id per beat, bounded log, non-zero exit on failure. Install as a LaunchAgent with `StartInterval` **well under TTL** — the template uses 600s for a 2× margin, so one missed run (sleep, network blip) does not expire the lab. **Verified firing: sequence 896 → 899, `LastExitStatus = 0`.**

## `host-policy-check`

Read-only, exits **1** on drift, so it can gate a session start. Compares what nothing was comparing: local policy age; whether every declared volume **actually exists**; free space against each declared reserve; repo watermarks against the local reserve; **whether an admission hold is stale** (re-evaluating the hold's own claim, since a hold is written once and never re-read); and **every lab heartbeat against the TTL**.

## Two disagreements reported, not decided

- **The repo policy declares a three-volume topology that no longer exists** — watermarks for `inner_workspace` (120/60/30 GiB) and `outer_ssd` (240/120/60 GiB) with no local `workspace`/`backing` volume. Whether to drop them or retain them for hosts that still have the hardware is an **owner/council call**, not a lab one.
- **Stale admission holds do not self-clear.** Whether a hold should be re-measured on read, or expire, is unresolved.

References #820, #1747.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

