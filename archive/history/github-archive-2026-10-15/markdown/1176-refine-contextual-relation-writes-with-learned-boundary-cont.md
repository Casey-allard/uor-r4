# #1176 — Refine contextual relation writes with learned boundary context

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T17:56:02Z
- **Updated:** 2026-10-09T13:17:11Z
- **Closed:** 2026-09-07T17:57:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1176
- **Merged:** 2026-09-07T17:57:31Z
- **Merge commit:** 4f9d02a29ceb0f4ca56ff174169d26cb5c6a518f
- **Base / Head:** main / UOR-Foundation:codex/contextual-writer-refinement
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained model had no writer record for two `quiet river` phrases. Continuing its sparse writer fixed those writes but exposed an earlier cross-boundary false record in object phrases ending in `holds`. Add learned endpoint-gap context and a matching exact NoWrite cache guard; selected artifact `79710468` repairs the phrase writes and preserves the accepted outputs.

The Rust fitter warm-starts all existing rows, freezes the dictionary/role geometry/candidate support, retains the best complete epoch including epoch zero, and reconstructs the exact parent from its witness. Optional boundary fields preserve older artifact identities. Serving uses the existing bounded integer/table path; no parser, hard sentence-admission rule or matrix product is added. Cache certification replays complete prime and gap signatures, with periodic proposals disabled for the new law.

Actual behavior:

- 232/232 construction writer labels and 32/32 new answers; 12/12 open and 76/76 prior phrase answers/writes.
- 200/200 original anchor outputs/writes match the parent. Both satisfy 159/200 historical answer targets; the 41 preexisting reader failures remain recorded.
- Selection precedes 12/12 fresh evicted phrases and 4/4 short boundary objects. New boundary construction/fresh cases directly have zero writes.
- All 663 retained answers and all selected preservation pass. Preserve the coefficient-only candidate's eight span regressions and the matched diagnostic.

Validation: pinned offline release probe/CLI and focused test builds; 4 warm-start, 8 relation/cache and 1 integer-kernel source tests; actual-artifact malformed-parent/dictionary/cache rejection, every-step checkpoints and zero allocations; selected and older coefficient-only artifact CLI outputs; formatting, architecture-policy, claim-wording and diff checks. Required PR/merge-group statuses are transport acknowledgements, not these local test results.

The cycle charged 292.555 model seconds and 1,254.788 engineering-command seconds. Cumulative model use is 5,163.298/5,250 seconds. Necessary extensions were recorded before use, with all previous charges and the 128 MiB storage stop margin preserved. No unique deletion or paid external compute occurred. No isolated feature ablation, general punctuation/correctness, object persistence outside the raw window, or speed/energy advantage is claimed. The next model work is a shared causal transition whose second operator consumes the first committed result.

References #1139, #1140, #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

