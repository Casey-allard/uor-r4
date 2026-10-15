# #56 — feat: immutable graph patch and route-translation engine (closes #30)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T04:00:46Z
- **Updated:** 2026-10-09T14:17:04Z
- **Closed:** 2026-07-22T04:20:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/56
- **Merged:** 2026-07-22T04:20:46Z
- **Merge commit:** ff90068bae53ecc4a357ef4f47fc89ee70077270
- **Base / Head:** main / UOR-Foundation:feature/issue-30-immutable-graph-patch-rfc
- **Reviews / inline review comments:** 1 / 6
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #30: Immutable GraphPatch delta data structures (added edges, residual updates, tombstones, route translation mapping), BLAKE3 CID hashing, CBOR serialization, in-place graph patch application, and Theorem 11 score consistency verifier.

## Timeline (8 entries; chronological: conversation comments, reviews, review comments)

### comment 5041711149 — 2026-07-22T04:00:50Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3627398414 — 2026-07-22T04:05:15Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_q.rs:11_

`ScoreQ` previously derived `Hash`, but the updated derive list drops it. Since `ScoreQ` is a public type, removing `Hash` is a breaking API change for downstream code that may store `ScoreQ` in `HashMap`/`HashSet`.

### review-comment 3627398420 — 2026-07-22T04:05:15Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:87_

`compute_cid()` hashes `serde_json::to_vec()` output and silently falls back to an empty byte vector on serialization error (`unwrap_or_default`). This can produce incorrect/unstable CIDs (especially since patches are otherwise serialized as CBOR), and would mask real serialization bugs by hashing empty bytes.

### review 4750918734 — 2026-07-22T04:05:16Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Implements Issue #30’s “immutable patch epoch” building blocks in `uor-r4-core` by introducing a `GraphPatch` delta format (added edges / residual updates / tombstones), a route-translation map, CBOR serialization, BLAKE3-based content IDs, and a basic Theorem 11 consistency verifier, with accompanying tests.

**Changes:**
- Add new `transformerless::graph_patch` module with `GraphPatch`, route translation mapping, CID hashing, CBOR encode/decode, in-place patch application, and a Theorem 11 verifier.
- Derive `serde::{Serialize, Deserialize}` for `EdgeKind`, `Edge`, and `ScoreQ` to support patch serialization.
- Add unit tests covering patch application, route translation mapping behavior, and CBOR round-tripping.

### Reviewed changes

Copilot reviewed 5 out of 5 changed files in this pull request and generated 6 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/graph_patch_test.rs | Adds tests for patch application, route translation mapping, and CBOR roundtrip. |
| crates/uor-r4-core/src/transformerless/transitions.rs | Adds serde serialization derives to edge types used in patches. |
| crates/uor-r4-core/src/transformerless/score_q.rs | Adds serde derives to `ScoreQ` for serialization support (but currently drops `Hash`). |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new `graph_patch` module behind the non-wasm cfg gate. |
| crates/uor-r4-core/src/transformerless/graph_patch.rs | Introduces patch data structures, CID computation, CBOR serialization, patch application, and Theorem 11 verifier. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3627398430 — 2026-07-22T04:05:16Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:22_

`RouteTranslationMap` uses `HashMap`, but patch CIDs and serialized bytes need deterministic iteration order to be stable across runs/machines. `HashMap` iteration order is randomized, so both CID computation and CBOR encoding can differ even for semantically identical patches.

### review-comment 3627398441 — 2026-07-22T04:05:16Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:112_

When applying `added_edges`, the patch-provided `Edge.id` is ignored and a new canonical ID is assigned by `add_edge_with_score`. This can make a patch payload internally inconsistent (CID covers `Edge.id`, but application doesn’t), and can break route-translation IDs if they were derived from the patch’s `Edge.id` values.

### review-comment 3627398454 — 2026-07-22T04:05:16Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:127_

Tombstone indices that are out of bounds are silently ignored, which can hide corrupted/invalid patches. Also, `build_reverse_index()` already calls `verify_theorem_7()`, so the subsequent explicit `verify_theorem_7()` is redundant work.

### review-comment 3627398467 — 2026-07-22T04:05:16Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:46_

`GraphPatch` encodes edge references as `usize` (`residual_updates: Vec<(usize, ScoreQ)>`, `tombstone_edge_ids: Vec<usize>`). The implementation plan explicitly calls out that artifact/domain values should be fixed-width and “never usize” (to keep formats portable and deterministic across architectures). Consider switching these IDs/indices to `u32` (matching canonical edge IDs) or `u64`.


---
_Comment/review ids in this file: 5041711149, 4750918734, 3627398414, 3627398420, 3627398430, 3627398441, 3627398454, 3627398467_

