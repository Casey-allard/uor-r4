# #2387 — D30 check-port: harness.state_formats, and the skipped hole it closes (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T22:13:08Z
- **Updated:** 2026-10-13T22:15:36Z
- **Closed:** 2026-10-13T22:15:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2387
- **Merged:** 2026-10-13T22:15:13Z
- **Merge commit:** 241b5710d634c5bce5a1ff941b757f78b843fced
- **Base / Head:** main / UOR-Foundation:deepseek/state-formats
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port: `harness.state_formats`.** Tooling only.

Decision: **KEEP** (tooling).

## Agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `harness.state_formats` | error | all pass | 0 findings |

Coverage reads **31 of 40 rules.**

## This rule was Rust before it was ported — only the translation was Python

The Python rule **shells out to `uor-harness check-formats`** and turns the exit code into a finding. **So the check itself was Rust the whole time**, and this port removes a **subprocess**, not a duplicate scan.

The rule now calls `Contract::check` **in process** — the same code `check-formats` runs. No binary to locate, no timeout, and **no chance of reporting a different build's answer than the one under test.** That is not hypothetical: `check-formats` exists because naming the Rust `check` produced **infinite recursion** until a timeout fired.

## The `skipped` status was a hole, and Rust cannot fall into it

The Python has three outcomes: `ok`, drift, and **`skipped` when the Rust binary is not built.** **The skipped case is `info` with `ok=True`** — so it does not fail, and **a contract check that did not run reads as acceptable.**

**That state cannot arise in Rust, because the rule IS the binary.** An unloadable contract is an **error**:

> *the frozen contract could not be loaded, so nothing was checked*

**The port closes the gap by having no such state to enter.** Previously, a machine without the binary got a green `check` on the one rule whose whole job is to notice that persisted state drifted from its frozen contract.

## One finding per format, not one per contract

**A single finding for several drifted formats sends the reader to the wrong file** — and the whole point of these rules is to name the thing that needs fixing.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 304 passed (3 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
python3 scripts/check_no_softmax.py             -> OK, no softmax added
parity.py                                       -> exit 0, rule inventory: 31 implemented
```

Record: `docs/labs/port-check-state-formats-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** A complete module set does not mean the *rules* are all done.
- **The one-finding-per-format change is a deliberate divergence from Python's shape.** The counts agree on the real repo (0 and 0), but **a drifted contract would produce a different *number* of findings** in the two implementations. Stated rather than presented as parity.
- **`Contract::load` failing is an error where Python would say `skipped`.** On a machine with a missing contract file, **the two implementations now disagree by design** — and the Rust answer is the one that does not hide a gap.
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule counts.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

