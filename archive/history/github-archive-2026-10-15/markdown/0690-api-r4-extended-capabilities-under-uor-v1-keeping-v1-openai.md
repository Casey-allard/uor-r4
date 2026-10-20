# #690 — api: R4 extended capabilities under /uor/v1, keeping /v1 OpenAI-pure (#654 phase G)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T02:32:50Z
- **Updated:** 2026-10-09T14:17:08Z
- **Closed:** 2026-08-14T02:52:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/690
- **Merged:** 2026-08-14T02:52:33Z
- **Merge commit:** 765405ba7c50254df98ee1f2c0b8e8f40194e784
- **Base / Head:** main / UOR-Foundation:i654g-uor-extended-namespace
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #654; closes #689 (phase G — the final #654 slice). Builds on phase F (#688, merged).

Reframed per your steer: rather than **removing** the R4-specific `/v1` control routes, **keep the capabilities** but move them under a non-conflicting vendor namespace so `/v1` stays a pure OpenAI surface.

## What
The three R4-specific routes that were squatting on `/v1` — `status` (4-stage lifecycle readiness), `reload` (hot-swap the R4G1 runtime + teacher), `corpus` (manage the extra-reading corpus) — become canonical under **`/uor/v1/*`**. The bare `/v1/status`, `/v1/reload`, `/v1/corpus` paths **stay working as deprecated aliases**: they respond with an RFC 8594 `Deprecation` header and a `Link` to the `/uor/v1` successor. Nothing removed.

`/uor/v1/` is vendor-prefixed (your choice) so it can never be mistaken for an OpenAI route or clash if OpenAI ships its own `/v2`, and versions independently of the emulated standard.

- Pure helpers `extended_route_canonical` (canonical + alias both resolve), `is_deprecated_v1_alias`, `deprecation_headers` — all unit-tested.
- `send_json_response` gains a `send_json_response_ext(.., extra_headers)` sibling that injects the `Deprecation`/`Link` lines; the three handlers accept both the canonical and alias path and pass the header to every exit.
- `README.md` + `docs/minimal_client.md` split the OpenAI `/v1` surface from the `/uor/v1` extended API and document the deprecated aliases.

The response extras (`uor_audit`/`cascade_trail`) stay on `/v1` responses — non-breaking, and the SDKs ignore them (verified in phase F).

## Gates
fmt, clippy `-D warnings`, wasm32 lib build, server lib 54 + bin 6 tests (3 new), claim-wording — all green.

**Completes the #654 A–G decomposition.** Deferred optional increment: streaming for `/v1/responses`.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

