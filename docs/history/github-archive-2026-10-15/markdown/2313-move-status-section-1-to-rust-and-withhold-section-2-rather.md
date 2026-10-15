# #2313 — Move status section 1 to Rust, and withhold section 2 rather than half-render it (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:54:55Z
- **Updated:** 2026-10-13T06:56:27Z
- **Closed:** 2026-10-13T06:56:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2313
- **Merged:** 2026-10-13T06:56:19Z
- **Merge commit:** 3e5aecbf3f153b78687d9f52e838d0897f45c02e
- **Base / Head:** main / UOR-Foundation:harness/rust-status
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## What and why

`status` is the largest remaining module at 1,863 lines, and half of it is GitHub reconciliation. This
ports the part every lab opens it for: **MILESTONES** — the headline acceptance number, the line
counts and the panel.

## Result

Decision: **KEEP** (tooling).

**Verified byte-identical to Python against its real `status` output, not a helper** — 41 lines:

```
status MILESTONES section identical: ok
```

What a lab reads is the *rendered view*, so that is what parity compares.

### Two details only a real comparison finds

- **`truncate` collapses whitespace** and strips the space a cut leaves. A naive cut ends `"word …"`
  where Python ends `"word…"` — and these fields come from JSON carrying newlines.
- **The registry's top-level `note` rides on the source line.** That's where a lab learns *which owner
  direction the cache was reconciled to*, before trusting anything below it.

### Section 2 is deliberately withheld — the main judgement here

Python's receipts section aggregates **all four labs' shards** and adds three 24-hour subsections:

- milestone issues updated within 24h
- milestones with no receipt in 24h
- **heartbeat cards emitted locally but never posted on the issue**

A single-lab version would read as a **complete answer while omitting three labs** — the worst
possible outcome for a section whose entire purpose is cross-lab visibility. So the renderer **states
that it is not ported**, `receipts_view` stays available as a tested single-lab API, and section 3 is
marked the same way. **An unported section must never read as "nothing to report."**

### Also: a real parser bug, found only by adding `--at`

The Rust argument parser accepted a flag only if it was in the **switch** list, so a value-only flag
was rejected with `unknown option` no matter which list it was added to. The two lists were consulted
as if the first were the complete set.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 74 passed + 1 doc-test
python3 -m unittest                   -> 310 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check  -> 51 rules, 0 errors
python3 scripts/check_no_softmax.py   -> OK (1648, unchanged)
python3 scripts/harness/parity.py     -> all eight comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **One of four sections is ported.** `status` is not a Rust command yet in any useful sense — it is a
  Rust command that renders section 1 correctly and tells you the rest is in Python. That is
  deliberate, but it is one fifth of the module.
- **Section 2's aggregation is the actual work**, and it is not a rendering problem: it needs a
  cross-shard reader and three 24-hour rules. Naming it as "next" twice now; it should be done rather
  than approached again.
- The parity comparison pins the timestamp (`--at`) so the two runs are comparable. A real `status`
  prints the current time, so the check compares a pinned render rather than a live one — it would not
  catch a bug that only appears at a different clock time.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

