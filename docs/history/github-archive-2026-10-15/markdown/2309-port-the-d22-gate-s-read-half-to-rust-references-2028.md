# #2309 — Port the D22 gate's read half to Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:28:36Z
- **Updated:** 2026-10-13T06:30:58Z
- **Closed:** 2026-10-13T06:30:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2309
- **Merged:** 2026-10-13T06:30:38Z
- **Merge commit:** b509a6b9a78d352bfcef415a36f7171d677e51a0
- **Base / Head:** main / UOR-Foundation:harness/rust-prep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. This is a port, not a new capability.

## What and why

Last round I noted that I'd been polishing tooling *around* the port instead of porting. This is the
correction: a **real module**, and the most consequential one — the **D22 pre-registration gate**.

`crates/uor-r4-harness/src/prep.rs` reads every sheet and answers the gate's own question: what does
its `TEST FITNESS` verdict say, and which required blocks are missing?

**Verified identical to Python on the real sheet:**

```
fitness=PENDING   blocks=8/8 present   complete=true   may_compute=false
```

And that agreement is now a permanent check in `parity.py` — **a reader that disagrees about a verdict
is a gate that opens for one implementation and not the other.**

```
$ python3 scripts/harness/parity.py
  prep sheets read identically by both: ok
```

New command: `uor-harness prep-check [--json]`.

## Result

Decision: **KEEP** (tooling).

### The read half first, deliberately

It's where the gate **decides**, and it's read-only. Rendering a sheet (`prep new`) writes a file, so
it's the riskier half and goes next. A half-ported module that writes is worse than one that only
reads.

### Two properties that are the whole point

**1. Completeness is not fitness.** A sheet with **every block filled in** and `TEST FITNESS: PENDING`
is **not** cleared for compute. Completeness is *"the lab filled the form in"*; fitness is *"a reviewer
said the bar is testable"*. Conflating them would quietly turn the D22 gate into a formatting check —
and the real sheet in this repository is exactly that case: complete, and PENDING, and correctly
refused.

**2. Prose mentioning the gate is not a verdict.** Dated records and lessons say
`TEST FITNESS: FIT` constantly — including the docs for this very gate. The line is anchored at line
start, and `NOT_FIT` is matched **before** `FIT` so it cannot be read as its opposite. Tests cover a
table cell, an inline mention, a lowercase verdict, and `FITNESS` not matching `FIT`.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 54 passed + 1 doc-test
python3 -m unittest                   -> 310 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check  -> 50 rules, 0 errors
python3 scripts/harness/parity.py     -> prep sheets read identically by both: ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **This is the read half only.** `prep new` still renders in Python, so a lab creating a sheet is
  still running Python. The gate's *decision* is now checkable in both; its *authoring* is not yet.
- **The gate is advisory until someone calls it.** `prep-check` reports `may_compute`; nothing forces
  a lab to ask before running compute. That is the existing `prep.fit_before_run` rule's job, still in
  Python.
- The port stands at roughly **2,900 Rust lines against 11,566 Python**. `status`, `briefing`,
  `mistakes`, `prlint` and `check`'s rules remain. The Python harness stays authoritative until each
  piece passes the same checks.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

