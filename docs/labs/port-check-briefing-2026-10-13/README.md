# D30 check-port 8 of 10: the `briefing` rules

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 8 of 10

## All three rules, exact agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `briefing.sources_exist` | warn | all pass | 0 findings |
| `briefing.milestone_numbers_agree` | warn | all pass | 0 findings |
| `briefing.lines_agree_with_prompts` | warn | all pass | 0 findings |

Coverage reads **27 of 40 rules** (arena 4, briefing 3, budget 2, harness 3, mistakes 2, multilab 6,
prep 5, status 2).

## Why these are agreement checks, and why that matters

**The registry and `docs/labs/session-goal.md` are two descriptions of one plan.** A lab reads the
prose at session start; the harness reads the registry. When they disagree, **the lab works the number
in the prose and `check` reports the number in the registry — and both are right, which is the
failure.** A mismatch is not a typo to fix later; it is **two authorities.**

## The subtlety `lines_agree_with_prompts` carries

**The contract also shows the PR-line *format*** — `Line: <name> · count n/3 · headline …` — which
contains a count and **must not be read as a declaration.** A regex matching any `count n/m` would
read the format example as a real line and report a mismatch against a line that does not exist, which
is worse than not checking.

So a declaration requires **`Line` followed by a quoted name**, straight or curly. The format example
has a colon, so it is skipped. **That case is a test**, because it is the one a naive implementation
gets wrong.

## A panic my own probe caught, and the class of bug it removed

My first parser **walked a byte index and advanced it by one**, then sliced `text[i..]`. That panics
the moment the index lands inside a multi-byte character — and this file is full of them (curly quotes,
the `·` separator):

```
start byte index 61 is not a char boundary; it is inside '·' (bytes 60..62 of string)
```

**Rewritten on `char_indices()`, which cannot produce a non-boundary offset.** The class of bug is gone
rather than guarded against — and **I found it by probing, after the test run had already failed once
for reasons I had guessed wrong.**

The probe also showed the fix works on real text: `Line "evaluation integrity" count 1/3` parses
correctly out of the actual session-goal file.

## Verification

| check | result |
|---|---|
| Rust vs Python, all three rules | **identical — all pass in both** |
| the real session-goal parses | 1 declared line read; the format example correctly skipped |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | **0 findings** |
| `cargo test -p uor-r4-harness` | **285 passed** (9 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **27 implemented** |

## Limitations, stated

- **`check` still delegates.** 8 of 10 modules.
- **`declared_lines` reads the session-goal file only**, and only the `Line "<name>"` form. The Python
  also matches a second form (a `DIRECTION:` prefix on a lab prompt) which **this port does not
  implement** — so a line declared only in a `DIRECTION:` block would be **invisible to the Rust rule
  while Python sees it.** The real file currently declares one line in the supported form, so the counts
  agree today; **they would diverge on a file using the other form.** This is the most significant gap
  in this unit and it is stated rather than left for someone to discover.
- **`milestone_numbers_agree` matches `current/of` as a substring**, so a session-goal mentioning
  `37/232` anywhere satisfies it — even in a sentence about something else. Python does the same; the
  looseness is deliberate (a stricter match would fail on wording changes that do not change the
  number), but it means the rule can be satisfied by an unrelated mention.
- **`sources_exist` checks `docs/mechanisms/<id>.md` and the panel registry**; it does not check that
  the brief's *contents* match the mechanism it names.
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule finding counts.
