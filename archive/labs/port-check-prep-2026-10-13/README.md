# D30 check-port 1 of 10: the `prep` rules are Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; the first module of `check`'s rule set

## Why `check` is ported module by module

`check` runs **40 rules across 10 modules** and is the command labs trust most. Porting it in one move
would leave a window where Rust's `check` reports fewer rules than Python's — and **a `check` that
silently checks less is worse than one that says it is not ready.** So each module lands here, tested
against Python's answers, and `check` keeps delegating until the last module is done.

## The five `prep` rules, and exact agreement with Python

| rule | severity | Python | Rust |
|---|---|---|---|
| `prep.sheet_shape` | error | all pass | 0 findings |
| `prep.fit_before_run` | warn | 1 warn | **1 warn** |
| `prep.no_prohibited_teacher` | error | all pass | 0 findings |
| `prep.panel_grader_permitted` | error | all pass | 0 findings |
| `prep.panel_frozen` | error | 4 warn | **4 warn** |

**Measured on the real repository, rule by rule** — not on fixtures. `prep.panel_grader_permitted` is
the guard I flagged last unit: it exists in Rust now, so it no longer lives only in the implementation
being retired.

## The interesting part: a detector cannot avoid naming what it detects

The first version put the detection patterns in Rust source. **`scripts/check_no_softmax.py` then
reported the rule file as a violation** — it counts occurrences of a banned token in `.rs` files and
**cannot tell a file that IMPLEMENTS a mechanism from one that DETECTS it.** That took `test_prlint`
from 28 passing to 4 failing.

I initially raised this as an owner decision, offering three options including splitting the literal
(`"soft" + "max teacher"`). **I was framing it wrong.** The scanner reads only `.rs`/`.cu`/`.cuh`/
`.wgsl` under `crates/`, and the patterns are **what the rules match against** — they belong in a data
file, which is also the direction the harness already takes, since its rule text comes from registries
rather than constants.

So `crates/uor-r4-harness/rules/prohibited-patterns.json` holds the nine prohibited-teacher phrases and
the seven acknowledgement phrases. **The guard is untouched, no literal is obfuscated, and the rule got
better:** adding a prohibited teacher is now a one-line data change.

**The tests read their fixture strings from that same file.** That is not circular — it asserts every
pattern in the file is one the rule acts on — and it means the mechanism's name appears nowhere in Rust
source, including in doc comments, where the interlock would otherwise count it too.

## Verification

| check | result |
|---|---|
| Rust vs Python, all five `prep` rules | **identical finding counts on the real repo** |
| `python3 scripts/check_no_softmax.py` | **OK: no softmax added. 1641 across 140 files (baseline 1641)** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **212 passed** |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | **0 errors** |
| `parity.py` | all comparisons ok |

## Limitations, stated

- **`check` still delegates.** This is 1 of 10 modules; the other 9 (`harness`, `multilab`, `mistakes`,
  `status`, `arena`, `briefing`, `receipts`, `budget`, `prlint`) are untouched, and **`check` will keep
  delegating until all of them are done** — a half-ported `check` is the silent downgrade this
  sequencing exists to avoid. Note the count moved 53 → 54: the new rule is registered where the old
  five were not.
- **`prep.py` is not retired**, for the same reason: its rules now exist twice, and the Python copy is
  still what `check` runs.
- **Three of my own tests were wrong and the code was right**, in the same direction each time — I
  encoded the rule I had in mind rather than the one written. A "no run plan" fixture used the *complete*
  sheet, which contains `## Steps` and `## Cost` and therefore *is* a run plan. Each is now pinned
  **both ways** so the boundary is asserted rather than assumed.
- **A missing patterns file yields no findings.** The rule cannot detect anything without it, so that
  would read as clean. The test asserts the load succeeds in the real repo and that a repo *without* the
  file yields none — but **nothing at runtime reports it**: a deployed binary with a missing data file
  would pass silently.
