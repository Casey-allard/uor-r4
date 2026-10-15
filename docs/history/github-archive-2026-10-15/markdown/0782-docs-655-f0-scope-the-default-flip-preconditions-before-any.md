# #782 — docs(#655-F0): scope the default-flip preconditions before any flip

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T04:52:14Z
- **Updated:** 2026-08-18T05:13:41Z
- **Closed:** 2026-08-18T05:13:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/782
- **Merged:** 2026-08-18T05:13:21Z
- **Merge commit:** aea30bae469db756272a03863b191a0f48598f50
- **Base / Head:** main / UOR-Foundation:feat/655-f0-default-flip-scoping
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Scopes #655-F ("flip the public default + docs") before any code or
public-doc changes — a design/decision record only, mirroring how
C1b/D0/E0 scoped their own tracks. Written after you authorized
proceeding with F and, when asked how, chose "scoping doc first" over
an immediate flip.

**Central finding**: F's own text (decomposition paragraph + activation
contract + scope + acceptance criteria, all quoted verbatim in the doc)
gates the flip behind two preconditions that are unmet today,
independent of the sign-off process:

1. **No distributed default bundle.** `.uor-models/` is gitignored;
   nothing ships with the repo/release. Distribution was explicitly
   spun out as #741 and deferred until model-quality work lands.
2. **Quality canary not met at baseline.** The most recent recorded
   assessment on #655 itself (after #759/#762 closed) found baseline
   generation still word-salad/degenerate on the best locally available
   bundle, concluding a flip now "would be premature on the merits,
   independent of the process gate." Nothing since has revisited that.

The doc also separates two things F's text conflates: tier reachability
(already effectively r4g1-only by default since #655-E2 — done, no F
work needed there) versus canonical model identity (the `uor-r4` alias
vs `r4`, F's actual remaining, still-unscoped question).

**Proposed sequence**: this record now; #741 + a fresh quality
measurement (or defaulting #762's opt-in mitigation) before anything
else; then the identity-rename slice itself still needs its own
explicit confirmation once preconditions clear — today's "proceed with
F" doesn't pre-authorize the flip itself once those preconditions are
met; that needs you looking at the cleared preconditions directly.

No behavior change. No README/RESEARCH/MODEL_LIFECYCLE/config/
threat-model edits (would describe an end state that isn't true yet).

## Test plan

- [x] `git diff --stat` confined to the one new doc file
- [x] `cargo check -p uor-r4-wasm-router --lib` — clean
- [x] R1 (`check-model`), R2/R3 (`repo-conformance`), R4 (`audit-deferral`), R5 (`audit-limits`), R6 (`cargo deny check`) — all pass

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

