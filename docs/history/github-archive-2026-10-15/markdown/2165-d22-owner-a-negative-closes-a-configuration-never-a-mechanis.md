# #2165 — D22 (owner): a negative closes a configuration, never a mechanism; five closures reopened (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T15:55:59Z
- **Updated:** 2026-10-10T17:52:07Z
- **Closed:** 2026-10-10T17:51:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2165
- **Merged:** 2026-10-10T17:51:55Z
- **Merge commit:** 864732ae26912f233cd9217e7b6a51372a4d24f0
- **Base / Head:** main / UOR-Foundation:claude/d22
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: lab governance (owner decision) · count n/a · headline unchanged

## What and why

The owner asked whether the labs were using D21's three-negatives rule to rule out novel mechanisms on weak tests. An audit of every closure since D21 found honest, scoped records but unfit tests:
- **Codex constructor:** never scored on the panel; stopped on LU-pivot numerics.
- **Codex native Context/prototype learning:** judged with a loss that later proved faulty.
- **DeepSeek identity pointer:** keyed on the wrong context (hit rate 0.003), and the line was archived on it.
- **DeepSeek read binding:** met its own target and was still counted as a negative.
- **DeepSeek 10 % dose:** doubled memory and was rejected by one row.
- **Claude learned rank table:** a 54 % gain was prescribed a stop.
- **What was kept:** it leaned to ordinary levers, for example 437/512 on the development panel and 0/128 fresh.

**D22**, in DECISIONS.md, with pointers in AGENTS.md and the session-goal contract:
- **§1:** a negative closes a configuration, never a mechanism. Stopping a novel mechanism needs the owner. A tooling failure is a blocker, not a negative.
- **§2:** a run that meets its own mechanism target without a significant headline loss is KEEP and iterate, and resets the count.
- **§3:** bars are never narrower than the panel's noise. Decisive runs use ≥ 2 seeds or ≥ 200 fresh rows, and train the mechanism in rather than bolting it on. Ordinary levers cannot be a novel line's decisive run.
- **§4:** every development KEEP also reports fresh transfer.
- **§5:** reopens the Codex constructor (reversing D21 §3) and joint Context + prototype learning; DeepSeek's addressed memory, the identity pointer with a real key, and powered read binding; and adopts the 10 % dose as M1's base.

## Checklist
- [x] DECISIONS.md, AGENTS.md, session-goal.md and current-state entry with **Next:**
- [x] `check_claim_wording.py` passes
- [x] Docs only

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6099378956 — 2026-10-10T15:56:01Z — Casey-allard

Exact-head self-review at `968a646818b944b3fe7c07f88f2813c73e0300ab`: docs only (owner decision D22), based on fresh origin/main; the claim-wording gate passes.

### comment 6099782633 — 2026-10-10T16:38:12Z — Casey-allard

Exact-head self-review at `8bb5d42497ff8858a0c93f03aefe0194ee5b495d`: merged origin/main (the mechanism briefs, #2167). current-state.md keeps both entries; docs only. Queued as soon as the org's Actions slots free up.

### comment 6100227641 — 2026-10-10T17:25:50Z — Casey-allard

Exact-head self-review at `826b4d18b1897cf4226d1acfd431fae2a2b75ee6`: merged origin/main again (current-state.md keeps every entry); docs only, and the claim-wording gate passes.


---
_Comment/review ids in this file: 6099378956, 6099782633, 6100227641_

