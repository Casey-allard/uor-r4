# #1598 — Combine an op-model compiler with its saved table by a load-time policy (session open relation 41/52)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T03:30:26Z
- **Updated:** 2026-10-09T13:08:43Z
- **Closed:** 2026-10-02T04:09:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1598
- **Merged:** 2026-10-02T04:09:47Z
- **Merge commit:** d8ade11a505e360d2d76e8c0adb77cd7650bd3e5
- **Base / Head:** main / UOR-Foundation:codex/compiler-hybrid
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552 (D19).

`OpPolicy` (`op` | `table` | `table_statements` | `unless_query`) combines an op-model compiler with the table and span head its artifact already carries. It is a load-time choice and the artifact is unchanged. `m-world session op_policy=` records it in the report; `op` is the default, so existing behaviour is unchanged.

**Measured** (paired: same v25b artifact, emit-1 emitter, M-world v2 development, 300 conversations): open relation **30 → 41/52** with `unless_query`. That beats the previous best, the table compiler at 34/52, which the `table` policy reproduces exactly. `table_statements` scores 38/52. Full table and diagnosis: https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5945088798

**Checks.** The `relation_compiler` tests pass: 12, including the policy grammar and refusal of non-table policies on a compiler with no op model. Release build `b4759bfa` produced the sessions above. Evidence branch: `codex/ci/compiler-hybrid` (workflow v3).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5945436397 — 2026-10-02T04:08:53Z — Casey-allard

**Exact-head review: `b4759bfacdd776d0fcbc4e09668135f2ab7da2ba`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- `OpPolicy::Op` is the default, and `with_op_policy` is never called unless `op_policy=` is given, so existing compilers and sessions behave exactly as before. The `table` policy reproduces v21's 34/52 exactly.
- A table-only compiler refuses every policy except `op` and `table` (tested).
- `table_action` is the previous action body, unchanged.
- The session records the policy in its report.

**Evidence at this head:** fmt and the check of all targets succeeded; `uor-r4-training` lib 513 passed, 0 failed; examples pass. The PR merges cleanly with the current main.

**Measured:** best D19 configuration emit-3 × `unless_query`: open 44/52, closed 15/17, MQAR 42/109 (#1552).

Verdict: approve for merge at this head.


---
_Comment/review ids in this file: 5945436397_

