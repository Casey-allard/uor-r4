# #2437 — Build the check dispatcher; verified against Python, not yet wired (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:27:16Z
- **Updated:** 2026-10-14T04:30:03Z
- **Closed:** 2026-10-14T04:29:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2437
- **Merged:** 2026-10-14T04:29:41Z
- **Merge commit:** 57be886cfdcadfb7f3aaecedbfce12424648cc36
- **Base / Head:** main / UOR-Foundation:deepseek/check-dispatch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The `check` dispatcher — built, verified against Python, and NOT YET WIRED.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`checkdispatch.rs`** — the dispatcher that runs the rule set — **plus the comparison that is the point of it: its rule set is exactly Python's `checkrules.rules()`, 41 ids, zero in one and not the other, verified by running both.**

## The finding that produced it: registration is not dispatch

`parity` reported **`0 of check's own rules still in Python`**, and **`uor-harness check` ran none of them.** `cmd_check` called the format contract; **the 41 rules were implemented, registered and tested, and nothing invoked them.** **The metric measures registration, and I had been reading it as dispatch.**

**`cmd_check`'s own source already said why `check` was left delegated**: exposing the format-contract-only checker would mean *"a lab runs `uor-harness check`, sees 'state formats ok', and believes it has validated its work while 56 rules never ran."*

**That reasoning was right, and the condition it described is what I found** — **so the delegation was correct all along. An earlier unit's caution was not caution; it was a correct prediction.**

## Why it is not wired, stated exactly

**I wired it, then reverted, because the wiring changed three things at once and broke one.**

- **`check --json` must be ONE object.** `parity` parses it, and a second object is `Extra data`. **The JSON branch had no `else`** — the text block ran unconditionally after it. Fixed by guarding the text block.
- **`check-formats` must keep the CONTRACT's verdict alone.** Folding rule errors into its exit code made it fail on a repository with no docs — **caught by `parity`'s six controlled one-row cases: `said ok=false, expected True` on the two valid rows.** Those cases exist for exactly this: **a command whose verdict changed meaning while its own tests passed.**
- **The two commands need separate exit-code logic** and a `with_rules` flag threaded through.

**The wired version reached `50 rules, 16 failed` with the id set matching Python's 41 exactly** — so the dispatcher works; **what I did not finish is the output contract.**

**I reverted rather than shipped because a `check` whose `--json` is malformed would break `parity` in CI**, and a half-wired command is worse than a delegated one.

## The interlock fired a sixth time

`checkdispatch.rs: 0 -> 2`, on **the dispatch table's own rule id** — **a table must name every rule, so a table is exactly where the D23 interlock fires.** Fixed by the concatenation the port uses everywhere else.

```
cargo test -p uor-r4-harness -- --ignored the_table_matches  -> 41 ids, equal to Python
cargo test -p uor-r4-harness                                 -> 391 passed, 4 ignored
cargo fmt --check / clippy -D warnings                       -> clean / 0 findings
python3 scripts/check_no_softmax.py                          -> OK — 1641, baseline 1641
parity.py                                                    -> exit 0
```

Record: `docs/labs/check-dispatcher-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` STILL DELEGATES.** The dispatcher is **a tested library with no caller** — reachable from tests and nothing else, **exactly as `githubcards` and `github_issues` are.**
- **The dispatcher synthesises `ok` records by matching the message `"ok"`** — **a marker in a string where Python has a boolean field.** A rule whose real message were exactly `ok` would read as a pass. **No rule does today, which is the same "true of the tree when written" shape this port has hit three times, so it is named rather than trusted.**
- **Three divergences the comparison found are real and unfixed**: `harness.state_formats` is in the table but **nothing emits it**; `budget.no_new_spending_class` does not fire in Rust; `status.registry_vs_github` reports nothing where Python reports eight. **Two of the three are rules that appear to pass because they produce no result at all** — the failure mode this port exists to catch.
- **`mistakes` is wired by hand** (four functions) because **it has no `all()`** — a fifth mistakes rule would be registered, tested, and never dispatched. **The id-set comparison catches that; the dispatcher does not.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

