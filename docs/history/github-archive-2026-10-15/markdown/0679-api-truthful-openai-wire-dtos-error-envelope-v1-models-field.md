# #679 — api: truthful OpenAI wire DTOs + error envelope + /v1/models fields (#654 phase B)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T23:26:11Z
- **Updated:** 2026-08-13T23:54:24Z
- **Closed:** 2026-08-13T23:54:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/679
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Phase B of #654 (builds on phase A pin/matrix #677/PR #678). Concrete server-code slice: (1) the standard OpenAI error envelope {error:{message,type,param,code}} + HTTP status mapping, replacing the ad-hoc {"error":"..."} bodies on the /v1 compatibility routes; (2) GET /v1/models required fields (id, object, created, owned_by) advertising only loadable models; (3) GET /v1/models/{model} agreeing with the list, 404 envelope on unknown; (4) unsupported/unknown chat params fail closed by name (deny_unknown_fields) with the envelope, never silently ignored. No streaming/responses/usage-accounting yet (phases C-E). Refs #654.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

