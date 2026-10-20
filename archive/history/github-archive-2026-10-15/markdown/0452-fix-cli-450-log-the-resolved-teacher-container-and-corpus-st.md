# #452 — fix(cli): #450 — log the resolved teacher container and corpus streams with their κ

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T05:17:00Z
- **Updated:** 2026-08-06T13:37:41Z
- **Closed:** 2026-08-06T13:37:41Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/452
- **Merged:** 2026-08-06T13:37:41Z
- **Merge commit:** f0851d20589990c45f1d72cfcd20dfa62155b291
- **Base / Head:** main / UOR-Foundation:i450-repro-hardening
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #450. Output-only; no default path changed, no artifact bytes changed, no schema field touched.

Motivated by the #448 false alarm: `--artifacts` defaults to a shared mutable `/tmp` path, a concurrent script staged a different teacher container into it, and nothing in the output revealed which container was read — costing hours and producing a bogus measurement-integrity report.

Adds, before the expensive work starts:
```
teacher container: <path> (κ blake3:8fbf3f68…)
corpus streams: <meta> + <recs> (κ blake3:0702bed3…)
cover container: <path> (κ …)
```
Covers all four sites that resolve an artifacts path (score, cover, cover_sweep, graph-compiler entry). The printed teacher κ cross-checks against `container.kappa` in the pinned `baseline_kappa.json`. κ spellings deduped into `reproducibility::container_kappa` / `corpus_stream_kappa` over the same bytes the four sites each hashed inline.

Incidental find, logged but deliberately NOT fixed (it would change `corpus_kappa` and therefore emitted bytes): `cover_command` hashes the *requested* corpus paths while loading from the *fallback-resolved* ones; `score_command` uses resolved paths for both. The new log line makes the divergence visible.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

