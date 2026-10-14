# `prlint` body fixtures: capturing Python before porting it

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `lint`/`prlint` port, the next item in the D30 order

## Why fixtures come before the port

`prlint` is **1,133 lines with 22 problem sites inside one function**, and `parse_pr_body` — the
shared input to every gate — is **105 lines producing 25 fields**.

**`parity.py`, the most valuable verification in this port, dies with Python.** It has already caught
the `prep.rs` stale rule list, the `guard_covered` 16-vs-40 bug, and a phantom declared rule. **When
it goes, behaviour captured as data is the only safety net that outlives it.**

**And a port written from the docstring and then checked against the docstring proves nothing.** This
is what the port gets checked against.

## What the capture exposed — by running the parser, not reading it

**Three cases a docstring-driven port gets wrong:**

1. **The raw template yields an asymmetric pair: `line_count: 0` with `line_of: 3`.**
2. **`milestone_ref` is populated in bodies that name no milestone.**
3. **An empty body still yields `headline_before`/`headline_after` objects**, not nothing.

## And it caught my own error, which is the better argument

**I wrote finding 1 as "`line_of` is set on the raw template while `line_count` is not."** My own test
asserted the absence and **failed** — the fixture holds `line_count: 0`, **not null.**

**The fixture corrected the prose.** I then fixed the assertion twice, because **the first fix was a
string replace against a block `cargo fmt` had already reformatted** — it matched nothing and the test
failed identically, which is the ninth silent no-op in this session and the second in this unit alone.

**A capture that corrects the person who wrote it is doing exactly its job.**

## The fixtures

`crates/uor-r4-harness/rules/prlint-body-fixtures.json` — **5 cases, self-contained** (body plus
expected output), so they survive Python's retirement:

| case | non-empty fields |
|---|---|
| `empty` | 2 |
| `template-raw` | 8 |
| `filled-basic` | 11 |
| `pipe-separators` | 11 |
| `wrapped-line` | 11 |

**`expected` is a record of behaviour, not a specification.** Where a value looks wrong the fixture is
still right, and the port matches it **until someone decides to change the behaviour deliberately** —
which is the difference between porting and "improving while porting".

**Regenerate** (while Python exists):

```python
import sys; sys.path.insert(0, "scripts/harness")
import prlint, json
d = prlint.parse_pr_body(body)
d.pop("raw", None); d.pop("body_text", None)
print(json.dumps(d, default=str, sort_keys=True))
```

## Verification

| check | result |
|---|---|
| fixtures load, 5 cases, none null | asserted |
| **25 field names recorded** | asserted, so a dropped field fails rather than shrinking the port |
| the three docstring-traps pinned | asserted, so they cannot be tidied away as fixture noise |
| bodies keep `·` and `→` | asserted |
| `cargo test -p uor-r4-harness` | **327 passed** (4 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 330 passed |
| `parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |

## Limitations, stated

- **5 cases is not coverage.** They were chosen to exercise the separators, the template, the wrap and
  the empty body — **not sampled from real PR history.** A port that passes all five can still differ
  on a body shape none of them has, and **the most likely such shape is a real PR body written by
  another lab, which none of these is.**
- **Nothing yet consumes the fixtures.** They are a target with no runner until the Rust parser
  exists, **so today they verify only that the capture is intact** — not that any port agrees with it.
- **The expectations were produced by Python, so a bug in Python is captured as correct.** That is the
  point of a behaviour record and it is also its limit: **a port that matches Python perfectly inherits
  Python's bugs, and the fixtures will call that agreement.**
- **`prlint` remains entirely Python and `lint` still delegates.** This unit adds no ported behaviour.
