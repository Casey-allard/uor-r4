# D30 port 6: `prep` is Rust — the D22 gate

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; port 6 of the series

## What moved

`uor-harness prep new|fit|list` is Rust and **out of the delegated set**. `prep.rs` gained
`prep_dir`, `sheet_name` and `render_fit`; the renderer itself was already there from an earlier port.

**This is the gate that decides whether a run may start**, so the property that matters is not that it
works but that it agrees with the Python it replaces.

## Byte-identical sheets

A Rust `prep new -m m1` and a Python `prep new -m m1` were run against the same fixture repository and
diffed:

```
rust sheet lines:       67   python: 67
cmp -s /tmp/r.md /tmp/p.md  ->  BYTE-IDENTICAL
```

**Both the rendered sheet and its length match exactly**, including the rule block, the panel line with
the D28 BLOCKED grader, the bars, the power block and the cost buckets. The sheets being equal is the
whole test: a pre-registration is a contract, and two tools that render it differently produce two
different contracts for the same milestone.

`sheet_name` reproduces Python's `<milestone>-<mechanism>-<date>` slug **so both tools write one file
rather than two** — two names for one sheet is how a milestone ends up with a duplicate
pre-registration and `prep-check` reporting on whichever it finds first.

## `prep fit`

```
$ uor-harness prep fit -m m1 --verdict fit --reviewer claude --lab deepseek
TEST FITNESS: FIT - M1 #2029
sheet: MISSING
reviewer: claude
note: MISSING
date: 2026-10-13
A FIT verdict clears compute for the configuration in that sheet only (D22).
recorded in docs/harness/ledger/deepseek-2026-10.jsonl
```

**An unknown verdict is refused, not recorded:**

```
$ uor-harness prep fit -m m1 --verdict FIT-ISH
prep fit: verdict must be one of FIT, NOT_FIT, PENDING, ABSENT (got "FIT-ISH")
```

A receipt saying `FIT-ISH` would clear a run on a typo, and the verdict is exactly the field that
decides whether compute may start.

**The receipt is best-effort on purpose.** A broken ledger must not lose the review — Python's rule,
kept: *the sheet matters more than its receipt.*

## Verification

| check | result |
|---|---|
| Rust vs Python sheet | **byte-identical**, 67 lines each |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | 196 passed |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | **0 errors** |
| `parity.py` | all comparisons ok |
| delegation notice for `prep list` | **gone** |

## Limitations, stated

- **`prep.py` is still there and still reachable** through `cli.py`. It is not retired, because its
  four `check` rules (`prep.sheet_shape`, `prep.fit_before_run`, `prep.no_prohibited_teacher`,
  `prep.panel_frozen`, `prep.panel_grader_permitted`) have not all been ported — **and a Rust `check`
  that skipped them would be a silent downgrade**, which is why `check` still delegates.
- **`prep.panel_grader_permitted` is NOT in the Rust `check`.** I added it last unit and said then that
  the port must carry it. **The Rust `check` is not yet ported, so the rule exists only in Python —
  and this unit did not change that.** Recorded because it is the one guard I have explicitly flagged
  as must-carry and it is still outstanding.
- **`--dry-run` is not implemented.** Python has it for `prep new`; a lab wanting to see a sheet without
  writing it uses `uor-harness prep-render`, which is a **different command name** for the same output.
- **`prep new` writes without checking whether a sheet already exists.** Python does too, so the port
  agrees — but it means running `prep new` twice in a day silently overwrites the first sheet, and a
  pre-registration that can be silently replaced is not much of a pre-registration.
- **The `NewSheet` fields I did not wire** (`--seeds`, `--rows`, `--split`, `--steps`, `--stop`,
  `--cost`) are accepted and passed through, but I only exercised the defaults. A non-default value
  reaching the renderer differently from Python is untested.
