# #2408 — Record body_text and raw, unblocking the registry-dependent prlint gates (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T01:23:52Z
- **Updated:** 2026-10-14T01:26:18Z
- **Closed:** 2026-10-14T01:25:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2408
- **Merged:** 2026-10-14T01:25:54Z
- **Merge commit:** bb179c28866336ae3ef157f2ef76d561f7996876
- **Base / Head:** main / UOR-Foundation:deepseek/body-text
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`body_text` and `raw`: the two fields that looked like internals until a gate needed them.** Tooling only.

Decision: **KEEP** (tooling).

## The blocker, found by measuring before writing

Before starting the registry-dependent gates I measured their closure instead of assuming it, and found **`_find_milestone` scores candidate milestones partly on `body_text`**:

```python
if milestone.id.lower() in body.lower():
    score += 1
```

**My Rust `Fields` did not carry it** — it held the 25 fields the gates read, and `body_text` looked like a parser internal. **Without it the milestone the body is *about* cannot be identified, so `count_matches_registry`, `headline_vs_registry` and `pivot_at_three` cannot be ported at all.**

**So the unit became "remove the blocker" rather than "port the gates badly."**

## Fixed

`Fields` carries **`body_text`** and **`raw`**, and `to_json` emits both — **because a gate consumes one of them, so it is part of the contract rather than an internal.**

**The fixtures now record them too: 27 fields × 5 cases = 135 comparisons**, up from 125, all matching Python. **The capture had to be extended, not the assertion relaxed** — the fixture set was complete *for the fields the ported gates were reading*, **which is exactly the kind of completeness that stops being true the moment the next consumer arrives.**

## Why the fixtures recorded the wrong thing before

**They recorded the 25 fields that seemed to matter** — captured by reading what the gates fed on, **not by recording everything the parser returns.** **That is the same mistake this port has made three times about "the registered set": taking the first list I found and stopping.**

**Here it cost one unit of delay and no wrong code**, because I measured the closure **before** writing the gates rather than after. **That is the difference the previous three did not have.**

```
cargo test -p uor-r4-harness                    -> 333 passed
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/body-text-unblocks-milestone-scoring-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **This port is two fields, not a gate. No new gate is reachable, `lint` still delegates, and coverage stays at 36** — it unblocks the next unit rather than advancing it.
- **The registry-dependent gates are not started, and their closure is larger than the blocker alone.** `_find_milestone` is **49 lines** and also needs `_registry_headline`, `_same_number`, `_norm`, `_find_line`, and the milestone `lines`/`headline` structures. **Estimating it as "now unblocked, therefore small" would repeat the mistake this unit was about.**
- **`clippy` caught a field-assignment style issue** and `cargo` was again the check that worked — **the third time this session the compiler found what my own reading did not.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

