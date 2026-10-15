# #1135 — Evaluate shared binding and preserve the wording-transfer negative

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-05T19:24:51Z
- **Updated:** 2026-10-09T13:17:40Z
- **Closed:** 2026-09-05T19:49:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1135
- **Merged:** 2026-09-05T19:49:35Z
- **Merge commit:** 647fd532c70d0e8b0257438912d6bf54ad771683
- **Base / Head:** main / UOR-Foundation:codex/native-binding-transfer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Conversational formatting and question wording can make the native model answer “Unknown” even when the requested value remains available for copying. Add an optional experiment that supplies the existing occurrence binding-mask/preceding-word feature to response entry, plus a matched fit without copy-extension geometry. **The experiment fails promotion; the previous d095a1ab artifact remains selected.**

Entry uses at most 32 sparse features and 16 lexical candidates, with no new persistent state or dense inference. The existing integer/table selector, immutable byte cursor, completion frame and typed precedence remain. Older artifacts omit the new flags. The shared sparse fitter and row scratch accept a compile-time bound so other completion callers retain their existing bounds.

The Rust preparation path adds 96 construction examples and saves a 16-case matched diagnostic and 32 reserved transfer cases. All required fact words in the reserved cases fit the existing sixteen-word capture. Labels never enter inference. Sources, artifact identities, outputs and complete cost vectors are recorded in the existing evidence record and local `wording-checkpoint.json`.

| Variant | Matched diagnostic | Original responses | Prior facts | Reserved transfer |
|---|---:|---:|---:|---:|
| Selected parent | 4/16 | 32/32 | 16/16 | 3/32 |
| More coverage, same mechanism | 12/16 | 28/32 | 16/16 | 0/32 |
| Shared binding | 12/16 | 32/32 | 12/16 | 8/32 |
| Matched fit without copy geometry | Not run | 32/32 | 12/16 | 8/32 |

Both shared fits get only the eight unsupported reserved questions correct: 0/24 supported complete answers. Both preserve 12/12 identifier and 8/8 binding responses and still fail the two older city cases. Reserved results were opened after design selection and did not tune the implementation. No geometric advantage is established. Zero-match contributions from unknown dictionary addresses are a concrete remaining hypothesis, not a completed correction.

Validation: 109 native unit tests passed. The new allocation fixture caught a second legacy validator rejecting the new serialized feature kind; after correction, 12 affected copy tests, the allocation fixture and all six preparation tests pass. Formatting, architecture policy and claim wording pass. One actual CLI run matches evaluator fields exactly. All 32 generated Rust source hashes match prior compilation/execution receipts. Broader release checks were not run.

On the shared fit's same 62 cases, dispatch preserves all response bytes and causal final states: complete generation 50.597 vs 65.275 ms in one pass; ordinary score lookups 1,619,990 vs 1,867,776; memory 294,190 vs 418,735; copy 90,651 in both. Failures are included. This retains conditional-execution savings, not a new model-quality or geometry-specific efficiency gain.

Cycle resources: 33.622/120 seconds model work; 1,104.632/1,200 seconds engineering, including the failed check. Cumulative model work 1,058.856/1,800 seconds. Peak sampled model RSS 761,085,952 bytes. The preauthorized 256 MiB storage increment carries the cumulative ceiling to 5,653,921,792 bytes with the existing stop margin. No deletion or external compute.

Refs #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

