# The regression arena

**What it is.** A lineage of evaluated geometric configurations, in
[`arena.json`](arena.json), with a comparison that reports **row by row against a candidate's
parent**. It answers "what did this change break, and what did it fix" on the frozen panels.

**What it is not.** A leaderboard, and not a place to race transformers. D23/D25 forbid softmax
and dense comparators, and the owner's 3 October direction withdrew transformer controls. The
arena contains no such row by design, and it cannot answer "is this better than a transformer" —
it does not try. What the project needs is a regression check, because the danger here is not
being slower than something else: it is silently losing a capability that was already won.

```
uor-harness arena board            # every row and its parent
uor-harness arena compare <id>     # row-by-row against the parent
uor-harness arena lineage <id>     # the chain a candidate descends from
```

## A row

One evaluated configuration. `docs/harness/arena.json`:

```json
{
  "id": "addressed-read",
  "artifact": "product-key memory at the answer layer, trained in from step 0",
  "milestone": "m1",
  "date": "2026-10-11",
  "parent": "base-d10",
  "verdict": "KEEP",
  "confidence": "provisional",
  "mechanism": "exact-addressed-memory",
  "panel": "v5-reply",
  "seeds": 1,
  "rows": 120,
  "scope": "same recipe as the parent; wrong-value failures on the 120-row set",
  "scores": {"wrong_value_failures": 34, "memory": 20},
  "notes": "-90% per-token compute on that layer, but +36.9M params and a 20x training wall",
  "record": "docs/labs/addressed-read-2026-10-11/README.md"
}
```

- **`parent`** is what makes this a lineage rather than a list. It is the configuration the
  candidate was derived from, so the rows form the tree of how the model actually evolved.
- **`confidence`** is the scope of the reading: `decisive`, `provisional`, `scoped-reading`,
  `uninterpretable`. Only `decisive` claims to support a decision, and it must carry ≥ 2 seeds
  or ≥ 200 fresh rows, or the comparison blocks it (D22 §3).
- **`scope`** is required. A number without the artifact, data and steps it was measured at is
  not a result, and the comparison says so instead of comparing it.
- **`record`** points at the dated lab record. A row without one is a number with no evidence.

## Why direction is declared per bar

The first version of this tool compared numbers and labelled the larger one a gain. Against the
real records it reported this:

```
gained    candidates_per_query    36 -> 69   (+33)
```

That is a **92% increase in the work each query does**, reported as an improvement. An instrument
that congratulates a regression is worse than no instrument, because it will be believed. So every
bar declares its direction and unit in `arena.json`:

```json
"bars": {
  "recall":               {"direction": "higher_is_better", "unit": "mqar recall"},
  "candidates_per_query": {"direction": "lower_is_better",  "unit": "candidates scored per query"},
  "bpb":                  {"direction": "lower_is_better",  "unit": "bits per byte"}
}
```

The same comparison now reads:

```
gained    recall                 0.329 -> 0.377  (+0.048)  higher_is_better mqar recall
lost      candidates_per_query   36 -> 69        (+33)     lower_is_better candidates scored per query
```

Three cases are kept distinct, and the distinction matters:

- **better** — a real gain in the declared direction;
- **worse** — a real regression, and `compare` says plainly that a gain elsewhere does not offset it;
- **unchanged or unknown** — a tie, or a bar whose direction nobody declared. An undeclared bar is
  reported as neither a gain nor a regression, with a note. It is never guessed.

## What it refuses to do

- **Compare across panels.** If the parent and candidate were scored on different panels, the
  comparison stops: that is not a comparison.
- **Average the bars.** There is no total score anywhere, by design. A candidate that gains
  0.02 BPB while losing three memory rows is a regression with a headline, and a single aggregate
  is exactly how that gets hidden — the historical pitfalls warn about this directly.
- **Invent a delta.** A bar one side does not report is `UNAVAILABLE`, never zero.
- **Report an uninterpretable row as a result.** `confidence: uninterpretable` sets
  `comparable: false`.
- **Act.** It does not choose a candidate, retire a mechanism, or change a D21 count. Stopping a
  novel mechanism needs an owner decision (D22).

## Adding a row

Add it when a configuration is actually measured, in the same PR as its record:

1. Give it a `parent` — the configuration it was derived from. A new line starts at a root row.
2. Record `scores` only for bars that exist in `bars`, and add the bar's direction if it is new.
3. State the scope: artifact, data split, steps, seeds, rows.
4. Point `record` at the dated `docs/labs/<topic>-<date>/README.md`.

`check` then enforces the rest: `arena.rows_valid`, `arena.parents_exist`,
`arena.decisive_power`, and `arena.knows_what_it_cannot_measure` — the last one fails if the
registry stops stating its own blind spots, because an instrument that does not name its limits
gets read as complete.
