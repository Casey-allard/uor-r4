# #641 — feat(#603): versioned TeacherTrace profiles + content-addressed trace sidecars

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T14:24:00Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-12T14:38:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/641
- **Merged:** 2026-08-12T14:38:43Z
- **Merge commit:** a023eaa0f6ff6a850d169c5619f9d648cd175a40
- **Base / Head:** main / UOR-Foundation:i603-teacher-trace
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#603): versioned TeacherTrace profiles + content-addressed trace sidecars

Four bounded profiles under uor-r4-teacher-trace/1 (minimal = exactly
today, never recorded; layer incl. the #95 lane; attention-support;
full) as typed records with pinned canonical bytes + declared digests;
registry refuses unknown or unbounded declarations via graph-compilers
existing sanctioned surface. Richer lanes ride NEW .trace sidecars
copying the .prob pattern (v4 records are a fixed 88-byte layout - no
additive in-record path), with per-shard kappa registration, canonical
merge, torn-row checks, crash-safe resume, and pinned row width; absent
support slots carry an explicit 0xFFFFFFFF marker - absence is never
zero-filled. ObservationManifest gains trace_profile/trace_row_bytes
(serde default) and identity_bundle_digest(): one blake3 over the six
identity components (#597 manifest kappa, input CID, #600 geometry,
#601 tokenizer, #602 operator, trace profile) with explicit
absent/present markers - the issues one stable bundle. Capture taps
the #599 forward_capturing path and #602 factored per-head weights,
bounded to declared indices; traced step bit-identical to plain step
(tested). Default stays minimal everywhere; #95s negative measurement
(2.8% vs 31.7%) cited as the recorded reason richer profiles are
opt-in and fitting adoption needs its own measured issue. From-text
batched drivers remain minimal-only (no BatchedTeacher capture surface
- recorded follow-up, not invented). 127 suites green; no new deps.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

