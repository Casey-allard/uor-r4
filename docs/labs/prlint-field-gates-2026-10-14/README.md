# Three `prlint` gates are Rust, and the coverage guard caught a vocabulary gap in itself

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `lint`/`prlint` port

## What landed

`crates/uor-r4-harness/src/prgates.rs` — the three gates whose entire input is the parsed fields,
ported **with Python's exact text and asserted against Python's own output for the same bodies**:

| gate | severity | condition |
|---|---|---|
| `prlint.line_declared` | error | a line name **and** a count are declared |
| `prlint.headline_present` | error | the **after** value is numeric |
| `prlint.decision_present` | error | a KEEP / REJECT / NOT YET PROMOTED decision |

**Asserted field for field — rule, severity, message *and* fix** — because **a port that raises the
right rule with the wrong fix tells a lab to do the wrong thing.**

Two details the capture made explicit rather than assumed:

- **the two gaps are joined into one problem** (`"the body does not declare Line name, count n/3"`)
  rather than raised twice, and **the order is Python's** — name first;
- **the headline hint carries the *before* value when there is one** (`headline 37 → <number/of>`),
  which is a different fix from the generic placeholder.

**`record_path` and `current_state_entry` are deliberately absent**: the first also **stats the
filesystem**, the second **reads `current-state.md`**. Each says so in the module docstring, **because
a gate that is missing and a gate that is not needed look identical from the outside.**

## The coverage guard caught itself

Parity failed immediately:

```
the Rust rule inventory declares rules Python does not register: prlint.decision_present,
prlint.headline_present, prlint.line_declared - a declared rule that does not exist inflates coverage
```

**The guard was right and its input was wrong.** `python_registered_rules()` returned only
`checkrules.rules()`, but **`prlint` declares its gates in `prlint.GATE_RULES`** — they are raised
against a **PR body**, which the engine never has, so they are **data rather than registered checks.**

**This is the third time in this port that "the registered set" turned out to have more than one
source** — after `mistakes._known_rule_ids` (three sources) and my own data file that took one and
stopped. **The first two produced a check that was wrong rather than missing; this one did too,
reporting three genuinely-ported rules as non-existent and calling the port inflated.**

**Fixed by delegating to `mistakes._known_rule_ids()` rather than re-listing the sources** — **it
already answers this question, and re-listing is how the previous two went wrong.**

## And it exposed a metric that was hiding progress

With the gates counted, the report moved from *"8 Python rules still in Python"* to *"18"*.
**The truth is that both numbers are needed and one was hiding the other:**

```
7 of check's own 39 rule(s) still in Python, 18 of 54 including prlint's PR-body gates
```

**`check`'s own rules are what `delegate_to_python` is waiting on**; the PR-body gates are real check
logic that lives in `prlint` and would otherwise read as already done. **Reporting one number made the
port look further along than it was the moment a gate moved into the inventory.**

## Verification

| check | result |
|---|---|
| the three gates against Python's output for the same bodies | asserted, field for field |
| a complete body raises none of them | asserted |
| each gate fires independently | asserted |
| `cargo test -p uor-r4-harness` | **333 passed** (5 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **36 implemented** |

## Limitations, stated

- **`check_pr` is not ported.** Eleven of its fourteen gates remain, plus the dispatcher and the
  `lint` command. **`lint` still delegates**, and these three gates are **not reachable from any
  command** — nothing calls them yet, so they are a verified unit rather than a working feature.
- **The three gates were asserted against four hand-written bodies**, not a real PR body.
- **`line_declared`'s fix text is a template hint**, quoting the contract's format — **it is not a
  valid body on its own**, and a lab pasting it literally would still be missing a real name and count.
- **The two denominators are now printed but nothing asserts them.** A future change could collapse
  them again and no test would notice — **the metric is honest today because I read it.**
