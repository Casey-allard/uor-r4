# #77 — CI Gate C trend tracking with regression alarms

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:36Z
- **Updated:** 2026-07-23T04:10:00Z
- **Closed:** 2026-07-23T03:52:55Z
- **Labels:** graph-compiler
- **Milestone:** Phase 5: Allocation-free graph runtime
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/77
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: 4-way Gate C harness (part of #64 finish) and canonical bits/token. Record the 4-way Gate C table per main commit (small corpus slice for speed) as a CI artifact with a regression threshold (e.g. top-1 drops > 2 points or bits/token worsens > 0.1 vs the pinned previous record → fail or loud annotation). Acceptance: ci.yml uploads the trend JSON; two consecutive commits demonstrate the alarm path on a synthetic regression.


---

## Definition of Done
- CI records the 4-way Gate C table (small corpus slice) per main commit as an artifact, with a regression alarm on a pinned threshold.
- The alarm path is demonstrated end-to-end.

## Acceptance criteria
- [ ] ci.yml uploads the trend JSON per run with commit metadata.
- [ ] A synthetic regression (top-1 drop > 2 points or bits/token +0.1 vs pinned previous) demonstrably triggers the alarm in CI logs.
- [ ] The threshold and the trend artifact location are documented in the workflow header.

## Comments (1)

### comment 5054126110 — 2026-07-23T03:52:54Z — auser

Complete (a5a1e63). DoD status: (1) ci.yml gains the gate-c-trend job: runs scripts/gate_c_trend.sh on every push/PR — the real Gate C measurement on the fixture corpus, compared against the pinned record docs/transformerless/gate_c_pinned.json (maintainer-bumped like the kappa baseline). (2) Alarm thresholds per the pin: top-1 drop > 0.02 abs or bits/token regression > 0.1 abs vs pinned, on EITHER the Rule 1+2 or the baseline row (a baseline regression is itself a signal worth stopping for). (3) Alarm path demonstrated end-to-end: exit 1 on a synthetic regression (pinned top-1 = 0.99 vs measured 0.3171) with the regression named in CI-readable output; exit 0 on the clean run. (4) Trend JSONL (commit sha + timestamp + the four metrics) uploads as a 90-day CI artifact per run for curve plotting; threshold and artifact location documented in the pin file and workflow.


---
_Comment/review ids in this file: 5054126110_

