# #1113 — Implement contextual retained value writes

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T05:49:04Z
- **Updated:** 2026-09-04T05:51:17Z
- **Closed:** 2026-09-04T05:50:14Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1113
- **Merged:** 2026-09-04T05:50:14Z
- **Merge commit:** 29c772746c737792f8529fca97cc75ef5d2f0ea7
- **Base / Head:** main / UOR-Foundation:codex/973-retained-context-write
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained decoder still wrote only `Wv(RMSNorm(x_t))` into its identity-addressed value field, so prompt context recovered by the strict-prior retained read was never stored for later tokens. This adds a versioned, parameter-compatible `R4ContextualValueWriteLanguagePathV1` that writes `Wv(RMSNorm(x_t + a_t))`, where `a_t` is the ungated strict-prior retained residual, while preserving the historical V1 implementation and artifact shape. An arbitrary-prompt `generate-contextual-retained` command loads the existing fitted artifact and exercises the new direct recurrence without training or corpus access.

Direct checks:
- `A purple turtle found a clock in the garden` -> `. She was a little cat named Tom. The cat went with Sam. They`
- `Albert Einstein was born in` -> ` the dog. He said to the ground. They said, "Max, you`
- both invocations loaded artifact `blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d`, completed on CPU, and reported zero corpus/teacher file reads
- `git diff --check`

The second output is factually incoherent. These weights were fitted under the old token-only write, so this PR establishes executable source-free behavior for the changed write law, not improved language quality or a fitted contextual model.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

