# #2364 — D30 port 6: prep is Rust - the D22 gate (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T20:05:27Z
- **Updated:** 2026-10-13T20:07:51Z
- **Closed:** 2026-10-13T20:07:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2364
- **Merged:** 2026-10-13T20:07:31Z
- **Merge commit:** 7e7dd06359f294735c50d767fa9f488af1c4f8ec
- **Base / Head:** main / UOR-Foundation:deepseek/port-prep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 port 6.** Tooling only.

Decision: **KEEP** (tooling).

## What moved

`uor-harness prep new|fit|list` is Rust and **out of the delegated set**. `prep.rs` gained `prep_dir`, `sheet_name` and `render_fit`; the renderer was already there.

**This is the gate that decides whether a run may start**, so the property that matters is not that it works but that it agrees with the Python it replaces.

## Byte-identical sheets

Rust and Python `prep new -m m1` against the same fixture repository:

```
rust sheet lines:       67   python: 67
cmp -s /tmp/r.md /tmp/p.md  ->  BYTE-IDENTICAL
```

**A pre-registration is a contract, and two tools that render it differently produce two different contracts for the same milestone.** The rule block, the panel line carrying the D28 BLOCKED grader, the bars, the power block and the cost buckets all match.

`sheet_name` reproduces Python's `<milestone>-<mechanism>-<date>` slug **so both tools write one file rather than two** — two names for one sheet is how a milestone ends up with a duplicate pre-registration and `prep-check` reporting on whichever it finds first.

## `prep fit`

```
$ uor-harness prep fit -m m1 --verdict fit --reviewer claude --lab deepseek
TEST FITNESS: FIT - M1 #2029
sheet: MISSING
reviewer: claude
date: 2026-10-13
A FIT verdict clears compute for the configuration in that sheet only (D22).
recorded in docs/harness/ledger/deepseek-2026-10.jsonl
```

**An unknown verdict is refused, not recorded:** `--verdict FIT-ISH` exits 2. The verdict is exactly the field that decides whether compute may start, and a receipt saying `FIT-ISH` would clear a run on a typo.

**The receipt is best-effort on purpose** — Python's rule, kept: *the sheet matters more than its receipt.*

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 196 passed
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
parity.py                                       -> all comparisons ok
delegation notice for `prep list`               -> gone
```

Record: `docs/labs/port-prep-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`prep.py` is still reachable and is not retired.** Its five `check` rules have not all been ported, and **a Rust `check` that skipped them would be a silent downgrade** — which is why `check` still delegates.
- **`prep.panel_grader_permitted` is still not in the Rust `check`.** I added it last unit and flagged it as must-carry; **this unit did not change that**, so the guard exists only in Python.
- **`--dry-run` is not implemented.** A lab wanting a sheet without writing it uses `prep-render` — a different command name for the same output.
- **`prep new` overwrites an existing sheet without checking**, as Python does. Running it twice in a day silently replaces the first — **and a pre-registration that can be silently replaced is not much of a pre-registration.**
- **The non-default `NewSheet` fields are untested.** `--seeds`, `--rows`, `--split`, `--steps`, `--stop`, `--cost` pass through, but only the defaults were exercised against Python.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

