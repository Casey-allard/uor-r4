# #1246 — Connect addressed attention forward phases and validate complete frozen gradients

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T07:00:05Z
- **Updated:** 2026-10-09T13:17:05Z
- **Closed:** 2026-09-13T07:01:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1246
- **Merged:** 2026-09-13T07:01:46Z
- **Merge commit:** eefe40d2d8b819a536ce0cf5ae4c6e6bd73a0a43
- **Base / Head:** main / UOR-Foundation:codex/addressed-attention-forward
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The addressed-attention primitives lacked a complete forward caller, a bound model artifact and a measured full gradient path. Connect the eight query/read/control/emission/observation/key phases to canonical H4/prime/token/zeta tables and exact objects, with a strict artifact envelope, cached offers and snapshot continuation. Offline parameter/sampled policies compute direct CE plus every hard trajectory event score; final runtime uses integer/table operations. Existing retained dispatch is unchanged.

Local release validation: **29 focused tests pass**, including complete interpreter/export trace parity, target causality, overflow/ordered arithmetic, exact publication and snapshot handling. The separately invoked report test passes one frozen four-particle ×64-position dry-run:309,230µs,282,887 finite nonzero gradient entries,unchanged parameters,86,417-byte initialized artifact,31,930,368-byte dense arrays. It performs zero selected arithmetic; dedicated runtime tests cover that path. No optimizer update, language fit or promotion. Both original checkouts, retained15baec48 and21 previous seals/733 files verify. Formatting, claim wording and source/evidence/link checks pass. Queue compatibility statuses do not execute tests.

Build/test/report charge112,342/180,000ms; no allowance increase, paid compute or cleanup. General language/coding, steady-state latency, allocations and energy remain unqualified. Result and precise next learning pilot are in `docs/native_geometric_addressed_attention_forward_973.md`; tracked evidence binds source, binary, logs and the sealed report. Current state/canonical roadmap updated.

References #973. References #964. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

