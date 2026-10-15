# #1494 — ROADMAP §2b and current-state: record the S4 trained-in 2I transport adoption (steward)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T06:16:18Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-29T20:09:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1494
- **Merged:** 2026-09-29T20:09:52Z
- **Merge commit:** c2108c6e929009d45a60be3261663ba1fa9d71ef
- **Base / Head:** main / UOR-Foundation:codex/s4-adoption-doc
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Lab 1 delivery request (05:34 UTC 09-29, #973): record the S4 trained-in 2I transport adoption in ROADMAP §2b and current-state.

## What changed

- **ROADMAP §2b, "Model family and configuration":** the main line's recurrent transport is the trained-in snap to the binary icosian group 2I; the memory-port and QAT phases continue from arm A with the snap on.
- **ROADMAP §2b, "Codec and serving profile":** replaces "merged (#1483); adoption is decided by its pre-registered fit" with the adoption result — arm A snapped `2.547953579954328` (`2b3b5681…`, `transport.json` = icosian) against arm B free `2.5373633745806394` (`fcc3099b…`), `+0.010590` inside the ≤ 0.02 band, so no second seed; executable `fbeedde1…`; roots and manifests recorded.
- **current-state:** replaces the stale "S4 (#1483), not merged" bullet with the merged + adopted record, with roots, manifests and scope limits.

Nothing else, per the request. The post-hoc-snap negatives keep their original scope (the ROADMAP §2b post-hoc-snap sentence and the dated September 28 current-state section are intentionally untouched).

## Checks

- `python3 scripts/check_claim_wording.py` — passed.
- Docs-only diff on two named paths.

References #973.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5884966979 — 2026-09-29T06:32:39Z — Casey-allard

**Lab 1 review of `7924d075` (06:32 UTC): approved.** The §2b rows and current-state match the re-read and decision (comment 5884326637) number for number, including the scope limits and the unbuilt S1.4 kernel. Kimi: deliver by the procedure (recheck the exact head, queue, verify the patch, notify).

### comment 5897866758 — 2026-09-29T20:11:52Z — Casey-allard

Delivered as `c2108c6e`. The squash patch equals the PR diff from its merge base, hunk for hunk; only the blob index lines differ because `main` advanced. — Lab 1


---
_Comment/review ids in this file: 5884966979, 5897866758_

