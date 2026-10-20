# #57 — feat: source-bias amplification, rare-group erasure, and provenance deletion tests (closes #31)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T04:14:57Z
- **Updated:** 2026-07-22T04:44:00Z
- **Closed:** 2026-07-22T04:43:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/57
- **Merged:** 2026-07-22T04:43:57Z
- **Merge commit:** 5535be70b73d5a2c2f53e3d08c633571a11e5ba6
- **Base / Head:** main / UOR-Foundation:feature/issue-31-fairness-provenance-harness
- **Reviews / inline review comments:** 2 / 8
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #31: Source-bias amplification metric (A_bias <= 1.05), rare-group erasure metric (R_rare >= 0.95), targeted provenance-deletion witness (PROV), BLAKE3 CID hashing, and CBOR-serializable FairnessAndProvenanceCertificate.

## Timeline (13 entries; chronological: conversation comments, reviews, review comments)

### comment 5041788861 — 2026-07-22T04:15:01Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3627445691 — 2026-07-22T04:19:44Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_q.rs:11_

The `Hash` derive was removed from `ScoreQ`. Since `ScoreQ` is a public type, this is a breaking API change for any downstream code that stores `ScoreQ` in hashed collections or uses it as part of a hashed key. If serde support is the goal, it should be added without dropping existing derives.

### review-comment 3627445703 — 2026-07-22T04:19:45Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:30_

`RouteTranslationMap` uses `HashMap`, but `GraphPatch::compute_cid()` hashes a serialization of the whole patch. `HashMap` iteration order is nondeterministic, so the computed CID can change across runs/platforms for the same logical mapping, breaking content addressing and CBOR round-trips that validate the CID.

### review-comment 3627445717 — 2026-07-22T04:19:45Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:87_

`compute_cid()` uses `serde_json::to_vec(...).unwrap_or_default()`, which can silently fall back to hashing empty bytes if serialization ever fails, yielding a constant CID. Also, this crate already uses CBOR bytes as the hashed payload for CIDs (e.g., `transformerless/certificate.rs`), which is a better match for the CBOR round-trip APIs here.

### review-comment 3627445733 — 2026-07-22T04:19:45Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:121_

`GraphPatch` hashes `added_edges` (including each edge's `id`) into the patch CID, but `apply()` ignores `edge.id` and lets `TransitionGraph::add_edge_with_score` assign a fresh canonical ID. This can make the patch's content-addressed payload disagree with the actual graph mutation, undermining reproducibility and any later verification that relies on the encoded edge IDs.

### review-comment 3627445747 — 2026-07-22T04:19:45Z — Copilot

_on crates/uor-r4-core/src/transformerless/graph_patch.rs:150_

The doc comment says this "formally" verifies Theorem 11 route translation, but the implementation only checks score equality for `RouteMapping::Retained` and ignores `Split`/`Merged`/`Removed`. Either expand verification to cover those mappings or narrow the comment so callers don't assume stronger guarantees than are provided.

### review-comment 3627445763 — 2026-07-22T04:19:45Z — Copilot

_on crates/uor-r4-core/src/transformerless/fairness_provenance.rs:85_

`compute_cid()` uses `serde_json::to_vec(...).unwrap_or_default()`, which can silently hash empty bytes on serialization failure (e.g., if any metric contains a non-finite `f64`). That would yield a constant CID and make `verify_cid()` meaningless. Hashing the same CBOR bytes used for `to_cbor_bytes()` avoids that pitfall and matches the existing certificate CID pattern in this crate.

### review 4750980806 — 2026-07-22T04:19:46Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds new transformerless certification utilities and tests intended to support Issue #31’s fairness/provenance requirements, including graph patching/Theorem 11 checks and a fairness+provenance certificate with BLAKE3 CIDs and CBOR round-trips.

**Changes:**
- Introduces `graph_patch` (graph deltas + route translation map + Theorem 11 verifier) with CBOR round-trip tests.
- Introduces `fairness_provenance` (bias amplification, rare-group erasure, provenance-deletion witness, and certificate) with tests.
- Extends core types (`EdgeKind`, `Edge`, `ScoreQ`) with serde derives for serialization.

### Reviewed changes

Copilot reviewed 7 out of 7 changed files in this pull request and generated 7 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/graph_patch_test.rs | Adds tests for applying a graph patch, Theorem 11 verification, and CBOR round-trip. |
| crates/uor-r4-core/tests/fairness_provenance_test.rs | Adds tests for bias/erasure thresholds, provenance witness creation, and CBOR round-trip. |
| crates/uor-r4-core/src/transformerless/transitions.rs | Adds serde (de)serialization derives to `EdgeKind` and `Edge`. |
| crates/uor-r4-core/src/transformerless/score_q.rs | Adds serde (de)serialization derives to `ScoreQ`. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes new `fairness_provenance` and `graph_patch` modules (non-wasm). |
| crates/uor-r4-core/src/transformerless/graph_patch.rs | New graph patch + route translation types, CID hashing, CBOR APIs, and Theorem 11 verifier. |
| crates/uor-r4-core/src/transformerless/fairness_provenance.rs | New fairness/provenance metrics, certificate + CID hashing, CBOR APIs, and evaluator helpers. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3627445779 — 2026-07-22T04:19:46Z — Copilot

_on crates/uor-r4-core/src/transformerless/fairness_provenance.rs:168_

`verify_provenance_deletion` does not perform any verification: it unconditionally sets `graph_integrity_verified: true` and just echoes inputs. This doesn't match the PR description's claim of a "targeted provenance-deletion witness". Consider either (a) renaming this to make it clear it's a constructor, or (b) changing it to take the structures needed to actually verify deletion/integrity and return `Result<ProvenanceDeletionWitness, _>` on failure.

### review 4751025681 — 2026-07-22T04:30:56Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3627481764 — 2026-07-22T04:30:56Z — Copilot

_on crates/uor-r4-core/src/transformerless/fairness_provenance.rs:168 · in reply to review-comment 3627445779_

Changed `verify_provenance_deletion` to perform actual verification in f5748cb: it now takes the `TransitionGraph` and the tombstoned edge IDs, checks each referenced edge exists and has weight 0, and returns `Result<ProvenanceDeletionWitness, String>` with a descriptive error on any integrity violation. Added a failure-path test covering both an untombstoned edge and an out-of-bounds edge ID.

### comment 5041886902 — 2026-07-22T04:33:02Z — Casey-allard

@copilot+claude-fable-5 resolve the merge conflicts in this pull request

### comment 5041913332 — 2026-07-22T04:37:57Z — Copilot


> @copilot+claude-fable-5 resolve the merge conflicts in this pull request

Resolved the merge conflicts in 0c88a08. The only conflicted file was `crates/uor-r4-core/src/transformerless/graph_patch.rs`; the merge keeps the `RouteMapping` enum definition (which main's version referenced but no longer defined), main's fixed-width `u32` edge IDs, CBOR-based CID computation, and tombstone bounds-check errors, plus this branch's post-apply Theorem 7 verification. Full workspace test suite passes after the merge.


---
_Comment/review ids in this file: 5041788861, 5041886902, 5041913332, 4750980806, 4751025681, 3627445691, 3627445703, 3627445717, 3627445733, 3627445747, 3627445763, 3627445779, 3627481764_

