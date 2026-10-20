# #2267 — The regression arena, and the harness as the session entry point (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:43:48Z
- **Updated:** 2026-10-13T02:46:05Z
- **Closed:** 2026-10-13T02:45:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2267
- **Merged:** 2026-10-13T02:45:43Z
- **Merge commit:** 8cd9efafb613348131d6b6358485e7e1a110bff1
- **Base / Head:** main / UOR-Foundation:harness/arena-and-session-start
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

No model compute runs in this PR. Milestone reference: #2028.

**Every number quoted below is a historical measurement already merged and recorded** — quoted from
its own lab record to explain what the arena does, not produced here. No run is being pre-registered
because no run is being made.

## What and why

Two gaps from the agreed follow-ups.

**1. The regression arena.** The project has frozen panels and a grading path, but nothing recorded
what each evaluated configuration actually scored, against which parent, at what scope — so a
capability already won could be lost without anyone noticing. [`arena.json`](docs/harness/arena.json)
plus [`arena.py`](scripts/harness/arena.py) close that, with five real rows each citing its merged
record:

```sh
python3 scripts/harness/cli.py arena board            # every row and its parent
python3 scripts/harness/cli.py arena compare <id>     # row by row against the parent
python3 scripts/harness/cli.py arena lineage <id>     # the chain a candidate descends from
```

It holds **no transformer or dense row by design** — D23/D25 forbid those comparators and the
owner's 3 October direction withdrew them — so it cannot answer "is this better than a
transformer", and does not try. The danger it addresses is the opposite: silently losing something
already won, which is what the historical pitfalls mean when they say to compare candidates row by
row against each prior artifact and not against an aggregate.

**2. The harness is now the session entry point.** [session-goal.md](docs/labs/session-goal.md)
opens with a *Start here: the harness* section: the three read-only commands to run before the first
status card, the three the contract already requires (`prep new` + FIT, `heartbeat`, `lint
check`/`check`), and the visibility habit (`skill use`, `team dispatch`, `team claim`, `mistakes
capture`), stating plainly that work without a receipt is invisible to the owner. Clause structure
and headline numbers are untouched, and `briefing.lines_agree_with_prompts` still parses the page.

## Result

Decision: **KEEP**. No model result is claimed.

**This unit found a real defect in its own instrument, which is the part worth reading.** The first
version compared raw numbers and called the larger one a gain. Against the real records it printed:

```
gained    candidates_per_query    36 -> 69   (+33)
```

The starling readout's move from 36 to 69 candidates scored per query (both figures from [starling-readout-2026-10-12](docs/labs/starling-readout-2026-10-12/README.md), already merged) — a **92% increase in the
work each query does** — was reported as an improvement. An instrument that congratulates a
regression is worse than none, because it will be believed.

Direction and unit are now declared per bar in `arena.json`, and the same comparison reads:

```
gained    recall                 0.329 -> 0.377  (+0.048)  higher_is_better mqar recall
lost      candidates_per_query   36 -> 69        (+33)     lower_is_better candidates scored per query
```

Three cases stay distinct and are never merged: **better**; **worse** (with the statement that a
gain elsewhere does not offset it); and **unchanged or unknown**, where an undeclared bar is
reported as neither, with a note, rather than guessed.

What it refuses to do is as much of the design as what it does: no cross-panel comparison, no
aggregate score anywhere, no invented delta (`UNAVAILABLE`, never zero), and no action — it does not
choose a candidate, retire a mechanism, or move a D21 count.

```
python3 -m unittest discover -s tests/harness -t .   ->  245 tests, OK (18 new)
python3 scripts/harness/cli.py check                 ->  43 rules, 0 error, 11 warn
python3 scripts/check_claim_wording.py               ->  passed
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] No model compute
- [x] Research record: docs/labs/regression-arena-2026-10-12/README.md
- [x] New entry at the top of docs/integration/current-state.md, with **Next:**
- [x] Every document this change makes stale is updated (harness README, CONTRACT, session-goal)
- [x] STATUS.md / ROADMAP.md / #2028 table: state unchanged — no capability or milestone status moved
- [x] README unchanged: nothing a reader relies on changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [ ] After merge: verify on fresh `origin/main`, delete the branch

### Stated rather than hidden

- Five rows is a start, not a history. The arena only knows what someone records; it cannot
  reconstruct scores that were never written at a comparable scope.
- Every row is `provisional` today. `arena.decisive_power` blocks a `decisive` claim lacking ≥2
  seeds or ≥200 fresh rows, but no row currently qualifies.
- `base-d10`'s wrong-value baseline (47) was inferred from the addressed-read record's own delta
  rather than read from one table; it is recorded so the comparison is possible, and cited.
- The session-goal section reaches labs that read the page. It installs nothing, and a lab that
  ignores it is unaffected — its receipts simply stay empty.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

