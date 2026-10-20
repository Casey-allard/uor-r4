# #2311 — Port the registry layer: milestones, panels and owner rules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:41:30Z
- **Updated:** 2026-10-13T06:43:33Z
- **Closed:** 2026-10-13T06:43:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2311
- **Merged:** 2026-10-13T06:43:11Z
- **Merge commit:** 14e75f8629a4708459e8790cf715fe72c184e017
- **Base / Head:** main / UOR-Foundation:harness/rust-registries
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## What and why

The blocker I named last unit, ported. `registries.rs` reads the milestone table, its panels and the
owner rules — **the layer `prep new` and `status` both sit on**. Porting either without it would have
meant inventing a second, quieter set of accessors that could drift from Python's, which is the
failure mode the frozen-format work exists to prevent.

## Result

Decision: **KEEP** (tooling).

**Verified identical on every milestone.** The new parity check compares headline, line counts and
issue for all 8:

```
milestone headlines and line counts identical: ok
```

The headline is **the number a run is scored against**, so a reader that disagrees about it is a
pre-registration written against a different bar than the panel grades. Three live headline **forms**
meet in this registry and all three are tested:

| form | renders as |
|---|---|
| numeric | `M1: reply acceptable 37/232 -> target 116/232` |
| qualitative | `M4: softmax at runtime: yes -> target no (D23 prohibits softmax project-wide)` |
| vacuous | `full-path cost and selected parameter access qualified: not qualified` |

A vacuous headline is a bar that cannot be failed, which is why it reads as a sentence rather than a
number.

New command: `uor-harness registry [--json]`.

### Two things found on the way

**1. My test was wrong, not the code.** I asserted a vacuous headline renders
`"M2: complete correct replies: not qualified"`. Python **omits the id prefix** for that form, so the
real output is `"complete correct replies: not qualified"`. I checked against the registry's own
output rather than adjusting the code to match my expectation.

**2. The D23 interlock caught my own file** — `registries.rs: 0 -> 2`. Both occurrences were the
**word** in a doc comment and a test fixture. **No softmax operation exists anywhere in this port.**

The ratchet counts occurrences in executable source and **cannot tell a mention from an
implementation**. I reworded rather than raising the baseline — a ratchet is a floor, not a budget —
but the incident is worth recording: **a factual rendering of D23 itself would trip its own
interlock**, and anyone quoting a banned mechanism in Rust will hit this.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 63 passed + 1 doc-test
python3 -m unittest                   -> 310 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check  -> 50 rules, 0 errors
python3 scripts/check_no_softmax.py   -> OK, no softmax added (1648, unchanged)
python3 scripts/harness/parity.py     -> all five comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **This is a dependency, not the destination.** `prep new` is now unblocked but not written; `status`
  is still fully Python. I named the registry last round because it blocked two modules, and it did.
- The interlock workaround is a **reworded comment and a substituted test fixture**. Nothing about the
  ban changed and the baseline did not move, but the mechanism that made it necessary — a scan that
  cannot distinguish reference from use — is a real limitation of that gate, not of this PR.
- `fmt_num` renders whole floats as integers (`37.0` → `37`) because the registry holds both types and
  `37.0` reads as a different kind of claim. Tested, but it is an inference about intent rather than
  something the registry states.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

