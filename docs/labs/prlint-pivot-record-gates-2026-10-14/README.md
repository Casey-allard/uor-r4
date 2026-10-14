# `pivot_at_three` and `record_path` — the first gate that stats the filesystem

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `lint`/`prlint` port

## What landed

**Two more gates, both asserted against Python's own output for the same bodies:**

| gate | severity | condition |
|---|---|---|
| `prlint.pivot_at_three` | error | the line is at 3/3 **and** no pivot is announced |
| `prlint.record_path` | warn | no dated record path **named**, or a named one that **does not exist** |

Coverage: **40 of 54**, up from 38.

## The details that make these two worth their tests

**`pivot_at_three` has two branches, and the `where` clause says which.** Either side can put the
line at three — the count the body declares, or the registry's count for that line — and the two
messages send a reader to **different places**: one to fix the body, the other to check whether the
line really moved. Both asserted:

```
line 'starling read' is at registry count 3/3 and the body announces no pivot
line 'exact addressed memory' is at 3/3 declared and the body announces no pivot
```

**A line at three is not a failure; it is where a decision is owed**, so the fix offers both ways out
— archive the line, or name the one decisive run. **And announcing a pivot silences it**, asserted,
because the gate asks for a decision rather than for the line to stop.

**`record_path` is the first gate that reads the filesystem**, which is why it was not ported with
the field-only group. **Its second branch is the one worth having**: a PR naming a record it never
wrote. Both branches asserted, and **an existing record is asserted silent** — the property that
keeps the gate usable rather than something labs route around.

**The fix text names *today*** (`docs/labs/<topic>-2026-10-14/README.md`), which is exactly why the
date is a **parameter** of `filesystem_gates` rather than read inside the gate: **a gate that reads
the clock cannot be tested at a fixed instant**, which is the defect this session already fixed once
in `parity`.

## Verification

| check | result |
|---|---|
| both `pivot_at_three` branches, against Python's text | asserted |
| an announced pivot silences it | asserted |
| a line below three is silent | asserted |
| both `record_path` branches, against Python's text | asserted |
| an existing record is silent | asserted |
| `cargo test -p uor-r4-harness` | **345 passed** (5 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **40 implemented** |

## Limitations, stated

- **`check_pr` is still not ported.** Seven of fourteen gates remain, plus the dispatcher and the
  `lint` command. **`lint` still delegates, and none of the eight ported gates is reachable from any
  command** — verified units, not a working feature.
- **`record_path`'s existence check uses `Path::exists`, which follows symlinks and returns `false`
  on a permission error.** Python's `Path.exists()` does the same, **so they agree today — but a
  permission-denied record would be reported as missing by both**, which is a wrong finding rather
  than a missing one.
- **`pivot_at_three` reads `has_pivot` from the parser**, and that flag is a regex over the body for
  `pivot (card|recorded|posted|point|line)`. **A lab that announces a pivot in wording outside that
  list is reported as announcing none** — the port inherits that, and the fixtures do not cover it.
- **The `today` parameter is passed but `pivot_at_three` ignores it.** It is in the signature for the
  shared `filesystem_gates` entry point; **an unused parameter on a gate that does not need it is
  noise, and removing it would mean splitting the entry point.** Recorded rather than hidden.
