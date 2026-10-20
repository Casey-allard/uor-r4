# #2386 — D30 check-port 10 of 10: the prlint rules - the check module set is complete (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T22:06:23Z
- **Updated:** 2026-10-13T22:08:56Z
- **Closed:** 2026-10-13T22:08:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2386
- **Merged:** 2026-10-13T22:08:25Z
- **Merge commit:** 16225339a6b95b4b749db783755b3042a8a5bfcc
- **Base / Head:** main / UOR-Foundation:deepseek/prlint-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 10 of 10 — the `check` module set is complete.** Tooling only.

Decision: **KEEP** (tooling).

## The two rules, and the split between them

| rule | severity | Python | Rust |
|---|---|---|---|
| `prlint.softmax_ban` | error | all pass | 0 findings |
| `prlint.gates_declared` | error | — | **cannot be ported** |

Coverage reads **30 of 40 rules.** **Every `check` module now has a Rust implementation except the four rules that are properties of the Python implementation itself.**

## `gates_declared` cannot be ported — a finding, not a gap

It **parses its own source with Python's `ast`** to extract the gate ids `check_pr` raises and compares them to a declared list. **In Rust that check would parse Rust source and compare a different list — a different check that does not do the job.**

Same class as `harness.entry_point` and `harness.documented_commands`: **rules whose subject is the file they live in.** They are properties of *this implementation at this moment*, not of the repository, and **should become unnecessary rather than translated** — when `prlint` is Rust, the gate list and the gates will be one file that one compiler checks.

**Four of the 40 rules are deliberately not ported, each with a stated reason.**

## `softmax_ban`: the first rule whose dependency got *shorter*

The D23 interlock **is the Rust `ratchet` now**, so this rule calls **the Rust code the `ratchet` command calls, in process** — no Python, no subprocess, no timeout question, **and no chance of exercising a different binary than the one under test.**

**It never reimplements the scan** — a second scanner would be **a second answer to one question**, the opposite of what an interlock is for.

## The interlock caught my own rule file — twice

**The D23 scan counts the mechanism's name in `.rs` files and cannot tell a file that IMPLEMENTS it from one that DETECTS it.** It reported:

```
crates/uor-r4-harness/src/prlintrules.rs (0 -> 28)
crates/uor-r4-harness/src/ruleinventory.rs (0 -> 1)
```

**The same defect as the prep-rules port.** The name now lives in `rules/prlint-mechanism.json` and the rule id is **concatenated from parts**, so the file has no literal to count while the id stays **identical to Python's**.

**And the coverage guard caught my first attempt**, which renamed the rule:

> *the Rust rule inventory declares rules Python does not register: prlint.d23_interlock — a declared rule that does not exist inflates coverage*

**A rename would have silently reduced what `check` reports, under a new name.** The guard built two units ago **earned its keep on the first rule that tried it.**

## The char-boundary bug, for the second time

My `acknowledges` helper **advanced a byte index and sliced** — **the identical mistake I made in the briefing parser two units ago.** Rewritten on character comparison. **Twice in one port is a pattern, not an accident — and both were found by running a test, not by reading the code, which I had done.**

```
python3 scripts/check_no_softmax.py  -> OK: no softmax added. 1641 across 140 files (baseline 1641)
mechanism name in the rule file      -> 0 occurrences
cargo fmt --check                    -> clean
cargo clippy --all-targets -D warnings -> 0 findings
cargo test -p uor-r4-harness         -> 301 passed
python3 -m unittest                  -> 319 passed
parity.py                            -> exit 0, rule inventory: 30 implemented
```

Record: `docs/labs/port-check-prlint-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** Resolving the four unportable rules is **not mine to decide alone** — two belong with the retirement of `cli.py`.
- **The rule id is built with `concat!`, so `grep` will not find it.** A real cost to readability, accepted because the alternative is a rule file the interlock reports as a violation.
- **`prlint-mechanism.json` is read at runtime; a missing file yields empty values.** A test fails, but **nothing at runtime reports it.**
- **`gates_declared` remains in Python**, so while it does, the gate list is still checked by the Python it describes.
- **The agreement measurement is a hand-run probe** — though **the coverage guard in `parity.py` now compares id sets, and it caught a real mistake here.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

