# #748 — docs: fold in #743/#742, fix stale #490 claim, add track-viability answer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T22:59:43Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-16T23:20:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/748
- **Merged:** 2026-08-16T23:20:09Z
- **Merge commit:** ed83e7e20b8617a6e8e5e692d6f5b782ffeea51b
- **Base / Head:** main / UOR-Foundation:issue-747-docs-overhaul
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Deep audit of README.md, ROADMAP.md, docs/RESEARCH.md against current main
and the live code (per the maintainer's request following the #655
coherence sweep). Findings:

- **Found and fixed a real drift.** README's geometric-router limitation
  bullet said retrieval stays word-overlap "until issue #490 clears its
  gate" -- but #490 (and follow-up #502) closed 2026-08-08, over a week
  before this audit, and the content-vector query is the current deployed
  default (0.8763 MRR / 0.99 recall per docs/RESEARCH.md). Corrected.
- **Found an incomplete project-layout table.** `uor-r4-naf`, `repo-model`,
  `repo-conformance`, and `xtask/` are real workspace members with no entry
  in README's crate table; `proofs/wasm-gemm-gnaf/` (vendored by #742) had
  no mention anywhere in the layout. Added all four with descriptions
  pulled from their own doc comments.
- **Added the track-viability answer** the maintainer asked for:
  `docs/RESEARCH.md#which-track-can-actually-produce-coherent-text` states,
  with citations, both halves of the current evidence -- the offline
  per-position signal from the broad-teacher programme (#509/#320) is real
  and P-4-clean, but has not yet composed into coherent end-to-end
  generation on the best locally compiled bundle (#745, open) -- and
  classifies engine tracks as goal-aligned (transformerless/R4G1),
  comparison-only (attention/r4-attention), or out of scope for text
  generation entirely (GNAF -- a WASM-GEMM optimality proof, not a language
  mechanism). Flags the dormant, P-4-legal `#604` route-attention kernel
  (built, differentially tested, wired into no serving path) as an
  unexplored lever worth evaluating against #745.
- Added #743/#744/#745 to RESEARCH.md's "Open, with defined work" table.
- Sharpened the generation-quality bullets in README/ROADMAP to cite the
  #655 sweep's concrete evidence (empty output / word-salad) rather than
  only the older aggregate percentages.
- ROADMAP.md: refreshed the review date, added a "Next up" section with
  the recommended backlog sequencing (#744 -> #745 -> #653 phase 2 ->
  #740/#741, with rationale), and a Landed entry for GNAF phase 1 (#742).

No goals, existing measurement conclusions, or evidence were changed --
this is additions and corrections to drifted/missing narrative text only.

## Verification

- `scripts/check_claim_wording.py` -- clean
- `cargo fmt --check` -- clean (no code touched)
- Markdown-only diff (README.md, ROADMAP.md, docs/RESEARCH.md), so CI's
  docs-only fast path applies

Closes #747

🤖 Generated with Claude (Cowork)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

