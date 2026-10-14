# `body_text` and `raw`: the two fields that looked like internals until a gate needed them

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** unblocking the registry-dependent `prlint` gates

## The blocker I found, and how it was found

Before starting the registry-dependent gates I measured their closure instead of assuming it, and
found **`_find_milestone` scores candidate milestones partly on `body_text`**:

```python
if milestone.id.lower() in body.lower():
    score += 1
```

**My Rust `Fields` did not carry `body_text`** — it held the 25 fields the gates read, and
`body_text` looked like a parser internal. **Without it the milestone the body is *about* cannot be
identified, so `count_matches_registry`, `headline_vs_registry` and `pivot_at_three` cannot be ported
at all.**

**So the unit became "remove the blocker" rather than "port the gates badly."** The gates are the
next unit and they now have their input.

## Fixed

`Fields` carries **`body_text`** (comment-stripped, newline-normalised) and **`raw`** (the body as
given), and `to_json` emits both — **because a gate consumes one of them, so it is part of the
contract rather than an internal.**

**The fixtures now record them too: 27 fields × 5 cases = 135 comparisons**, up from 125, and all
matching Python. **The capture had to be extended, not the assertion relaxed** — the fixture set
was complete for the fields the gates I had ported were reading, **which is exactly the kind of
completeness that stops being true the moment the next consumer arrives.**

## Why the fixtures recorded the wrong thing before

**They recorded the 25 fields that seemed to matter.** They were captured by reading what the gates
fed on — **not by recording everything the parser returns.** That is the same mistake this port has
made three times about "the registered set": **taking the first list I found and stopping.**

**Here it cost one unit of delay and no wrong code**, because I measured the closure before writing
the gates rather than after. **That is the difference the previous three did not have.**

## Verification

| check | result |
|---|---|
| **135 of 135 fields match Python** (27 × 5, up from 125) | asserted per field |
| `body_text` differs from the body where comments were stripped | captured, and asserted by the fixture |
| `cargo test -p uor-r4-harness` | **333 passed** |
| `cargo fmt --check`, clippy | clean, **0 findings** (a field-assignment lint caught one first) |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **This port is two fields, not a gate.** **No new gate is reachable and `lint` still delegates.**
  The unit unblocks the next one; it does not advance coverage, and the count stays at **36**.
- **The registry-dependent gates are not started**, and their closure is larger than the blocker
  alone: `_find_milestone` is 49 lines and also needs `_registry_headline`, `_same_number`, `_norm`,
  `_find_line`, and the milestone `lines` and `headline` structures. **Estimating it as "now
  unblocked, therefore small" would repeat the mistake this unit was about.**
- **`clippy` caught the field-assignment style, and `cargo` was again the check that worked** — the
  third time this session that the compiler found what my own reading did not.
