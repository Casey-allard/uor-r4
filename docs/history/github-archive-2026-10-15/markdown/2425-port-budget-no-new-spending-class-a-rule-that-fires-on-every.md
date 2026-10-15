# #2425 — Port budget.no_new_spending_class: a rule that fires on every run by design (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T03:04:41Z
- **Updated:** 2026-10-14T03:07:55Z
- **Closed:** 2026-10-14T03:07:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2425
- **Merged:** 2026-10-14T03:07:28Z
- **Merge commit:** f2936d8eff90055b41f28aea40243de1115ee09a
- **Base / Head:** main / UOR-Foundation:deepseek/gates-budget
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`budget.no_new_spending_class` — a rule that fires on every run by design.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`budget.no_new_spending_class`** — the owner's caps, printed on every `check`. Coverage **50 of 54**; **2 of `check`'s own rules remain**, down from 3.

## It does not test anything, and the port says so

`_check_caps` returns one result **unconditionally**, with `ok=False` and severity `info`. **That pair is what makes the caps visible without counting as a failure** — Python's own output confirms the shape:

```
harness check: 1 rules, 1 failed (0 error, 0 warn)
```

**So this rule is "failed" on every clean run, at a severity that contributes to neither count.** Deliberate — *"the owner caps are printed so a session cannot silently exceed them"* — and **changing the severity would change the summary counts for every lab.**

**I did not invent a comparison.** There is no spend reading to check against, and writing *"report when a cap is exceeded"* would be **inventing a check the engine does not have** — the same reasoning as `receipts.skill_usage_recorded`, **in the opposite direction: that rule never fires, this one always does.**

**A test asserts both properties**, so a future change making it conditional is **visible as a behaviour change rather than passing silently with a different meaning.** The message is asserted **including Python's `:.2f` formatting** — byte-identical.

## `prlint.gates_declared` is NOT ported, and the reason is not difficulty

**Python reconciles `LOGICAL_RULES` against the gates `check_pr()` raises by AST-introspecting `prlint.py`.** In Rust, **the declared list and the gates share a crate**, so a mismatch is a compiler error or a failing test — **the check is a proxy for something the type system and test suite already do.**

**But "structurally guaranteed" is a claim I have not measured, and this port has a rule about claims I have not measured.** The real port is **a test that collects the rule ids `check_pr` actually emits and asserts they are a subset of the declared set** — it would exercise the code rather than parse it, **stronger than Python's AST walk.**

**It is one test. It is not written. I am saying so rather than registering the rule and moving the number.**

```
cargo test -p uor-r4-harness                    -> 376 passed (2 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 50 implemented
```

Record: `docs/labs/spending-caps-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** Two of its own rules remain: **`harness.documented_commands`** (needs the `argparse` introspection ported properly) and **`prlint.gates_declared`** (needs the emitting test above) — plus `status.registry_vs_github`, which `parity` counts outside `check`'s own set.
- **The caps are constants in Rust now, and Python still holds its own copy.** `parity` compares shared constants (TTL, receipt kinds) but **not these two**, so **a cap changed in one implementation and not the other would not be caught by anything** — worth knowing before the numbers are next revised.
- **The rule cannot tell a lab it is exceeding a cap**, only that caps exist. **It is a reminder, not an interlock**, and the port preserves that rather than upgrading it silently.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

