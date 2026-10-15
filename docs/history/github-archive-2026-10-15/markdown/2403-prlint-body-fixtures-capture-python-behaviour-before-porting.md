# #2403 — prlint body fixtures: capture Python behaviour before porting the parser (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T00:48:08Z
- **Updated:** 2026-10-14T00:50:34Z
- **Closed:** 2026-10-14T00:50:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2403
- **Merged:** 2026-10-14T00:50:11Z
- **Merge commit:** a988b5fac90cfd9d648ba23cda99e6d1af48cb35
- **Base / Head:** main / UOR-Foundation:deepseek/prlint-fixtures
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`prlint` body fixtures: capturing Python before porting it.** Tooling only.

Decision: **KEEP** (tooling).

## Why fixtures come before the port

`prlint` is **1,133 lines with 22 problem sites inside one function**, and `parse_pr_body` — the shared input to every gate — is **105 lines producing 25 fields**.

**`parity.py`, the most valuable verification in this port, dies with Python.** It has already caught the `prep.rs` stale rule list, the `guard_covered` 16-vs-40 bug, and a phantom declared rule. **When it goes, behaviour captured as data is the only safety net that outlives it.** And **a port written from the docstring and then checked against the docstring proves nothing.**

## What the capture exposed — by running the parser, not reading it

1. **The raw template yields an asymmetric pair: `line_count: 0` with `line_of: 3`.**
2. **`milestone_ref` is populated in bodies that name no milestone.**
3. **An empty body still yields `headline_before`/`headline_after` objects**, not nothing.

## And it caught my own error — the better argument

**I wrote finding 1 as "`line_of` is set on the raw template while `line_count` is not."** My own test asserted the absence and **failed**: the fixture holds `line_count: 0`, **not null.**

**The fixture corrected the prose.** I then fixed the assertion **twice**, because the first fix was a string replace against a block `cargo fmt` had already reformatted — it matched nothing and the test failed identically. **That is the ninth silent no-op in this session, and the second in this unit alone.**

**A capture that corrects the person who wrote it is doing exactly its job.**

## The fixtures

`crates/uor-r4-harness/rules/prlint-body-fixtures.json` — **5 self-contained cases** (body plus expected output), so they survive Python's retirement.

**`expected` is a record of behaviour, not a specification.** Where a value looks wrong the fixture is still right, and the port matches it **until someone deliberately changes the behaviour** — the difference between porting and "improving while porting".

```
cargo test -p uor-r4-harness                    -> 327 passed (4 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 330 passed
parity.py                                       -> exit 0
python3 scripts/check_no_softmax.py             -> OK, no softmax added
```

Record: `docs/labs/prlint-fixtures-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, **and its presence verified with a working pattern**
- [x] Branch deleted after merge

### Stated rather than hidden

- **5 cases is not coverage.** They exercise the separators, the template, the wrap and the empty body — **not sampled from real PR history.** A port passing all five can still differ on a body shape none of them has, and **the most likely such shape is a real PR body written by another lab, which none of these is.**
- **Nothing yet consumes the fixtures.** They are a target with no runner until the Rust parser exists, **so today they verify only that the capture is intact — not that any port agrees with it.**
- **The expectations were produced by Python, so a bug in Python is captured as correct.** That is the point of a behaviour record and also its limit: **a port that matches Python perfectly inherits Python's bugs, and the fixtures will call that agreement.**
- **`prlint` remains entirely Python and `lint` still delegates.** **This unit adds no ported behaviour.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

