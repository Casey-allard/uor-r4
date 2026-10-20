# `check_pr` is Rust — the dispatcher that makes thirteen gates reachable

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the unit that turns verified gates into a working function

## What landed

**`prgates::check_pr`** — every gate, in Python's order, returning Python's shape: **`ok`**,
**`problems`**, **`summary`**, **`fields`**.

**Asserted against Python's own output for four bodies**, comparing **`ok`, the summary string, the
problem count, the ORDER, the severities, and the message and fix text.**

## Why the order is ported rather than chosen

**A reader meets the first problem first**, and the gates run *body declarations → registry
agreement → housekeeping*. **Re-ordering would not change `ok` and would change what a lab fixes
first**, so it is copied.

**And a dispatcher error is invisible to per-gate tests**: thirteen gates that each agree in isolation
can still run in the wrong order, or with one omitted, and every gate test passes. **That is the
failure this fixture set exists to catch**, and the comparison names a difference by position.

## Two inputs are parameters, for the reason `record_path` takes a `repo`

**`state_text`** is `current-state.md` and **`now`** is the local date. **A gate that reads the clock
or the filesystem itself cannot be tested at a fixed instant or against a fixture** — the defect this
session already fixed once in `parity`.

The fixture test passes **`None`** for the state text, which **proves the bodies do not consult it**
rather than assuming they do not.

## The finding: my own docstring tripped the interlock, and the fixture recorded the dirt

`check_no_softmax.py` failed on **`prgates.rs: 0 -> 1`** — **my `check_pr` docstring named the
prohibited mechanism**, in a sentence explaining that this function does *not* run the scan.

**Both internals reported the same file and the same count**, and I had initially misread the two
messages as disagreeing — **they were truncating differently, not diverging.** That is worth
recording because the first reading would have become a false bug report.

**The fixture had been captured while the tree was dirty**, so it encoded a finding that no longer
existed and the comparison failed for a reason unrelated to the port. **Re-capturing against a clean
tree fixed it, and the note is in the fixture file**: **a capture taken while the tree is dirty
encodes that dirt.**

**This is the fourth time in this port that a detector's own vocabulary tripped the guard it
enforces** — and the first time the *prose* was the cause rather than the code.

## Verification

| check | result |
|---|---|
| `ok`, summary, count, order, severity, text vs Python — 4 bodies | asserted, difference named by position |
| `ok` polarity: warnings and `info` are not errors | asserted |
| `cargo test -p uor-r4-harness` | **358 passed** (2 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **`lint` still delegates.** `check_pr` is **not reachable from any command** — the `lint`
  subcommand is Python, so **this unit makes the gates reachable from a function rather than from the
  CLI.** Dropping `lint` from the delegated set is the next unit, and **until it lands the objective's
  own measure has not moved.**
- **The four fixture bodies are hand-written.** They cover empty, minimal, complete and an unknown
  line, **not a real PR body**, and the `softmax_ban` message is compared only by its 60-character
  prefix **because it carries a live file count** — so that gate's count is not verified here.
- **`fields` is returned but not compared.** Python's `check_pr` returns the full parsed fields and
  **nothing asserts they survive the assembly**, so a dispatcher that returned `{}` would pass.
- **`parse` cannot be reached from `check_pr` either**: the Rust parser is total, so the one gate
  Python can raise first is the one Rust cannot — recorded in the module docstring rather than
  papered over.
