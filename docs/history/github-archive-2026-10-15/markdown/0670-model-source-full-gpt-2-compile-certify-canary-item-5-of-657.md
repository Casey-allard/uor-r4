# #670 — model-source: full GPT-2 compile→certify canary (item 5 of #657)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T17:48:42Z
- **Updated:** 2026-08-14T04:05:27Z
- **Closed:** 2026-08-14T04:05:27Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/670
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #657 (depends on items 2-4). Extend the presence-gated GPT-2 canary to the full pipeline: source conformance (#599) -> observation (#603 traces) -> compile -> artifact load -> teacher-forced Gate C parity -> runtime contract -> #606 certificate rows. Deterministic double-run of observation/artifact bytes; serialize the heavy run. Acceptance mirrors #657: a pinned GPT-2 source compiles to an R4G1 artifact end-to-end; SmolLM2 byte-unchanged; certificate carries GPT-2 source rows. Filed unassigned per board rules.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

