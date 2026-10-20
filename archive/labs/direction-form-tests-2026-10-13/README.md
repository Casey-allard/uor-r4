# The `DIRECTION:` form's tests, which I claimed and had not written

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the previous unit, `docs/labs/briefing-direction-form-2026-10-13/README.md`

## What happened

The previous unit ported the `DIRECTION:` declaration form and stated, in the record and the PR:

> *"**`cargo test -p uor-r4-harness` | **318 passed** (3 new)**"*

**Rust was 318 before that unit and 318 after it.** The three tests **did not exist.** My insert was an
anchor-based string replace against a doc-comment I had already reformatted; **it matched nothing and
reported success.**

**I found it by verifying the specific hunk on `origin/main`** — the check I have been running all
session — which showed the label present and `a_direction_prefixed_declaration_is_read_and_labelled`
**absent**. The test *count* had not moved either, and **I read the unchanged number as the new number.**

## The claim I made and am retracting in the record

**"318 passed (3 new)" was false when written.** The number was right and the parenthetical was not,
**which is the worst form of a wrong number: it looks verified.** The record for that unit now carries
this correction.

## What is actually fixed now

The label and the anywhere-matching were real and are on `main`. **The three tests are here:**

- a `DIRECTION:`-prefixed declaration **is read and labelled** `lab prompt DIRECTION`;
- a plain declaration **is labelled `session-goal contract`**, so the two are distinguishable;
- a declaration **after other prose on one line is read**, which is the behaviour that requiring a
  word boundary had broken.

```
tests in briefingrules.rs:  9 -> 12
rust suite:                 318 -> 321
```

**Both numbers moved, and I checked both** — because the last time one of them moved in my head and
not in the file, I did not notice.

## Why this matters beyond one hunk

**This is the fourth silent no-op edit in this session**: a `len == 12` guard that matched nothing, a
shell-quoted commit that never ran, a repeated-substitution loop that reworded the same line six
times, and now this. **Each one reported success and each was caught by something else — a probe, a
test count, a hunk check.**

**The pattern is that an edit which changes nothing looks exactly like an edit that worked.** For a
record I wrote by hand, the only guard is checking the artefact rather than the intention — which is
what found this.

## Verification

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **321 passed** (318 + 3, and both numbers were watched) |
| tests in `briefingrules.rs` | **12** (was 9) |
| the three test names on `origin/main` after merge | verified by hunk |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 322 passed |
| `parity.py` | exit 0 |

## Limitations, stated

- **The correction is in the record, not in the merged PR body.** PR #2394's body still says
  *"318 passed (3 new)"* and **I cannot edit a merged PR's claim as it stands** — a reader of that PR
  sees the false statement. **The current-state entry carries the correction, and this record names
  the PR.**
- **Nothing detects this class of error automatically.** A test count that does not move is not
  checked against a claim that it did. **A `parity`-style comparison of "claimed new tests" against
  the suite delta does not exist**, and building one for prose is not obviously tractable.
- **I am relying on the same anchor-based edits to fix it.** This unit's insert was verified by
  reading the file's test count and the suite total **before and after** — which is the check that
  would have caught the original.
