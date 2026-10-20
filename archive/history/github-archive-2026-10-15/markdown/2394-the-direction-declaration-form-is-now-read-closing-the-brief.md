# #2394 — The DIRECTION: declaration form is now read, closing the briefing gap (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T23:08:43Z
- **Updated:** 2026-10-13T23:20:03Z
- **Closed:** 2026-10-13T23:10:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2394
- **Merged:** 2026-10-13T23:10:56Z
- **Merge commit:** 81c4317a375385e567fe193f9c67b96915cffd5b
- **Base / Head:** main / UOR-Foundation:deepseek/direction-form
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Closing the `DIRECTION:` gap I flagged when porting `briefing`.** Tooling only.

Decision: **KEEP** (tooling).

## The gap, in my own words from that unit

> **`declared_lines` reads only the `Line "<name>"` form. The Python also matches a second form (a `DIRECTION:` prefix on a lab prompt) which this port does not implement — so a line declared only in a `DIRECTION:` block would be invisible to the Rust rule while Python sees it.**

**Now closed — and there were two differences, the second being the actual defect.**

## 1. The `DIRECTION:` form

Python matches it and **labels the finding** `"lab prompt DIRECTION"`. Rust now does both — **the label matters because it is what a finding cites**, and a line found via a lab prompt should say so rather than point the reader at the contract.

## 2. `Line` is matched *anywhere* — and this was the real defect

My parser required a **word boundary** before `Line`; **Python's regex does not.** So my matcher was **strictly narrower in a way that could only ever find *fewer* declarations** — and **fewer declarations reads as agreement.**

**That is the shape of every silent-guard failure in this port: the check reports clean because it looked at less.** A `DIRECTION:` declaration would have been missed **twice over** — once by the form, once by the boundary rule, since `DIRECTION: … Line "x"` has a non-boundary `Line`.

## Measured against the live file

```
declared lines: 1
  "session-goal contract" <the line the contract names> 1/3
lines_agree: 0 finding(s)          (Python: all pass)
```

**The `DIRECTION:` form has no instances in the live file today — which is exactly why the gap was latent: both implementations agreed because they agreed on zero.** **A latent gap that agrees is the most dangerous kind, because it looks like a passing check.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 318 passed (3 new)
python3 -m unittest discover -s tests/harness   -> 322 passed
parity.py                                       -> exit 0
```

Record: `docs/labs/briefing-direction-form-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **The gap was latent; this is the fix, not a behavioural change on today's file.** The two implementations produced **identical results before and after**, so **the tests are the only evidence the fix works — not a count that moved.**
- **The `where_` label is inferred from the line prefix**, not by matching Python's regex structure. A declaration whose `DIRECTION:` sits on a *previous* line would be labelled as the contract form. The behaviours agree, **but by two different arguments rather than by construction.**
- **`declared_lines` still matches `Line` inside prose.** Python has the same looseness; the `count n/m` requirement is what keeps it from firing.
- **`check` still delegates** — this port is rule-level, not command-level.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

