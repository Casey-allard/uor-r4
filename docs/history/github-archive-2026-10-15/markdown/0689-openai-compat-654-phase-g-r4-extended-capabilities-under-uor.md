# #689 — OpenAI-compat #654 phase G: R4 extended capabilities under /uor/v1 (keep /v1 OpenAI-pure)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T02:31:51Z
- **Updated:** 2026-08-14T02:52:34Z
- **Closed:** 2026-08-14T02:52:34Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/689
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Sub-issue of #654 (phase G, final). Reframed per Casey: rather than REMOVING the R4-specific /v1 control routes, KEEP the capabilities but move them under a non-conflicting vendor namespace so /v1 stays a pure OpenAI surface.

The three R4-specific routes squatting on /v1 (status = 4-stage lifecycle readiness; reload = hot-swap the R4G1 runtime + teacher; corpus = manage the extra-reading corpus) become canonical under /uor/v1/*. The bare /v1/status, /v1/reload, /v1/corpus paths stay working as DEPRECATED ALIASES (RFC 8594 Deprecation header + Link to the /uor/v1 successor) — nothing is removed.

Namespace: /uor/v1/ (vendor-prefixed, Casey's choice) — collision-proof vs OpenAI (can't be mistaken for an OpenAI route or clash if OpenAI ships its own /v2), and versions independently.

All in src/server.rs + docs. Pure helpers extended_route_canonical / is_deprecated_v1_alias / deprecation_headers (unit-tested); send_json_response gains a send_json_response_ext(.., extra_headers) sibling for the Deprecation/Link headers. README + docs/minimal_client.md updated to split the OpenAI /v1 surface from the /uor/v1 extended API. The response extras (uor_audit/cascade_trail) stay on /v1 responses — non-breaking, SDKs ignore them.

Completes the #654 A–G decomposition. Deferred increment (optional, post-#654): streaming for /v1/responses.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

