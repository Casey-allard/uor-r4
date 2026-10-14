# `check` is Rust — the last delegated command labs trust most

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** finishing the wiring the previous unit built and reverted

## What landed

**`uor-harness check` is Rust.** It runs **the frozen format contract AND all 41 rules**, and it no
longer prints *"not ported to Rust yet"*.

**Coverage: 53 of 54; `0 of check's own 39 rule(s) still in Python`.**

| | |
|---|---|
| `uor-harness check` | **48 rules, 14 failed (0 error, 14 warn)** |
| `check --json` | **one object**: 8 formats + 48 rule records |
| `parity.py` | **exit 0** |

## The previous unit reverted this, and the reason it failed is the reason it works now

**One boolean through one function.** `with_rules` threaded through `cmd_check` made
**`check-formats` load the registries on a bare repository** and return `2` — which **`parity`'s six
controlled one-row cases caught as `said ok=false, expected True` on the two valid rows.**

**So the fix is not a better flag, it is two functions.** `cmd_check` stays **exactly as `main` had
it** — the format contract and nothing else, proven by those controlled cases — and
**`cmd_check_rules`** adds the rule set and the single-object JSON.

**Two commands with two bodies cannot drift that way; a flag can.** The first attempt was not wrong
about what to run, it was wrong about **how many things one parameter had to mean at once.**

## The three things the previous unit listed, each now done

* **`check --json` is ONE object.** The formats print with an opening brace and **no closing brace**;
  the rules print into the same object; the brace closes once at the end. **A second object after the
  first is `Extra data` to any JSON reader**, which is how the first attempt was caught.
* **`check-formats` keeps the contract's verdict alone** — because it is a different function whose
  body never mentions the rules.
* **No `with_rules` flag exists at all.**

## An earlier unit's warning, confirmed rather than overridden

The dispatch site carried this note:

> exposing the format-contract-only checker as `check` "would mean a lab runs `uor-harness check`,
> sees 'state formats ok', and believes it has validated its work while 56 rules never ran."

**That was a correct prediction of the exact defect I found by verifying two units later.** The note
is kept in the source, with the correction that the objection is now answered.

## Verification

| check | result |
|---|---|
| `parity.py` (including the six controlled one-row cases) | **exit 0** |
| `uor-harness check` runs the rules | 48 rules, 14 failed |
| `check --json` parses as **one** object | 8 formats, 48 records |
| `uor-harness check` prints no delegation notice | confirmed |
| `cargo test -p uor-r4-harness` | **391 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Limitations, stated

- **`check` reports 48 rules where Python's `check` reports 57.** Python's 57 includes its 16 PR-body
  gates, which run against a **PR body** that `check` does not have; `uor-harness lint check --body-file`
  is where those live, and 13 of the 14 are Rust. **So the two "57" and "48" are not the same
  question — but a reader comparing them will think one implementation is missing nine rules**, and
  that needs saying in the output rather than only here.
- **The three dispatch divergences found last unit are still unfixed**: `harness.state_formats` is in
  the table but **nothing emits it**; `budget.no_new_spending_class` does not fire in Rust;
  `status.registry_vs_github` reports nothing where Python reports eight. **Two of the three are rules
  that look like passes because they produce no result at all.**
- **Five `delegate_to_python` sites remain** — the unported tail of `mistakes`, `git`, `multilab` and
  `team`. **`check` leaving the delegated set does not mean the objective is met.**
- **`check`'s non-JSON output is not byte-identical to Python's.** The shape is the port's own:
  `N rules, M failed (E error, W warn)` then the findings. **`parity` compares the JSON, not the text**,
  so the text form is unverified against Python and is stated as such.

---

## Correction to the verification method, found while confirming the merge

**The first check I ran after merging tested the WRONG BINARY.**

`uor-harness` on `PATH` resolves to **`~/.cargo/bin/uor-harness`**, dated **Oct 13 23:30** — an install
predating this work. It delegated, and it ran **55 rules over Python**. **The worktree build**
(`local/harness-target/debug/uor-harness`, **Oct 14 00:39**) runs **47 rules over Rust with zero
delegation notices.**

**So the merged code is correct and my confirmation of it was not.** The port's own rule — *verify the
artifact you shipped, not the one you remember building* — applies to the **binary**, not only to the
source.

**This is worth stating as a residual risk rather than a footnote**: **anyone running
`uor-harness check` from `PATH` today gets the python-delegating binary**, because nothing in this
merge reinstalls it. `docs/harness/BOOTSTRAP.md` says to build with
`CARGO_TARGET_DIR=local/harness-target`; **it does not say to install, and the install is stale.**

**Not fixed here** — a build-and-install step is a delivery question, not part of wiring `check`, and
inventing one silently would be the kind of change this port refuses to make without saying so.
