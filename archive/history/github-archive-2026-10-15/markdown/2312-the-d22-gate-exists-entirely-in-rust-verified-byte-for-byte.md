# #2312 — The D22 gate exists entirely in Rust, verified byte-for-byte (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:47:00Z
- **Updated:** 2026-10-13T06:49:23Z
- **Closed:** 2026-10-13T06:49:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2312
- **Merged:** 2026-10-13T06:49:03Z
- **Merge commit:** 07e9d85d8c7f2d3bbbf0d4bfb2e0272c0b7dcb63
- **Base / Head:** main / UOR-Foundation:harness/rust-prep-render
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Result

Decision: **KEEP** (tooling).

**The D22 gate now exists entirely in Rust.** `render_sheet` joins the read half, so the gate's
**decision** and its **authoring** are one implementation rather than half in each language. New
command: `uor-harness prep-render` — prints, never writes, because `prep new` writing files is a
separate decision and a renderer that writes on its first outing is one nobody can inspect first.

**Verified byte-identical to Python: 4,714 bytes, `diff` clean**, and that is now a permanent
`parity.py` check:

```
rendered pre-registration sheet identical: ok
```

A sheet is a **document two implementations must agree about** — a lab may read one written by either,
and the D22 gate scores against it. A one-character difference in a bar or a target is a different
pre-registration, which is why this compares the whole text rather than selected fields.

## Two real bugs in my Rust, both caught only by the comparison

**1. The date was empty.** Python defaults it to today. The date is part of the sheet's filename and
its identity.

**2. The mechanism brief claimed `docs/mechanisms/<name>.md` whenever the *name* was non-empty**,
without checking the file exists. A sheet citing a brief that does not exist **reads as prepared when
it is not** — and the brief is where the mechanism's idea and the fair test are stated. That is
precisely the *"looks fine, is wrong"* failure a pre-registration exists to prevent.

**Neither would have surfaced in a unit test written from my own assumptions about the expected
text.** Both were one `diff` away. That is the argument for comparing whole documents rather than
selected fields, and it is the third time this session that pointing an implementation at real data
found something reasoning did not.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 63 passed + 1 doc-test
python3 -m unittest                   -> 310 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check  -> 51 rules, 0 errors
python3 scripts/check_no_softmax.py   -> OK (1648, unchanged)
python3 scripts/harness/parity.py     -> all seven comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **`prep new` does not write yet.** The Rust renderer prints; Python still creates the file. The
  gate's text is now one implementation, its file-writing is not. That is the remaining half and it is
  small — but it is not done.
- **One test case, one milestone.** The byte-for-byte comparison uses `m1` with an empty mechanism.
  A milestone whose headline is qualitative or vacuous, or a sheet with every field filled in, could
  still differ. The registry parity covers the headline forms; the *render* parity does not cover
  every sheet shape.
- The `mechanism_path` fix lists known briefs on a miss. That list is read from the directory, so it
  is correct by construction — but it makes the rendered text depend on what is on disk, which means
  the parity check is comparing two live reads rather than two fixed strings.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

