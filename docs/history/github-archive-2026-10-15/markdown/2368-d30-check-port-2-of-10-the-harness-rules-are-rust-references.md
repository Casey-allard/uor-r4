# #2368 — D30 check-port 2 of 10: the harness rules are Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T20:25:45Z
- **Updated:** 2026-10-13T20:28:06Z
- **Closed:** 2026-10-13T20:27:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2368
- **Merged:** 2026-10-13T20:27:47Z
- **Merge commit:** b593db0d05d5d92ebd817f6ee726cf9522724b1a
- **Base / Head:** main / UOR-Foundation:deepseek/check-harness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 2 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## The three rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `harness.milestones.present` | error | all pass | 0 findings |
| `harness.skills.nonempty` | error | all pass | 0 findings |
| `harness.mistakes.have_guards` | warn | all pass | 0 findings |

Measured on the real repository, rule by rule.

`mistakes.have_guards` **keeps two failures apart**, which is why it is worth porting carefully:

- **no guard at all** — the lesson is a note that will be forgotten again;
- **a guard that names no rule** — `guard: be careful` **reads as a control and enforces nothing**.

Both asserted separately, with a test that fails if they ever report identically. **`none: <reason>` is accepted on purpose** — forcing a guard manufactures a fake rule, worse than a recorded absence. **A closed lesson is not reported**: its absence is already history.

## Three of the six `harness` rules are deliberately NOT ported

`harness.entry_point` (docs must invoke `uor-harness`, not `cli.py`) and `harness.documented_commands` (**documented commands exist in the parser — by reading the Python parser**) are **checks about the Python entry point that die with it.** Porting them would produce a Rust program that reads a Python argument parser. **They belong with the retirement of `cli.py`, not before it** — and `harness.entry_point` should become *unnecessary rather than translated*, because the condition it asserts will be true by construction.

`harness.state_formats` is the one of the three worth porting; it needs the contract modelled in Rust first.

## The finding that matters more than the port: `check`'s rule set is a moving target

**The registered rule count moved three times during this unit alone — 54, then 53, then 55 — while this branch touches no file under `scripts/harness/`.**

So the movement is **other labs registering rules into the same modules while I port them.** A module is not "done" because it matched Python at one instant: if another lab adds a rule to that module afterwards, **the Rust module is short a rule and nothing reports it** — the port would look complete while checking less.

**The 53/54 difference I flagged in my previous unit was this, not a dropped guard.** I was right to record it rather than explain it away, and **wrong to imply it might be a regression.**

**This is a gap in the porting method I have not closed. A per-module rule-count check belongs in `parity.py` before `check` stops delegating** — without it, the last module landing is not evidence that the port is complete.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 221 passed (9 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
python3 scripts/check_no_softmax.py             -> OK, no softmax added
parity.py                                       -> all comparisons ok
```

Record: `docs/labs/port-check-harness-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 2 of 10 modules done.
- **The per-module rule-count check is not written.** It is named as the next dependency, not done.
- **Three tests use temp fixtures** rather than the real registry; the real-registry assertions are separate and do run.
- **`mistakes_have_guards` treats `status != "active"` as closed**, so a typo'd status **silently disables the rule for that lesson**. Python behaves the same; the port agrees, and the defect is recorded rather than quietly fixed in only one implementation.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

