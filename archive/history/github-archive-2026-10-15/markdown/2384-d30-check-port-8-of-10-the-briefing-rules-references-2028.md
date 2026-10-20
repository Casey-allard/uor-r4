# #2384 — D30 check-port 8 of 10: the briefing rules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T21:49:33Z
- **Updated:** 2026-10-13T21:52:01Z
- **Closed:** 2026-10-13T21:51:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2384
- **Merged:** 2026-10-13T21:51:42Z
- **Merge commit:** b814e22064d0f5aa62eb9e591f141cbf93b2fbd9
- **Base / Head:** main / UOR-Foundation:deepseek/briefing-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 8 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## All three rules, exact agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `briefing.sources_exist` | warn | all pass | 0 findings |
| `briefing.milestone_numbers_agree` | warn | all pass | 0 findings |
| `briefing.lines_agree_with_prompts` | warn | all pass | 0 findings |

Coverage reads **27 of 40 rules** (arena 4, briefing 3, budget 2, harness 3, mistakes 2, multilab 6, prep 5, status 2).

## Why these are agreement checks

**The registry and `docs/labs/session-goal.md` are two descriptions of one plan.** A lab reads the prose at session start; the harness reads the registry. When they disagree, **the lab works the number in the prose and `check` reports the number in the registry — and both are right, which is the failure.** A mismatch is not a typo to fix later; it is **two authorities.**

## The subtlety `lines_agree_with_prompts` carries

**The contract also shows the PR-line *format*** — `Line: <name> · count n/3 · headline …` — which contains a count and **must not be read as a declaration.** A matcher accepting any `count n/m` would read the format example as a real line and report a mismatch against a line that does not exist, **which is worse than not checking.**

So a declaration requires `Line` followed by a **quoted name**, straight or curly; the format example has a colon. **Asserted as a test**, because it is the case a naive implementation gets wrong.

## A panic my own probe caught

My first parser **walked a byte index and advanced it by one**, then sliced `text[i..]`. That panics the moment the index lands inside a multi-byte character — and this file is full of them (curly quotes, the `·` separator). The panic named a char-boundary violation at one of those separators; the exact wording is in the record.

**Rewritten on `char_indices()`, which never yields a non-boundary offset — the class of bug is gone rather than guarded against.** I found it **by probing, after the suite had already failed once for reasons I had guessed wrong.** The probe confirmed the fix against real text, parsing a real `Line "<name>" count 1/3` declaration out of the actual session-goal file.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 285 passed (9 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 27 implemented
```

Record: `docs/labs/port-check-briefing-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 8 of 10 modules.
- **`declared_lines` implements only the `Line "<name>"` form, not the Python's second `DIRECTION:`-prefixed form.** A line declared only in a `DIRECTION:` block would be **invisible to the Rust rule while Python sees it.** The counts agree today because the real file uses the supported form — **they would diverge on a file using the other.** **This is the most significant gap in this unit**, and it is stated rather than left to be discovered.
- **`milestone_numbers_agree` matches `current/of` as a substring**, so an unrelated mention of the same number satisfies it. Deliberate looseness: a stricter match would fail on wording changes that do not change the number.
- **`sources_exist` does not check that a brief's *contents* match the mechanism it names.**
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule counts.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

