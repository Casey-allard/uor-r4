# #1025 — Implement source-relative relation preflight for #954

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-31T22:49:04Z
- **Updated:** 2026-10-09T14:17:21Z
- **Closed:** 2026-08-31T22:50:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1025
- **Merged:** 2026-08-31T22:50:14Z
- **Merge commit:** 2b95417d0e6fdf468a04ebc5592c0a62a30a8242
- **Base / Head:** main / UOR-Foundation:issue-954-relation-entailment
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements the independently frozen C1-SB2 `R4SourceRelativeRelationHeadV1` mechanism and records its decision-bearing negative result.

The new seam:

- encodes each exact source sentence relative to the exact question through the immutable #1017 six-layer R4/Spin causal-softmax executor;
- retains the final question-token width-288 state;
- fits a fixed `288 -> 32 ReLU -> 1` relation probe;
- loads canonical self-addressed artifacts through strict Rust validation; and
- collapses exact duplicate text before fail-closed exact-copy, typed abstention, or typed conflict.

The Python campaign freezes 3,360 construction and 420 lexically disjoint development records, reversal/query-swap controls, four product commitments, and a mandatory 12-fit/12-sealed matched-transfer gate before Rust parity or the sole full fit.

## Observed decision

Terminal: `FAIL_MATCHED_TRANSFER_PREFLIGHT_STOP`.

- Fit: positive `12/12`, negative `20/20`, copied span `6/6`.
- Sealed transfer: positive `5/12`, negative `14/20`, copied span `0/6`.
- Sealed outcomes: answer `0/6`, abstain `3/4`, conflict `1/2`.
- Candidate-array order equivariance passed; same-source, query-swap, duplicate-agreement, and distinct-value conflict controls failed.
- Rust parity, the 512-step full fit, development evaluation, final-head emission, all four product probes, and browser/HTTP wiring are `NOT_RUN`.

No final relation head is qualified. This does not revoke established ordinary causal attention or #1017 bounded coherent generation. The next proposed #954 mechanism must train relation supervision into the representation through the existing R4/Spin attention path while retaining the exact-copy and typed non-answer Rust boundary.

## Focused verification

- 13 source-relation Python contract tests pass.
- 8 strict Rust artifact/scoring/decision tests pass.
- Grounded-answer policy test passes.
- Root library and binary compile pass.
- Targeted root clippy with warnings denied passes.
- CLI and `answer` help render the current boundary.
- Rust formatting, diff whitespace, JSON validity, and claim-wording gate pass.

The broad recurring suite was intentionally not run locally because the frozen preflight already made the scientific decision; the protected merge queue remains the repository-wide integration verdict.

References #954


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

