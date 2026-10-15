# #1119 — Expose position-preserving R4 artifact generation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T16:11:34Z
- **Updated:** 2026-09-04T16:15:43Z
- **Closed:** 2026-09-04T16:13:05Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1119
- **Merged:** 2026-09-04T16:13:04Z
- **Merge commit:** 1ff79dcb0fbf9b1be0799ad0ef7defdb56bc32e6
- **Base / Head:** main / UOR-Foundation:codex/973-position-r4-generation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The accepted ordinary #973 artifact could generate prompt-dependent text, but the existing position-preserving R4/H4 cache had no artifact-only generation surface. This adds `generate-position-r4-language-path`, binds the exact ordinary weights, exact-H4 geometry, and H4 frame sidecar by content identity, primes one chronological cache, and advances it through the existing `execution="r4"` / `intervention="native"` path.

With seed 9738, top-k 40, temperature 0.8, and 16 tokens, the two fixed prompts produced token- and text-identical trajectories to their already-recorded ordinary controls. The direct runs recorded nonzero H4 transport work with zero provider, teacher, future, or forbidden reads and did not load the historical invalid #1043 fitted artifact. This is two-prompt trajectory preservation through the full f32 softmax cache; it does not establish exact logit equivalence, general quality, H4 advantage, recurrent compression, or transformerless serving.

Validation:
- CLI import/help completed successfully
- both fixed R4 generations completed once; existing ordinary outputs were read for comparison without rerunning them
- `python3 scripts/check_claim_wording.py`
- `git diff --cached --check`
- independent read-only review found no blockers

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

