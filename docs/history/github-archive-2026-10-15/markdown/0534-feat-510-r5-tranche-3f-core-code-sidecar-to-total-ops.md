# #534 — feat(#510): R5 tranche 3f — core code_sidecar to total ops

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T07:59:15Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-09T08:18:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/534
- **Merged:** 2026-08-09T08:18:35Z
- **Merge commit:** 8b9de8981bb263cb4bb362fc62d0ffd48177193b
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t3f-sidecar
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Continues **core** (issue #510): the per-record graded-code sidecar (`uor-r4-core::transformerless::code_sidecar`).

- `parse_sidecar` → `Option<Vec<[u8; STAGES]>>`. A sidecar either verifies into the exact code block or it does not; magic / version / stages / record-count / κ / digest are all properties of the bytes on disk, so the eight-variant `SidecarReject` enum is removed and every check yields `None`. `load_codes` still logs a greppable MISS line with the path and κs.
- `build_store_cached` (wasm + native) → `(Store, Vec<[u8; STAGES]>)`. After #528 made `codes_with_threads` total, this had no remaining error path. The graph-cli caller drops its `?`.

Tests: the per-variant rejection assertions become `is_none()` — the safety property (a stale/foreign/truncated/corrupted sidecar never loads) is preserved; the variant distinction is the R5 diagnostic trade-off.

`audit-limits`: `code_sidecar.rs` reports **zero** unsanctioned errors. Build / clippy (`-D warnings`) / fmt clean; the code_sidecar test target compiles and is behavior-preserving.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

