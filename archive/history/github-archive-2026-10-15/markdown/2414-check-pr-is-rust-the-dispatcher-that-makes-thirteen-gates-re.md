# #2414 — check_pr is Rust: the dispatcher that makes thirteen gates reachable (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T02:05:51Z
- **Updated:** 2026-10-14T02:08:24Z
- **Closed:** 2026-10-14T02:08:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2414
- **Merged:** 2026-10-14T02:08:01Z
- **Merge commit:** 66095895f4ae9f570b39a1f01f852fda1f99c346
- **Base / Head:** main / UOR-Foundation:deepseek/check-pr
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`check_pr` is Rust — the dispatcher that makes thirteen gates reachable.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`prgates::check_pr`** — every gate in Python's order, returning Python's shape: **`ok`**, **`problems`**, **`summary`**, **`fields`**.

**Asserted against Python's own output for four bodies**, comparing **`ok`, the summary string, the count, the ORDER, the severities, and the message and fix text.**

## Why the order is ported rather than chosen

A reader meets the first problem first, and the gates run *body declarations → registry agreement → housekeeping*. **Re-ordering would not change `ok` and would change what a lab fixes first.**

**And a dispatcher error is invisible to per-gate tests**: thirteen gates that each agree in isolation can still run in the wrong order, or with one omitted. **That is the failure this fixture set exists to catch** — and the comparison names a difference **by position**.

## Two inputs are parameters

**`state_text`** and **`now`** — because **a gate that reads the clock or the filesystem itself cannot be tested at a fixed instant or against a fixture**, the defect this session already fixed once in `parity`. The test passes `None` for the state text, **which proves the bodies don't consult it** rather than assuming they don't.

## The finding: my own docstring tripped the interlock

The interlock reported **`prgates.rs` newly above baseline** — **my `check_pr` docstring named the prohibited mechanism**, in a sentence explaining that this function does *not* perform the scan.

**Both internals reported the same file and count, and I had initially misread the two messages as disagreeing — they were truncating differently, not diverging.** Worth recording, because the first reading would have become a **false bug report**.

**And the fixture had been captured while the tree was dirty**, so it encoded a finding that no longer existed. **Re-capturing against a clean tree fixed it, with the note in the fixture file: a capture taken while the tree is dirty encodes that dirt.**

**Fourth time in this port that a detector's own vocabulary tripped the guard it enforces — and the first time the *prose* was the cause.**

```
cargo test -p uor-r4-harness                    -> 358 passed (2 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/prlint-check-pr-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`lint` still delegates, so `check_pr` is not reachable from any command.** This makes the gates reachable **from a function rather than from the CLI** — and **until `lint` lands, the objective's own measure has not moved.**
- **The four fixture bodies are hand-written**, not a real PR body, and **the D23 message is compared only by its 60-character prefix** because it carries a live file count — **so that gate's count is not verified here.**
- **`fields` is returned but not compared.** Nothing asserts the parsed fields survive the assembly, so **a dispatcher returning `{}` would pass.**
- **`parse` is unreachable from `check_pr` too** — the Rust parser is total, so the one gate Python can raise first is the one Rust cannot.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

