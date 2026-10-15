# #2314 — Aggregate every lab's shard in status section 2, and drop --lab (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T07:02:09Z
- **Updated:** 2026-10-13T07:04:28Z
- **Closed:** 2026-10-13T07:04:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2314
- **Merged:** 2026-10-13T07:04:07Z
- **Merge commit:** de1229bcefeb1f112b3fe594ff6393b8e917733a
- **Base / Head:** main / UOR-Foundation:harness/rust-status-receipts
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Result

Decision: **KEEP** (tooling).

The section I named twice and then did. `status` now reads **every lab's shard** — claude, codex, deepseek,
kimi and the legacy unsharded files — and renders the aggregate: per-kind counts for the day and month,
the last five receipts, skills used and agents dispatched, the milestones with no receipt in 24h, and
**the heartbeat cards emitted locally but never posted on the issue.**

That last clause is the one that catches a lab checking in where only its own machine can see it —
and it is exactly why this section **cannot** be rendered from one shard.

```
status RECEIPTS aggregation identical: ok
```

### Two clauses legitimately differ, and are excluded explicitly rather than tolerated

- the newest receipt's **live timestamp**, which moves between two runs; and
- the **milestone-issue freshness** clause, which the Rust side reports `UNAVAILABLE` because it does
  not query GitHub. Claiming the issues are current without looking would be the worse bug, so
  *"we did not look"* is stated rather than papered over.

### `--lab` is gone, and that's the sign the section is right

A cross-lab view has no *"whose receipts"* question, so the parameter became unused and was **removed**
rather than left as decoration. `ReceiptsContext` groups the five date/boolean arguments, because a
positional call site that swaps two `&str` dates **compiles and reports the wrong month**.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 81 passed + 1 doc-test
python3 -m unittest                   -> 310 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check  -> 51 rules, 0 errors
python3 scripts/harness/parity.py     -> all nine comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

**Most of this unit went on recovering from my own scripted source edits** — a brace-counting slice
that cut `previous_month` in half, and an ambiguous test assertion whose `"not ported"` check also
matched section 3's wording, so it failed against a *correctly* rendered section. **Line-range surgery
on formatted Rust failed three times in this session** and a plain edit with an immediate build would
have been faster every time. Recorded in the current-state entry, not just here.

- Still not ported: **section 3 (delivery: PRs, merge queue)** and **section 4 (GitHub)**. This is two
  of four.
- The GitHub reconciliation the objective's clause (a) depends on is still Python-only, so the Rust
  `status` cannot yet be a lab's only status command.
- A test asserting *"the aggregate exceeds one lab's shard"* is weak evidence of cross-lab reading — it
  would also pass if a second lab's shard were empty. The stronger check is the parity comparison
  against Python's own aggregate, which is what actually pins it.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

