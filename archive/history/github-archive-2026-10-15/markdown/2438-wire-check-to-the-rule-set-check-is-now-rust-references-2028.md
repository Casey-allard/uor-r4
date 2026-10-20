# #2438 — Wire check to the rule set; check is now Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:36:14Z
- **Updated:** 2026-10-14T04:39:01Z
- **Closed:** 2026-10-14T04:38:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2438
- **Merged:** 2026-10-14T04:38:39Z
- **Merge commit:** 591319482c7860ab62122c05e4280337e703f4ca
- **Base / Head:** main / UOR-Foundation:deepseek/check-wire
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`check` is Rust — the last delegated command labs trust most.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`uor-harness check` runs the frozen format contract AND all 41 rules**, and no longer prints *"not ported to Rust yet"*. **Coverage 53 of 54; `0 of check`'s own 39 rules still in Python.**

| | |
|---|---|
| `uor-harness check` | **48 rules, 14 failed (0 error, 14 warn)** |
| `check --json` | **one object**: 8 formats + 48 rule records |
| `parity.py` | **exit 0** |

## The previous unit reverted this, and why it works now

**One boolean through one function.** `with_rules` threaded through `cmd_check` made **`check-formats` load the registries on a bare repository** and return `2` — **caught by `parity`'s six controlled one-row cases as `said ok=false, expected True` on the valid rows.**

**So the fix is not a better flag, it is two functions.** `cmd_check` stays **exactly as `main` had it** — the contract and nothing else, proven by those controlled cases — and **`cmd_check_rules`** adds the rule set and the single-object JSON.

**Two commands with two bodies cannot drift that way; a flag can.** The first attempt was not wrong about *what* to run — it was wrong about **how many things one parameter had to mean at once.**

## The three items from last unit, each done

* **`check --json` is ONE object** — formats print with an opening brace and **no closing brace**, the rules print into the same object, and the brace closes once.
* **`check-formats` keeps the contract's verdict alone**, because its body never mentions the rules.
* **No `with_rules` flag exists at all.**

## An earlier unit's warning, confirmed

The dispatch site already carried this note:

> exposing the format-contract-only checker as `check` *"would mean a lab runs `uor-harness check`, sees 'state formats ok', and believes it has validated its work while 56 rules never ran."*

**That was a correct prediction of the exact defect I found by verifying two units later.** The note is kept, with the correction that the objection is answered.

```
cargo test -p uor-r4-harness                    -> 391 passed, 4 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0 (incl. the 6 controlled cases)
```

Record: `docs/labs/check-wired-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` reports 48 rules where Python reports 57.** Python's 57 includes its 16 PR-body gates, which run against a **PR body** `check` does not have — those live in `lint check --body-file`, 13 of 14 in Rust. **The numbers are not the same question, but a reader comparing them will think nine rules are missing**, and that needs saying in the output rather than only in the record.
- **Three dispatch divergences found last unit are still unfixed**: `harness.state_formats` is in the table but **nothing emits it**; `budget.no_new_spending_class` does not fire; `status.registry_vs_github` reports nothing where Python reports eight. **Two of the three are rules that look like passes because they produce no result at all.**
- **Five `delegate_to_python` sites remain** — the unported tail of `mistakes`, `git`, `multilab`, `team`. **`check` leaving the delegated set does not mean the objective is met.**
- **`check`'s non-JSON output is not byte-identical to Python's.** `parity` compares the **JSON**, not the text — **so the text form is unverified against Python and is stated as such.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

