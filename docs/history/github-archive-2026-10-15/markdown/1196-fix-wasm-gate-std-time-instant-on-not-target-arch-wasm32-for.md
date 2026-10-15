# #1196 — fix(wasm): gate std::time::Instant on not(target_arch = "wasm32") for in-browser inference

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T06:36:27Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-09-08T06:37:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1196
- **Merged:** 2026-09-08T06:37:30Z
- **Merge commit:** 190f133f50f0159dfaae33eb81199dc79d7a6eda
- **Base / Head:** main / UOR-Foundation:fix-wasm-time
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Gates `std::time::Instant` in `native_capability_api.rs` to fix `time not implemented on this platform` unreachable panic in in-browser WASM execution.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

