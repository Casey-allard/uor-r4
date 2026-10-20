# #2407 — Three prlint gates ported; the coverage guard caught a vocabulary gap in itself (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T01:17:27Z
- **Updated:** 2026-10-14T01:19:55Z
- **Closed:** 2026-10-14T01:19:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2407
- **Merged:** 2026-10-14T01:19:28Z
- **Merge commit:** b1c395fe798a3fc2de111bcbe962dc85306c3b4c
- **Base / Head:** main / UOR-Foundation:deepseek/prlint-field-gates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Three `prlint` gates are Rust, and the coverage guard caught a vocabulary gap in itself.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

`crates/uor-r4-harness/src/prgates.rs` — the three gates whose entire input is the parsed fields, ported **with Python's exact text and asserted against Python's own output for the same bodies**:

| gate | severity | condition |
|---|---|---|
| `prlint.line_declared` | error | a line name **and** a count are declared |
| `prlint.headline_present` | error | the **after** value is numeric |
| `prlint.decision_present` | error | a KEEP / REJECT / NOT YET PROMOTED decision |

**Asserted field for field — rule, severity, message *and* fix** — because **a port that raises the right rule with the wrong fix tells a lab to do the wrong thing.**

Two details the capture made explicit: **the two gaps are joined into one problem** (`"the body does not declare Line name, count n/3"`) with Python's order — name first; and **the headline hint carries the *before* value** (`headline 37 → <number/of>`), a different fix from the generic placeholder.

**`record_path` and `current_state_entry` are deliberately absent** — the first **stats the filesystem**, the second **reads `current-state.md`** — and **each says so in the module docstring, because a gate that is missing and a gate that is not needed look identical from the outside.**

## The coverage guard caught itself

```
the Rust rule inventory declares rules Python does not register: prlint.decision_present,
prlint.headline_present, prlint.line_declared - a declared rule that does not exist inflates coverage
```

**The guard was right and its input was wrong.** `python_registered_rules()` returned only `checkrules.rules()`, but **`prlint` declares its gates in `prlint.GATE_RULES`** — raised against a **PR body**, which the engine never has, so they are **data rather than registered checks.**

**Third time in this port that "the registered set" had more than one source** — after `mistakes._known_rule_ids` (three sources) and my own data file that took one and stopped. **The first two produced a check that was wrong rather than missing; this one did too**, reporting three genuinely-ported rules as non-existent.

**Fixed by delegating to `mistakes._known_rule_ids()` rather than re-listing the sources** — **it already answers this question, and re-listing is how the previous two went wrong.**

## And it exposed a metric hiding progress

The report moved from *"8 Python rules still in Python"* to *"18"*. **Both numbers are needed:**

```
7 of check's own 39 rule(s) still in Python, 18 of 54 including prlint's PR-body gates
```

**`check`'s own rules are what `delegate_to_python` waits on**; the PR-body gates are real check logic that would otherwise read as already done.

```
cargo test -p uor-r4-harness                    -> 333 passed (5 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 36 implemented
```

Record: `docs/labs/prlint-field-gates-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check_pr` is not ported. Eleven of its fourteen gates remain**, plus the dispatcher and the `lint` command — and **these three gates are not reachable from any command. Nothing calls them yet, so they are a verified unit rather than a working feature.**
- **The three were asserted against four hand-written bodies**, not a real PR body.
- **`line_declared`'s fix text is a template hint** quoting the contract's format — **not a valid body on its own.** A lab pasting it literally would still be missing a real name and count.
- **The two denominators are printed but nothing asserts them.** A future change could collapse them again and no test would notice — **the metric is honest today because I read it.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

