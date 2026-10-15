# #2091 — Clarify native input influence versus explicit occurrence access

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T21:16:24Z
- **Updated:** 2026-10-09T21:25:00Z
- **Closed:** 2026-10-09T21:22:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2091
- **Merged:** 2026-10-09T21:22:12Z
- **Merge commit:** a8e45fe357f56cdd528bf885240059bfbec85898
- **Base / Head:** main / UOR-Foundation:codex/input-access-trace-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The owner asked whether geometry can compare back to the beginning of user input for the next token. Trace the actual M2 caller and correct the conflation of whole-query state influence with independently addressable input occurrences. M2 already uses all admitted input in recurrent state plus query-specific Cue/U paths; its explicit read/Copy/donor list is Source-only. This does not establish the cause of 8/512 or describe M1's separate all-position read.

## Result

KEEP existing input influence and Source provenance. REJECT automatic query-as-Source promotion. A separate read-only history role is NOT YET PROMOTED. Next is a prospectively typed earliest-query intervention on the retained actual parent, with per-path witnesses and full native alias pools, to distinguish admission/information loss from scoring/learning failure before another mechanism change. No model or Rust source changed; accepted 8/512 complete development replies unchanged.

Exact head `aaf4c8d204a6facb82b7d59aa428c70c91e1171a`: geometry connections **29/29 PASS**, all **14** regenerated SVGs parse and are hash-identical across repeat generation, claim-wording/diff checks PASS. New flow diagram inspected as a rendered PNG. Existing 600-cell SVG regeneration changes element order only (identical line multiset). Engineering and adversarial source review confirm scope; final exact-head review posted before queue. No Rust compile/model execution or GPU in this documentation/source-audit piece.

## Checklist

- [x] Exact-head geometry/generation/wording/diff checks; reviewer receipt before queue
- [x] Canonical record `docs/labs/input-access-trace-2026-10-09/README.md`
- [x] Newest current-state entry ending in Next
- [x] Source-linked claim search; matching geometry section and generated figure updated
- [x] STATUS/ROADMAP/#2028 unchanged: no artifact, score or milestone-status change
- [x] Root README unchanged: clarification of existing paths, no capability or mechanism adoption
- [x] Claim-wording gate PASS
- [x] Fresh main verified; delivery branch/worktree removed; evidence verified in iCloud; scoped hygiene complete; M2 result posted

References #2030. Source audit at main65f2eb046 includes #2088/#2090; no competing M1 work or all-input reader adoption.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6089409636 — 2026-10-09T21:17:16Z — Casey-allard

Exact-head checks/review at `97991a642f860b100b7cf9ea7ac99969071ea905`: geometry verification **29/29 PASS**, all **14** figures regenerated, claim-wording and diff checks PASS, clean tree after regeneration. At unchanged generator/SVG head f86c, repeat hashes and all SVG XML parsing passed; final head changes only the delivery-status sentence and repeats verification/generation. Rendered new diagram inspected. Existing600-cell SVG changes ordering only, identical line multiset independently confirmed. Engineering source trace and adversarial exact-head review PASS. No Rust/model changes or model execution; accepted8/512 unchanged. Protected merge requested.


### comment 6089466973 — 2026-10-09T21:21:34Z — Casey-allard

Delivery head `aaf4c8d204a6facb82b7d59aa428c70c91e1171a`: all29 geometry checks PASS, all14 figures regenerated with clean tree/XML parsing; claim-wording/diff PASS. Only numeric-label spacing changed since independent approved97991; parent exact-head delta review PASS, no semantic/source/model change. One normal CI run is now active on this head. Prior head had two successful workflow suites but queue still reported all five acknowledgements expected; no manual check fabrication, protection change or bypass.


### comment 6089511424 — 2026-10-09T21:24:59Z — Casey-allard

Verified protected merge and six matching main blobs. Delivery branch removed remotely/locally; owned worktree archived/removed; owner checkout clean main. Evidence archive `codex-input-access-trace-20261009`:433152 bytes, MD5 `902ae611b22bc31583a600488379b18e`, fresh remote index and byte stream verified. Scoped hygiene applied; protected peer worktree retained by temporary lock, locks restored, busy uv skipped; hygiene itself freed0GiB after owned worktree removal, about53GiB free. No model/GPU work. Final head had one normal successful transport run and merged through queue; prior head’s duplicate-suite expected-status inconsistency was not bypassed.


---
_Comment/review ids in this file: 6089409636, 6089466973, 6089511424_

