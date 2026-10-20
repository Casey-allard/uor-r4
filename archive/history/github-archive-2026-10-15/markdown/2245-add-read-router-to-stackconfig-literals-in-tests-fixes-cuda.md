# #2245 — Add read_router to StackConfig literals in tests (fixes CUDA parity compile) (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T07:10:05Z
- **Updated:** 2026-10-12T07:13:54Z
- **Closed:** 2026-10-12T07:13:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2245
- **Merged:** 2026-10-12T07:13:53Z
- **Merge commit:** e1742aac74b34028ad3b48ac35099e313047e0dd
- **Base / Head:** main / UOR-Foundation:deepseek/invariant-router
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #2243: the bulk field-add missed the tests/ directory, so the two StackConfig literals in tests/cuda_stack_ops_parity.rs (and d4_map_codec_adapter.rs, temporal_cli.rs) were missing read_router: None. This only compiled under --features cuda, so CI (which does not compile cuda) never caught it; the pod bootstrap parity check (cargo test --test cuda_stack_ops_parity) failed to compile with E0063 missing field read_router.

Fix: add read_router: None to the 13 StackConfig literals across the three test files. cargo check --all-targets passes (non-cuda); the cuda compile is re-verified on the pod bootstrap.

References #2029 (never Closes).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

