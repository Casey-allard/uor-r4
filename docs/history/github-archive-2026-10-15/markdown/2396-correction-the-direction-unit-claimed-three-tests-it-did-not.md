# #2396 — Correction: the DIRECTION: unit claimed three tests it did not have (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T23:17:13Z
- **Updated:** 2026-10-13T23:20:01Z
- **Closed:** 2026-10-13T23:19:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2396
- **Merged:** 2026-10-13T23:19:32Z
- **Merge commit:** 4b823beaa14ee769f9e104c46fac90e76352938d
- **Base / Head:** main / UOR-Foundation:deepseek/direction-tests
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Correction and fix: the previous unit claimed three tests it did not have.** Tooling only.

Decision: **KEEP** (tooling).

## What happened

The previous unit ported the `DIRECTION:` declaration form and stated, in the record **and in PR #2394**:

> `cargo test -p uor-r4-harness` | **318 passed** (3 new)

**Rust was 318 before that unit and 318 after it. The three tests did not exist.**

My insert was an **anchor-based string replace against a doc-comment I had already reformatted**; it matched nothing and reported success. **I found it by verifying the specific hunk on `origin/main`** — the check I've run all session — which showed the label present and the test function **absent**. The test count hadn't moved either, **and I read the unchanged number as the new number.**

**"318 passed (3 new)" was false when written.** The number was right; the parenthetical wasn't — **which is the worst form of a wrong number: it looks verified.**

## What was real, and what this PR adds

**The label and the anywhere-matching were real and are on `main`.** The three tests are here:

- a `DIRECTION:`-prefixed declaration **is read and labelled** `lab prompt DIRECTION`;
- a plain declaration **is labelled `session-goal contract`**, so the two are distinguishable;
- a declaration **after other prose on one line is read** — the behaviour that requiring a word boundary had broken.

```
tests in briefingrules.rs:  9 -> 12
rust suite:                 318 -> 321
```

**Both numbers moved, and I checked both** — because the last time one moved in my head and not in the file, I didn't notice.

## Why this matters beyond one hunk

**This is the fourth silent no-op edit in this session:** a length guard that matched nothing, a shell-quoted commit that never executed, a substitution loop that reworded one line six times, and now this. **Each reported success; each was caught by something else** — a probe, a test count, a hunk check.

**An edit which changes nothing looks exactly like an edit that worked.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 321 passed
python3 -m unittest discover -s tests/harness   -> 322 passed
parity.py                                       -> exit 0
```

Records: `docs/labs/direction-form-tests-2026-10-13/README.md` (this fix) and the **corrected** `docs/labs/briefing-direction-form-2026-10-13/README.md`.

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, **and the previous record corrected in place**
- [x] Branch deleted after merge

### Stated rather than hidden

- **The correction is in the record, not in PR #2394's body.** That PR still says *"318 passed (3 new)"* and **a reader of it sees the false statement** — I can't retroactively edit a merged PR's claim.
- **Nothing detects this class of error automatically.** A test count that doesn't move isn't checked against a claim that it did. **A comparison of "claimed new tests" against the suite delta does not exist.**
- **I'm relying on the same anchor-based edits to fix it.** This unit's insert was verified by reading the file's test count and the suite total **before and after** — which is the check that would have caught the original.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

