# #1630 — Bound the dose run's wall time so memory pressure cannot hang it

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:44:26Z
- **Updated:** 2026-10-02T17:46:27Z
- **Closed:** 2026-10-02T17:46:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1630
- **Merged:** 2026-10-02T17:46:27Z
- **Merge commit:** c5830bd4d094466ebfb778c9269d1077284a8c51
- **Base / Head:** main / UOR-Foundation:codex/post-epoch-eval-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The dose launcher set **no `max_seconds`**, so both phases would run until finished or forever. That is a real risk right now, measured while queueing:

- **swap at 8.7 GiB of 10.2 GiB, 1.5 GiB free**
- a **1.8 GiB `llama-server`** (an offline teacher) resident at **0% CPU**
- the queueing job (a peer lab's 16,000-step rung) already at 1.5 GiB

Under that pressure an unbounded run can **thrash rather than progress**, and the difference between *slow* and *stuck forever* is invisible from the outside. A bounded run stops, checkpoints, and reports.

## Change

| phase | bound | projection | slack |
|---|---:|---:|---|
| LM | **43,200 s (12 h)** | 4.7–7.1 h | ~1.7× |
| response | **10,800 s (3 h)** | ~0.5 h | ~6× |

Both overridable by environment variable (`LM_MAX_SECONDS`, `DIALOGUE_MAX_SECONDS`). Bounds carry real slack, so they fire only on genuine pathology and not on a slow-but-healthy run. **Checkpoints every 2,518 and 512 updates mean a bounded stop preserves the work rather than losing it.**

This also matches the project's own convention for long runs — #820's rung used `max_seconds 43200`.

## Exact-head evidence

**Reviewed head:** `bc060b67`. `bash -n` clean.

```
32:  checkpoint_every=2518 sample_tokens=0 max_seconds="${LM_MAX_SECONDS:-43200}"
41:  max_seconds="${DIALOGUE_MAX_SECONDS:-10800}"
```

The running waiter executes this file directly, so the bound takes effect on the queued launch with no further action.

## Scope

Orchestration only; **no model behaviour change and no capability claim.** The result this run produces will be reported separately and labelled **geometry-on-principle, not a demonstrated advantage**.

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

