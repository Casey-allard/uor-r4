# #1729 — mqar-bench fact layout: fall back to another value kind when a pool runs dry (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T07:33:31Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-10-05T07:34:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1729
- **Merged:** 2026-10-05T07:34:31Z
- **Merge commit:** 53abb049fbef6b6b0aa166b13ca8f21b3fec9fb5
- **Base / Head:** main / UOR-Foundation:codex/step2-fresh-value-fallback
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The bug:** on the pod, every Step 2 run failed with `invalid reference request: no fresh value`. Values must be unique within a sequence, and the numeral pool holds only 10 values, so a sequence that drew more than 10 numerals failed. This is rare per sequence, so the laptop smoke runs (few eval batches) passed, while long runs with many eval batches always hit it.

**The fix:** the drawn kind is tried first, with the same RNG calls as before, so sequences that never run dry are unchanged. Then the other kinds are tried in a fixed order, then a deterministic scan of each pool.

**Checks at head 29d2f7d15a96dfd87cc54f9b1e64a803d65b5c04:**
- rustfmt passes, and the example tests pass 14/14.
- Stress runs at seeds 1–3 (40 steps, an eval every 2 steps, 512 final sequences) all complete.
- On the second 4090 pod, the Step 2 calibration that failed within seconds now runs normally.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5990108479 — 2026-10-05T07:33:32Z — Casey-allard

Exact-head self-review at 29d2f7d15a96dfd87cc54f9b1e64a803d65b5c04: a one-function change in a bench example; the checks above ran at this head. Queuing.


---
_Comment/review ids in this file: 5990108479_

