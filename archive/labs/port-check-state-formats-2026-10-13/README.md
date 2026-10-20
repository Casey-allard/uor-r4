# D30 check-port: `harness.state_formats` — the rule was already half-Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028

## Agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `harness.state_formats` | error | all pass | 0 findings |

Coverage reads **31 of 40 rules.**

## This rule was Rust before it was ported — only the translation was Python

The Python rule **shells out to `uor-harness check-formats`** and turns the exit code into a finding.
**So the check itself was Rust the whole time**, and what this port removes is a **subprocess**, not a
duplicate scan.

The rule now calls `Contract::check` **in process** — the same code `check-formats` runs. No binary to
locate, no timeout, and **no chance of reporting a different build's answer than the one under test.**
That last one is not hypothetical: `check-formats` was added specifically because Python's `check`
delegated back to the Rust `check`, and naming the wrong command produced **infinite recursion** until
a timeout fired.

## The `skipped` status was a hole, and Rust cannot fall into it

The Python has three outcomes: `ok`, drift, and **`skipped`** when the Rust binary is not built. **The
skipped case is reported as `info` with `ok=True`** — so it does not fail, and **a contract check that
did not run reads as acceptable.**

**That state cannot arise in Rust, because the rule IS the binary.** There is no "not built" to skip
into, so a contract that will not load is an **error**:

> *the frozen contract could not be loaded, so nothing was checked*

**The Python's `skipped` was a real gap** — a machine without the binary got a green `check` on the one
rule whose whole job is to notice that persisted state drifted from its frozen contract. **The port
closes it by having no such state to enter.**

## One finding per format, not one per contract

The Python returns a single finding for a drifted run. This returns **one per format**, because **a
single finding for several drifted formats sends the reader to the wrong file** — and the whole point
of these rules is to name the thing that needs fixing.

## Verification

| check | result |
|---|---|
| Rust vs Python | **0 findings; Python all pass** |
| the real repo matches its own contract | asserted by test |
| a missing contract is an error | asserted by test |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **304 passed** (3 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | exit 0, **31 implemented** |

## Limitations, stated

- **`check` still delegates.** Three rules remain unported plus this one's siblings; the module set
  complete at 10 of 10 does not mean the *rules* are all done.
- **The one-finding-per-format change is a divergence from Python's shape.** The counts agree on the
  real repo (0 and 0), but **a drifted contract would produce a different number of findings** in the
  two implementations. That is deliberate and is stated rather than presented as parity.
- **`Contract::load` failing is reported as an error where Python would say `skipped`.** On a machine
  where the contract file is missing, **the two implementations now disagree by design** — and the
  Rust answer is the one that does not hide a gap.
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule counts.
