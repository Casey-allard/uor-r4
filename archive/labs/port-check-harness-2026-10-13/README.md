# D30 check-port 2 of 10: the `harness` rules are Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 2 of 10

## The three rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `harness.milestones.present` | error | all pass | 0 findings |
| `harness.skills.nonempty` | error | all pass | 0 findings |
| `harness.mistakes.have_guards` | warn | all pass | 0 findings |

**Measured on the real repository, rule by rule.**

## `harness.mistakes.have_guards` keeps two failures apart

That distinction is the reason this rule is worth porting carefully rather than quickly:

- **no guard at all** — the lesson is a note that will be forgotten again;
- **a guard that names no rule** — `guard: be careful` reads as a control and **enforces nothing**.

Those are different problems with different fixes, and a rule that collapsed them would send a reader
to the wrong one. Both are asserted separately, with a test that fails if they ever report identically.

**`none: <reason>` is accepted on purpose.** Some lessons genuinely have no mechanical guard, and
forcing one manufactures a fake rule — worse than a recorded absence. The escape is explicit and has
to be written down.

**A closed lesson is not reported.** Its guard's absence is already history; re-reporting it would
train readers to ignore the rule.

## The three rules NOT in this unit, and why

`harness.state_formats`, `harness.entry_point` and `harness.documented_commands` are **not ported
here**, and two of them should probably never be:

- **`harness.entry_point`** checks that normative docs invoke `uor-harness` rather than
  `scripts/harness/cli.py`.
- **`harness.documented_commands`** checks that commands named in the docs exist in the argument parser
  — **reading the Python parser to do it.**

Both are **checks about the Python entry point that die with the Python entry point.** Porting them to
Rust would produce a Rust program that reads a Python argument parser, which is a check that cannot
outlive the thing it checks. **They belong with the retirement of `cli.py`, not before it** — and when
the port finishes, `harness.entry_point` should become unnecessary rather than translated, because the
condition it asserts will be true by construction.

`harness.state_formats` reads the frozen format contract against persisted state. It is a real rule with
a real subject, and it is the one of the three that should be ported; it needs the `state-formats.json`
contract modelled in Rust first.

## Verification

| check | result |
|---|---|
| Rust vs Python, all three rules | **identical — all pass in both** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **221 passed** (9 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | all comparisons ok |

## Limitations, stated

- **`check` still delegates.** 2 of 10 modules done; `multilab`, `mistakes`, `status`, `arena`,
  `briefing`, `receipts`, `budget` and `prlint` remain, plus the three `harness` rules above.
- **The registered rule count moved three times during this unit alone — 54, then 53, then 55.** I
  changed no Python rule: this branch's diff touches **no file under `scripts/harness/`**. So the
  movement is **other labs registering rules into the same modules while I port them**, which is the
  real finding and a more interesting one than a miscount:

  **`check`'s rule set is a moving target, and a port measured against it can silently lag.** A module
  is not "done" because it matched Python at one instant; if another lab adds a rule to that module
  afterwards, the Rust module is short a rule and **nothing reports it** — the port would look
  complete while checking less. **The 53/54 difference I flagged in my last unit was this, not a
  dropped guard**, and I was right to record it rather than explain it away, and wrong to imply it
  might be a regression.

  **This is a gap in the porting method I have not closed.** A per-module rule *count* check — Rust
  reports how many rules it implements per module, and a mismatch against Python fails — belongs in
  `parity.py` before `check` stops delegating. Without it, the last module landing is not evidence
  that the port is complete.
- **Three tests in this unit write fixtures into temp directories**, so they exercise the rule logic but
  not the real registry's edge cases. The real-registry assertions are separate and do run.
- **`mistakes_have_guards` reads `status != "active"` as closed.** A lesson with a typo'd status
  (`Active`, `actve`) is silently treated as closed and skipped, so a misspelled status **disables the
  guard rule for that lesson**. The Python rule has the same behaviour; the port agrees with it, and
  the defect is recorded rather than quietly fixed in only one implementation.
