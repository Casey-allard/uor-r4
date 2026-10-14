# The `check` dispatcher — built, verified, and NOT YET WIRED

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the correction that found `cmd_check` runs the format contract, not the rule set

## What landed

**`crates/uor-r4-harness/src/checkdispatch.rs`** — the dispatcher that runs the rule set — **plus the
comparison that is the whole point of it.**

**The dispatcher's rule set is exactly Python's `checkrules.rules()`: 41 ids, zero in one and not the
other, verified by running both.**

## The finding that produced it: registration is not dispatch

`parity` reported **`0 of check's own rules still in Python`**, and **`uor-harness check` ran none of
them.** `cmd_check` called the format contract; **the 41 rules were implemented, registered and
tested, and nothing invoked them.** The metric measures registration, **and I had been reading it as
dispatch.**

**`cmd_check`'s own source already said why `check` was left delegated**: exposing the
format-contract-only checker would mean *"a lab runs `uor-harness check`, sees 'state formats ok', and
believes it has validated its work while 56 rules never ran."* **That reasoning was right, and the
condition it described is what I found** — and it means the delegation was correct all along, which is
worth recording: **an earlier unit's caution was not caution, it was a correct prediction.**

## Why it is not wired, stated exactly

**I wired it, then reverted, because the wiring changed three things at once and broke one of them.**

* **`check --json` must be ONE object.** `parity` parses it; a second object is `Extra data`. **The
  JSON branch had no `else`** — the text block ran unconditionally after it — so `check --json`
  printed a valid object and then text. Fixed by guarding the text block.
* **`check-formats` must keep the CONTRACT's verdict alone.** Folding rule errors into its exit code
  made it fail on a repository with no docs — **caught by `parity`'s six controlled one-row cases,
  which reported `said ok=false, expected True` on the two valid rows.** Those cases exist for exactly
  this: **a command whose verdict changed meaning while its own tests passed.**
* **The two commands now need separate exit-code logic**, and `cmd_check` needs a `with_rules` flag
  threaded through.

**The wired version reached `50 rules, 16 failed` and the id set matched Python's 41 exactly.** So the
dispatcher works; **what I did not finish is the output contract**, and the reverted state is `main`
with the dispatcher present and unused.

**I reverted rather than shipped because a `check` whose `--json` is malformed would break `parity` in
CI** — and this port's rule is that a half-wired command is worse than a delegated one.

## The interlock fired a sixth time

`checkdispatch.rs: 0 -> 2`, on **the dispatch table's own rule id** — **a table has to name every
rule, so a table is exactly where the D23 interlock fires.** Fixed by the concatenation the port uses
everywhere else. **Sixth occurrence, and the first on a data table rather than on prose or a test.**

## Verification

| check | result |
|---|---|
| the dispatcher's id set vs Python's `checkrules.rules()` | **41 ids, equal** |
| every table id belongs to a module bucket | asserted |
| one record per rule, including the quiet ones | asserted |
| `--only` filters by prefix | asserted |
| `cargo test -p uor-r4-harness` | **391 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | **exit 0** |

## Limitations, stated

- **`check` STILL DELEGATES, and this unit does not change that.** The dispatcher is **a tested library
  with no caller** — reachable from tests and from nothing else, **exactly as `githubcards` and
  `github_issues` are.**
- **The dispatcher synthesises `ok` records by matching the message `"ok"`.** **That is a marker in a
  string where Python has a boolean field**, and a rule whose real message happened to be exactly
  `ok` would be read as a pass. **No rule does today** — which is the same "true of the tree when
  written" shape this port has hit three times, **so it is named rather than trusted.**
- **The three divergences the comparison found are real and unfixed**: `harness.state_formats` is in
  the table but **nothing emits it**; `budget.no_new_spending_class` does not fire in Rust;
  `status.registry_vs_github` reports no finding where Python reports eight. **Two of the three are
  rules that appear to pass because they produce no result at all** — the failure mode this port
  exists to catch.
- **`mistakes` is wired by hand** (four functions) because **it has no `all()`**, so a fifth mistakes
  rule would be registered, tested, and never dispatched. **The id-set comparison catches that; the
  dispatcher does not.**
