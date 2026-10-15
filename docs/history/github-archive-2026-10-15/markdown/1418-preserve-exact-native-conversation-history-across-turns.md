# #1418 — Preserve exact native conversation history across turns

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T05:12:46Z
- **Updated:** 2026-10-09T13:17:14Z
- **Closed:** 2026-09-27T05:14:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1418
- **Merged:** 2026-09-27T05:14:04Z
- **Merge commit:** 2d3ee5f6fa1b3135a1f9bbff2f29d1d818edc4b1
- **Base / Head:** main / UOR-Foundation:codex/dialogue-continuity-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Successive native chat requests can reconstruct history from trimmed display text and restart the sampling seed. This adds an opt-in persistent `DialogueConversation` over the retained integer session: exact generated IDs and pending tokens remain in state, categorical randomness continues across turns, and the complete next request is checked before mutation. Model EOS and explicitly requested caller closure have separate provenance; execution failures poison the session. Shared tokenizer helpers preserve segmented literal-role framing without re-encoding generated text. Google retains CLI integration ownership.

The lab guide, roadmap and current state record the owner’s September27 clarification: this fourth lab keeps the native, multiplier-free serving target and does not adopt D10’s converted-backbone or serving-multiplier exceptions. Historical records and other labs’ work remain preserved. Source review updates track Claude’s corrected resume/stability claims and the independently owned weighted-shard objective issue.

Validation on source `b4c2baa15796acf794b73a0867712cdb00b5fa6a`: 5 tokenizer checks and10 integer-generation checks pass, including5 new focused checks for framing, preflight rejection, pending/EOS handling, sampler continuity and poisoning. Touched-source rustfmt, diff and claim-wording checks pass. Independent source and receipt review found no remaining actionable issue.

One unchanged accepted-bundle witness compares two4-token responses against direct integer stepping. All8 prediction records/full probability hashes, exact history, raw bytes and sampler cursors match; the persistent session uses36 steps without prefix replay. It completed in2.576s with15,810,560B peak child RSS; debug observation is not a performance claim. Both outputs remain capped fragments (`“Is there the`, `1 asked the people`), so there is no chat/prose/geometry/energy promotion. Build/check work took112.04s. Source/executable/artifact bindings, complete elapsed accounting and resource limits are in `docs/evidence/dialogue-continuity-validation-2026-09-27.json`.

The frozen Dot/Lorentz/Affine adaptation study is unchanged and still running. No training, new model dependency, numerical kernel, loader expansion or CLI change is part of this PR.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

