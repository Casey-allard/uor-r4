# #978 — #953: record natural-agreement support hard stop

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-27T23:19:54Z
- **Updated:** 2026-10-09T14:16:57Z
- **Closed:** 2026-08-27T23:21:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/978
- **Merged:** 2026-08-27T23:21:03Z
- **Merge commit:** 950c90b96ef3870637a6028614c9c4318a35a84b
- **Base / Head:** main / UOR-Foundation:issue-953-natural-grammar-revision
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

The independently motivated natural-agreement revision was frozen before selection and stopped at its support-only preflight. The current seven-row union admits adjacent-spin fallback even when direct-plus-divisor support is non-empty:

- step 1 expected 3 entries and `{still}`; observed 8 entries and `{athlete,generally,run,runs,still}`;
- step 2 expected 6 entries and `{run,runs}`; observed 11 entries and the same five-candidate union;
- left/right support and work steps were identical, no ceiling truncation occurred, and one adjacent-spin row hit at each step;
- support record: `blake3:70375921e267b5ceff2198f879356cfb42dd6907accc0c2b720fc8b89b59b271`;
- terminal: `REVISE_I1_GENERATOR_IN_PLACE`.

H4 path costs, selection, decode/append/cycle/termination, the four full/disabled arms, and final replay are `NOT_RUN_SUPPORT_PREFLIGHT_HARD_STOP`. The ignored four-arm witness now recomputes and enforces the support gate before generator construction, so an explicit invocation cannot bypass the stop.

## Frozen identities

- fixture: `blake3:0e018c9bcd43a29ed6f043665b2646c9579dd31d881d331f198fb89543184259`
- codec: `blake3:6db64540ef344562903e01adac102f7bcc96c65908d162b1deca9b83550b35ed`
- vocabulary: `blake3:3b74f7ace425c039b4eab751b400f2603d92baf4ccfc9f4b8ac9409446291b58`
- artifact: `blake3:b222510ccc01ed3257c8b38b743ca771f5e60c87ebf12c565f92fadbbd00332d`
- attention manifest: `blake3:1c3baf432b9fdcf2f3d90014797a5cae5850c0acba2fda63e0d6b659d49562de`

## Focused verification

- natural-agreement target: 2 passed, 1 ignored with the hard-stop literal;
- existing #953 relabel regression: passed;
- #969 decoded causal-path regression: passed;
- #958 attention regressions: 8 passed;
- focused core check, WASM library check, formatting, diff whitespace, and claim wording: passed;
- focused strict-clippy initially reached unchanged main-line lint debt outside this diff; the changed target passed with only those five pre-existing lint categories allowed.

Workspace-wide tests, corpus/model/teacher work, generation canaries, BDD, Gate C, kappa reproduction, audit, fuzz, formal proof, conformance, product QA, and release qualification were not run. Required workflow checks are dormant-QA transport acknowledgements, not verification evidence.

## Claim boundary and next hypothesis

This is an admission/indexing result, not an H4-ranking, grammar, coherent-generation, semantic, correctness, attention-general, or reasoning result. The smallest recorded next hypothesis is a visible policy tier with I1/I2/ordered-sentence plus divisor as primary support, consulting adjacent-spin only when that primary tier is empty. It is deliberately not implemented here; the trigram/Markov path remains dormant.

References #953


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

