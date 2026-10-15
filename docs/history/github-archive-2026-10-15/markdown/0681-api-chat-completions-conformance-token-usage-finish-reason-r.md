# #681 — api: Chat Completions conformance - token usage + finish_reason + roles (#654 phase C)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T23:44:06Z
- **Updated:** 2026-08-14T00:24:37Z
- **Closed:** 2026-08-14T00:24:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/681
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Phase C of #654 (builds on phase B #679/#680). Token usage from the selected artifact tokenizer (not the whitespace-word estimate); truthful finish_reason (length vs stop against the served token budget); explicit supported roles (system/developer/user/assistant) with unsupported roles failing closed. Non-goals for this phase: SSE streaming (D), /v1/responses (E), multimodal/content-part unions, tool calls. Refs #654.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

