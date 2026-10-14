# `budget.no_new_spending_class` — a rule that fires on every run by design

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `check` port

## What landed

**`budget.no_new_spending_class`** — the owner's caps, printed on every `check`.

Coverage: **50 of 54**; **2 of `check`'s own rules remain**, down from 3.

## It does not test anything, and the port says so

`_check_caps` returns one result **unconditionally**:

```python
def _check_caps(paths, reg):
    return [RuleResult(rule_id="budget.no_new_spending_class", severity="info", ok=False, ...)]
```

**`ok=False` with `severity="info"` is the pair that makes the caps visible without counting as a
failure.** Python's own output confirms the shape:

```
harness check: 1 rules, 1 failed (0 error, 0 warn)
```

**So this rule is "failed" on every clean run, at a severity that contributes to neither count.**
That is deliberate — *"the owner caps are printed so a session cannot silently exceed them"* — and
**changing the severity would change the summary counts for every lab.**

**I did not invent a comparison.** There is no spend reading here to check against, and writing
*"report when a cap is exceeded"* would be **inventing a check the engine does not have** — the same
reasoning as `receipts.skill_usage_recorded` in the previous unit, **in the opposite direction: that
rule never fires, this one always does.**

**A test asserts both properties** — that it always fires, and that its severity is `info` — so a
future change that made it conditional would be **visible as a behaviour change rather than passing
silently with a different meaning.**

**The message is asserted including Python's formatting**: `$8.00/h` with two decimals, because the
value is a float and `:.2f` is what a reader compares against. **The port's message is byte-identical
to Python's.**

## `prlint.gates_declared` is NOT ported, and here is the honest reason

I did not port it this round, and the reason is not that it is hard.

**Python reconciles `LOGICAL_RULES` against the gates `check_pr()` raises by AST-introspecting
`prlint.py`.** In Rust, **the declared list and the gates share a crate**, so a mismatch is a
compiler error or a failing test rather than a runtime check — **the check is a proxy for something
the type system and the test suite already do.**

**But "structurally guaranteed" is a claim I have not measured**, and this port has a rule about
claims I have not measured. **A test that collects the rule ids `check_pr` actually emits and asserts
they are a subset of the declared set is the real port** — it exercises the code rather than parsing
it, which is *stronger* than Python's AST walk. **It is one test, it is not written, and I am saying
so rather than registering the rule and moving the number.**

## Verification

| check | result |
|---|---|
| the rule always fires, at `info`, path `AGENTS.md` | asserted |
| the message matches Python byte-for-byte, `$8.00/h` included | asserted |
| `cargo test -p uor-r4-harness` | **376 passed** (2 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **50 implemented**, **2 of `check`'s own rules left** |

## Limitations, stated

- **`check` still delegates.** Two of its own rules remain: `harness.documented_commands` (needs the
  `argparse` introspection ported properly) and `prlint.gates_declared` (needs the emitting test
  above) — plus `status.registry_vs_github`, which `parity` counts outside `check`'s own set.
- **The caps are constants in Rust now**, and **Python still holds its own copy.** `parity` compares
  shared constants (TTL, receipt kinds) but **not these two**, so **a cap changed in one
  implementation and not the other would not be caught by anything** — worth knowing before the
  numbers are next revised.
- **The rule cannot tell a lab it is exceeding a cap**, only that caps exist. **It is a reminder, not
  an interlock**, and the port preserves that rather than upgrading it silently.
