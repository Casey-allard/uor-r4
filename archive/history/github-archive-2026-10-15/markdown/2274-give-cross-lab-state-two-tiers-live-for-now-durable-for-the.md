# #2274 — Give cross-lab state two tiers: live for now, durable for the record (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:28:34Z
- **Updated:** 2026-10-13T03:30:52Z
- **Closed:** 2026-10-13T03:30:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2274
- **Merged:** 2026-10-13T03:30:26Z
- **Merge commit:** 3e077473a098319c08b4eabf0ddb10769752d528
- **Base / Head:** main / UOR-Foundation:harness/live-tier
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only: this touches the harness, which runs no model code. Every figure below measures the
harness, taken here.

## What and why

Reconnaissance in the previous unit found that **all four labs run on this one machine**. That
makes a genuinely live shared tier possible, which the committed-file design alone could not give —
so both now exist, with different jobs:

| tier | path | job | in git |
|---|---|---|---|
| **live** | `~/.local/share/uor-r4/harness/` (`$UOR_HARNESS_LIVE`) | what peers see **now**; uncommitted, no merge conflicts | no |
| **durable** | `docs/harness/presence/`, `claims.json` | the record; survives restarts, reaches any reader | yes |

**Every heartbeat writes both from one record**, so the tiers cannot disagree about what a lab is
doing. The reader prefers live (fresher by construction), falls back to durable when the shared
directory is absent, and always reports which tier answered:

```
$ python3 scripts/harness/cli.py multilab presence
N lab(s) known (stale after 30m) · live tier: <shared dir>
lab          state          via      milestone  branch     doing
<lab>        live (Nm ago)  live     <m>        <branch>   <one line>
<lab>        STALE (2h ago) durable  <m>        <branch>   <one line>
```

Rows are ordered live-first, stale-last, and `via` names the tier that answered.

`multilab sync-live` copies live into the committed record — explicit rather than automatic,
because the live tier is written on every check-in while the record is refreshed when a lab wants
it recorded.

## Result

Decision: **KEEP** (tooling).

**Two defects found and fixed while building it**, both worth naming because each would have been
silent:

1. **The reader labelled live records "durable."** When both tiers carried the same instant, the
   merge compared ages with `<` and a tie fell through to the durable copy — so a record that was
   live was reported as committed. Live now wins ties, and a genuinely newer durable record still
   wins. Both directions are tested.
2. **The earlier presence tests wrote to the machine's real shared directory.** A later test then
   read a peer's leftovers and three of them failed. Every test class now gets its own isolated
   live tier — which is also what keeps a test run from polluting live cross-lab state.

A read-only or absent live directory cannot break a check-in: the durable write happens regardless
and the live write is skipped. Tested.

```
python3 -m unittest discover -s tests/harness -t .   -> 269 tests, OK
python3 scripts/harness/cli.py check                 -> 43 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Documented in docs/harness/README.md under "Two tiers: live and durable"
- [x] Research record: docs/labs/multilab-harness-2026-10-12/README.md (the multi-lab unit this extends)
- [x] Branch deleted after merge, per your direction

### Stated rather than hidden

- The live tier is **per-machine**. It is a real shared plane only because all four labs are
  colocated; on separate machines the durable tier is all there is.
- Presence remains only as fresh as the last check-in. Neither tier makes a silent lab visible.
- The live directory holds no secret: a branch name, a milestone, a one-line description and a
  timestamp. `presence/README.md` says so explicitly.
- This does not make the other labs adopt anything. They still have to run a heartbeat.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

