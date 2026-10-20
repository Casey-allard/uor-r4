# #2432 — Port the registry decision; registry_vs_github is registered (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T03:52:47Z
- **Updated:** 2026-10-14T03:55:23Z
- **Closed:** 2026-10-14T03:54:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2432
- **Merged:** 2026-10-14T03:54:58Z
- **Merge commit:** 7144210b08e123b7f85b0c9f44a06ebe49fd8a13
- **Base / Head:** main / UOR-Foundation:deepseek/registry-decision
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The decision after the reading — `registry_vs_github` is registered.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**The six-outcome decision** after the card reading is Rust and **registered. Coverage 52 of 54 — ONE of `check`'s own rules remains**: `harness.documented_commands`.

## Verified against Python on live data

**All eight findings on the real repository agree message for message** — severity, `ok`, and text:

| milestone | outcome |
|---|---|
| M1, M4, M5–M8 | `no status card found…; nothing to compare` |
| M2, M3 | `has no machine-readable number; registry says…` |

**The test is `#[ignore]`d and asserts the two implementations AGREE** rather than what the findings say — the latter would be the *"true of the tree when written"* mistake this port has made three times.

## Three outcomes are "did not compare", and that is the point

**GitHub unavailable → `warn`, `ok=false`.** No comments cached → info. No card → info. **The port keeps all three distinct** — an earlier unit fixed this rule reporting a **green pass when it had not checked**, and **collapsing "not compared" into "agrees" is exactly the defect that fix closed.**

## The bug the live comparison found: `repr` chooses its quote

My `py_repr` always used `'`. **The live comparison failed on M2**, whose card text contains an apostrophe — **Python's `repr` uses `"` when the string contains a `'` and no `"`.**

**A message compared character-for-character is behaviour**, so *"usually the same quote"* shows up **only in messages containing an apostrophe — which is most prose a lab writes.**

**Second time this port needed `py_repr`** — and **the live test caught what the unit tests could not: the fixture texts had no apostrophes.**

```
cargo test -p uor-r4-harness -- --ignored the_decision_matches_python  -> agrees, 8 findings
cargo test -p uor-r4-harness                    -> 388 passed, 2 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 52 implemented
```

Record: `docs/labs/registry-decision-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** One rule remains — `harness.documented_commands` — **and until it lands, `check` keeps delegating, because a `check` that skipped it would be a silent downgrade.**
- **The live test is `#[ignore]`d, so CI does not run it.** **The `repr` bug it caught would not have been caught by CI either** — structurally, because **the fixtures were text I wrote and I did not put an apostrophe in any of them.**
- **`github_issues` is called with `refresh` passed through**, so a run with `refresh=false` reads the cache — **which makes the answer depend on cache age.** Python has the same shape.
- **A card whose number is present and non-numeric is a case neither implementation handles by design**, since the reading parses only digits.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

