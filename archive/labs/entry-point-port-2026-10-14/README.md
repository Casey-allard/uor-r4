# `harness.entry_point` — and the half I deliberately did not port

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `check` port, after round 46 corrected my reason for deferring this pair

## What landed

**`harness.entry_point`** — no normative doc may invoke `python3 scripts/harness/cli.py`.

**Verified against Python on the real repo: 927 docs discovered, `AGENTS.md` first, 0 stale
invocations** — Python's own numbers.

Coverage: **49 of 54** — **3 of `check`'s own rules remain**, down from 5.

## And `harness.documented_commands` is deliberately NOT ported

**This is the one place in this port where I have declined to port a rule that is portable.**

Python validates a documented command by **building the live `argparse` parser and resolving the
documented words against it** — so it checks the **command, its subcommand and its flags**.

**A Rust version written without that introspection could only check the command word.** Shipping
that under the same rule id would be **a weaker check wearing the name of the real one** — the silent
downgrade this port refuses everywhere else. **So it stays in Python, and `doccheck.rs` says why in
its module docstring and in the inventory**, where a reader comparing coverage will find it.

**The difference matters because the two catch different mistakes.** A command-word check catches
`uor-harness statu`; it does not catch `uor-harness status --refreshh`. **The second is the one a lab
copying from a doc actually hits.**

## The exclusions, each ported for its own reason

| excluded | why |
|---|---|
| `docs/labs/<topic>-<date>/` | **evidence of what was run at the time; rewriting it would falsify the record** |
| `docs/integration/current-state.md` | **records commands, does not instruct anyone to run them** |
| `docs/history/`, `research/` | retained history |
| `crates/**/README.md` | **a README beside source is not an instruction set** |

**Excluding them is what stops the rule firing on the project's own record** — and **a rule that
fires on history is one people learn to ignore.** Each is asserted with a fixture carrying a Python
invocation that must NOT be reported.

## And coverage was under-reporting

Registering this rule moved the count **47 → 49**, not 47 → 48.

**`harness.owner_rules_applied` had been written, tested and shipped in an earlier unit but never
added to the inventory** — so coverage **under-reported by one while `parity` reported the rule as
"still in Python", and I believed it.** The rule that exists to stop owner rules being silently
omitted was itself silently omitted from the coverage record. **Found only by editing the same list
for a different reason.**

## Verification

| check | result |
|---|---|
| discovery vs Python on the real repo (927 docs, order) | asserted |
| each exclusion, by its stated reason | asserted, with an invocation that must not be reported |
| a normative Python invocation **is** an error, with Python's text | asserted |
| a clean repo reports the **success line**, not silence | asserted |
| `cargo test -p uor-r4-harness` | **373 passed** (4 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **49 implemented**, **3 of `check`'s own rules left** |

## Limitations, stated

- **`check` still delegates**, so the objective is not met. **Three of its own rules remain**:
  `harness.documented_commands` (above), `prlint.gates_declared`, and `budget.no_new_spending_class`
  — plus `status.registry_vs_github`, which `parity` counts outside `check`'s own set.
- **The docs are scanned for one exact string.** A doc invoking `cli.py` through an alias, a
  variable, or `sh scripts/harness/cli.py` is not caught — **Python has the same narrowness, and it
  is the reason the rule is a backstop rather than a proof.**
- ~~**`discover_docs` falls back to an empty list when git is unavailable.**~~ **FIXED in this same
  unit, after I first wrote it up as "not fixed".** The docstring *claimed* a walk fallback while the
  code returned an empty list, **so off git Rust found nothing and reported success while Python
  would still scan** — the "reports success because it looked at nothing" failure this port keeps
  finding. The fallback is implemented, skips `.git`/`target`/`node_modules` rather than filtering
  afterwards, and is asserted by a fixture **with no `git init` at all.**
  **I had written the limitation before checking whether it was one.**
- **The walk fallback is bounded by the three directory names it skips.** A checkout with a large
  non-standard build directory is walked, **so off-git discovery is slower than the index path — and
  it is the path nobody's CI takes, which is why it went unimplemented unnoticed.**
- **The success line is a finding-shaped result with `severity="warn"`.** That is Python's shape and
  the port keeps it, **but a reader counting warnings sees this rule contribute one on every clean
  run.**
