# #2435 — Correction: check runs the format contract, not the rule set (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:08:11Z
- **Updated:** 2026-10-14T04:10:20Z
- **Closed:** 2026-10-14T04:10:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2435
- **Merged:** 2026-10-14T04:10:02Z
- **Merge commit:** 8115ce70a3a3c0fbafc8782fe2f48ee7da088971
- **Base / Head:** main / UOR-Foundation:deepseek/check-dispatch-finding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Correction: `check` runs the format contract, not the rule set. The objective is NOT met.** Tooling only.

Decision: **KEEP** (tooling).

## What the numbers say, and what they do not

`parity.py` now reports **`0 of check's own 39 rule(s) still in Python`** and **53 implemented**. That is a statement about **the rule inventory** — every rule in `check`'s set has a Rust implementation and is registered.

**It is not a statement that `uor-harness check` runs them.**

`cmd_check` (`crates/uor-r4-harness/src/main.rs:1990`) calls **`Contract::load(repo)` and `contract.check(repo)`**, which reports the state-format contract: formats, files, rows, violations. **The 53 registered rules are not invoked by it.** They exist, they are tested, and `cmd_check` does not call them.

**So `uor-harness check` is WEAKER than `python3 scripts/harness/cli.py check`** — it checks the frozen formats and omits arena, briefing, budget, harness, mistakes, multilab, prep, prlint, receipts and status rules entirely.

## This is the failure mode this port refuses, one level up

Every unit here has refused to ship **a weaker check wearing a real rule's name.** **`cmd_check` is that failure at the command level**: the rule count says complete, the registered ids say complete, and **the command a lab actually runs checks a fraction of what Python's does.**

**The parity metric I have been reporting measures REGISTRATION, NOT DISPATCH — and I read it as dispatch.** I found this by verifying what `check` does, not by trusting the number I had been quoting.

## And `prlint.parse` is not the blocker

**I had been treating `prlint.parse` as the last obstacle**, since it cannot exist in Rust (the parser is total). **It is not.** The obstacle is that **`cmd_check` must call the rule set before `check` can stop delegating.**

## What is actually done

- **53 of 54 rules implemented and registered** — every rule in `check`'s own set.
- **`prlint.parse` is the 54th**, **unreachable in Rust by construction** — a policy question, not a port.
- **Five real `delegate_to_python` sites remain** (`main.rs` 362, 548, 1034, 1167, 1305): the unported tail of `mistakes`, `git`, `multilab`, `team`.

```
cargo test -p uor-r4-harness -- --ignored the_table_agrees  -> 143 rows, identical to Python
cargo test -p uor-r4-harness                               -> 388 passed, 3 ignored
cargo fmt --check / clippy -D warnings                     -> clean / 0 findings
parity.py                                                  -> exit 0, 53 implemented, 0 of check's own
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Correction carried in `docs/labs/documented-commands-port-2026-10-14/README.md` and current state
- [x] Branch deleted after merge

### The next unit, precisely

**Wire `cmd_check` to the 53 rules** — a dispatcher plus **a parity comparison of `check`'s *output* against Python's, not its inventory.** **Until that lands, `check` must stay delegated**, because a `check` that returned the format contract's verdict while a lab believed it had run every rule **is exactly the silent downgrade this port exists to prevent.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

