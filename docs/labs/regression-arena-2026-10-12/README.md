# The regression arena, and the harness as the session entry point

**Question.** Two gaps from the agreed follow-ups. First: the project has frozen panels and a
grading path, but nothing recording what each evaluated configuration actually scored, against
which parent, at what scope — so a capability already won could be lost without anyone noticing.
Second: the harness only helps a session that already knows to use it.

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).
No model compute was run, and none is admitted to any of this (D24/D25).

## 1. The arena is a regression check, not a leaderboard

[`docs/harness/ARENA.md`](../../harness/ARENA.md) and
[`scripts/harness/arena.py`](../../../scripts/harness/arena.py). Five real rows are recorded,
each citing the merged record it came from: the D10 base, the addressed read, the starling tree,
the starling readout self-distillation, and the M2 whole-path source reference (recorded as
`NOT_RUN`, because an empty lineage would be less honest than a row that says so).

```sh
python3 scripts/harness/cli.py arena board            # every row and its parent
python3 scripts/harness/cli.py arena compare <id>     # row by row against the parent
python3 scripts/harness/cli.py arena lineage <id>     # the chain a candidate descends from
```

It holds no transformer or dense row **by design** — D23/D25 forbid those comparators and the
owner's 3 October direction withdrew them — so it cannot answer "is this better than a
transformer", and does not try. The danger this addresses is the opposite one: silently losing a
capability that was already won, which is exactly what the historical pitfalls warn about when
they say to compare candidates row by row against each prior artifact and not against an aggregate.

### The bug this unit found in its own instrument

The first version compared raw numbers and called the larger one a gain. Run against the real
records it printed:

```
gained    candidates_per_query    36 -> 69   (+33)
```

The starling readout's move from 36 to 69 candidates scored per query — a **92% increase in the
work each query does** — was reported as an improvement. An instrument that congratulates a
regression is worse than no instrument, because it will be believed.

So direction is now data in `arena.json`, declared per bar with its unit, and the same comparison
reads:

```
gained    recall                 0.329 -> 0.377  (+0.048)  higher_is_better mqar recall
lost      candidates_per_query   36 -> 69        (+33)     lower_is_better candidates scored per query
```

Three cases stay distinct and are never merged: **better**, **worse** (with the statement that a
gain elsewhere does not offset it), and **unchanged or unknown** — where an undeclared bar is
reported as *neither*, with a note, rather than guessed. That last path is itself tested.

What it refuses to do is as much of the design as what it does: no cross-panel comparison, no
aggregate score anywhere, no invented delta for a bar one side does not report (`UNAVAILABLE`, never
zero), and no action — it does not choose a candidate, retire a mechanism, or move a D21 count.

## 2. The harness is now the session entry point

The [session contract](../../labs/session-goal.md) opens with a **Start here: the harness** section:
the three read-only commands to run before the first status card, the three the contract already
requires (`prep new` + FIT, `heartbeat`, `lint check`/`check`), and the visibility habit — `skill
use`, `team dispatch`, `team claim`, `mistakes capture` — with the point stated plainly: work done
without a receipt is invisible to the owner.

That is a documentation change in the one place every session reads, rather than a new mechanism.
The clause structure and the headline numbers are untouched, and the harness's own
`briefing.lines_agree_with_prompts` rule still parses the page and passes.

## Evidence

```
python3 -m unittest discover -s tests/harness -t .   ->  245 tests, OK (18 new arena tests)
python3 scripts/harness/cli.py check                 ->  43 rules, 0 error, 11 warn
python3 scripts/check_claim_wording.py               ->  passed
python3 scripts/harness/cli.py check --only arena    ->  4 rules, all pass
```

The arena tests cover the direction bug specifically: a cost increase must be a regression, an
equal value is neither, an undeclared direction is flagged rather than guessed, a missing bar is
`UNAVAILABLE`, and a KEEP with no transfer evidence is blocked — with a separate test proving the
phrase "transfer not measured" does not satisfy that requirement.

## Limitations

- Five rows is a start, not a history. The arena only knows what someone records; it cannot
  retroactively reconstruct scores that were never written down at a comparable scope.
- Rows are `provisional` or `scoped-reading` today. `arena.decisive_power` blocks a `decisive`
  claim that lacks ≥ 2 seeds or ≥ 200 fresh rows, but no row currently qualifies.
- `base-d10`'s wrong-value baseline (47) was inferred from the addressed-read record's own delta
  rather than read from a single table; it is recorded so the comparison is possible, and the
  record is cited.
- The session-goal change reaches labs that read the page. It does not install anything, and a lab
  that ignores it is unaffected — the receipts simply stay empty for that work.

**Next:** `prep new` could offer to append an arena row when a sheet is scored, which would close
the loop between pre-registration and lineage; and the two labs still holding worktrees with
uncommitted files should commit and switch into the single checkout.
