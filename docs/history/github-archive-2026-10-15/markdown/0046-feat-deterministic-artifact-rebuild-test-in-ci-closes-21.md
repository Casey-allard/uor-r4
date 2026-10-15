# #46 — feat: deterministic artifact rebuild test in CI (closes #21)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T01:31:50Z
- **Updated:** 2026-10-09T14:17:04Z
- **Closed:** 2026-07-22T01:49:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/46
- **Merged:** 2026-07-22T01:49:26Z
- **Merge commit:** c08711eeb5c6b5b6edad1e9ef54ecfa35ee6a9f3
- **Base / Head:** main / UOR-Foundation:feature/issue-21-deterministic-artifact-rebuild-test
- **Reviews / inline review comments:** 1 / 5
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #21: Deterministic artifact rebuild test enforcing Gate E / Decision D2, asserting byte-identical container output and identical BLAKE3 CIDs.

## Timeline (10 entries; chronological: conversation comments, reviews, review comments)

### comment 5040824406 — 2026-07-22T01:31:52Z — Casey-allard

@github-copilot review

### comment 5040824653 — 2026-07-22T01:31:55Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3626811608 — 2026-07-22T01:37:26Z — Copilot

_on crates/uor-r4-core/tests/deterministic_rebuild_test.rs:26_

This test silently skips when the fixture is missing, which means CI would not actually enforce Gate E / Decision D2 if the artifact file is absent or mispackaged. Since the PR goal is a deterministic rebuild check in CI, the fixture read should fail the test rather than returning early.

### review-comment 3626811629 — 2026-07-22T01:37:26Z — Copilot

_on crates/uor-r4-core/tests/deterministic_rebuild_test.rs:45_

The current assertions compare two parses of the same input against each other, but never assert that the rebuilt container bytes match the pinned fixture bytes. As written, the test can pass even if parse/serialize deterministically produces *wrong* bytes, and it doesn’t validate a byte-identical rebuild against the pinned artifact.

### review-comment 3626811645 — 2026-07-22T01:37:26Z — Copilot

_on crates/uor-r4-core/tests/deterministic_rebuild_test.rs:83_

This transition-graph determinism test builds graphs by inserting edges in a fixed order, so it doesn’t exercise the main compilation path (`compile_transitions_from_corpus`) where `HashMap` iteration can make edge insertion order (and thus canonical edge IDs) non-deterministic. For a Gate E / D2 determinism check, this should construct the graph twice via the real compiler routine and compare the resulting `edges` / reverse indexes.

### review-comment 3626811668 — 2026-07-22T01:37:26Z — Copilot

_on crates/uor-r4-core/src/transformerless/certificate.rs:90_

`compute_cid()` silently treats JSON serialization failures as an empty byte vector (`unwrap_or_default()`), which would make the CID the hash of empty content and can mask real serialization bugs. Since `compute_cid()` is security/consistency-critical, serialization errors should be surfaced (panic or return a `Result`).

### review 4750256347 — 2026-07-22T01:37:27Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR aims to implement Issue #21 by adding CI tests intended to enforce deterministic artifact rebuild semantics (Gate E / Decision D2), while also introducing supporting primitives for quantized scoring (`ScoreQ`) and a machine-checkable `Certificate` schema.

**Changes:**
- Add `ScoreQ` (Q16.16 fixed-point score) and thread it into `TransitionGraph::Edge` via a new `add_edge_with_score` API.
- Add a new `Certificate` type with CID computation, CBOR serialization, and attestation verification, plus associated tests.
- Add a new “deterministic rebuild” test suite intended to assert byte-identical rebuilds and stable CIDs.

### Reviewed changes

Copilot reviewed 8 out of 9 changed files in this pull request and generated 5 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/score_q_test.rs | Adds unit tests for `ScoreQ` conversions/arithmetic and `StorageDescriptor::decode`. |
| crates/uor-r4-core/tests/deterministic_rebuild_test.rs | Adds Gate E / D2 determinism tests for artifact container bytes/CIDs, transition graphs, and certificates. |
| crates/uor-r4-core/tests/certificate_test.rs | Adds basic tests for certificate CID verification and CBOR roundtrip. |
| crates/uor-r4-core/src/transformerless/transitions.rs | Extends `Edge` to include `ScoreQ` and adds `add_edge_with_score` plus a compatibility `add_edge`. |
| crates/uor-r4-core/src/transformerless/score_q.rs | Introduces `ScoreQ` fixed-point type and `StorageDescriptor` decoding helper. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes `score_q` module and conditionally exports `Certificate`. |
| crates/uor-r4-core/src/transformerless/certificate.rs | Implements certificate schema, CID computation, attestation verification, and CBOR serde. |
| crates/uor-r4-core/Cargo.toml | Adds `ciborium` dependency for CBOR serialization. |
| Cargo.lock | Locks `ciborium` and transitive dependencies. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626811688 — 2026-07-22T01:37:27Z — Copilot

_on crates/uor-r4-core/src/transformerless/certificate.rs:130_

`verify_attestation()` does not check `attestation.deterministic_canonical_mode`. As a result, a certificate can pass structural attestation even when the caller explicitly marks deterministic canonical mode as not verified, which undermines the meaning of the attestation payload.

### comment 5040865114 — 2026-07-22T01:39:36Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5040914225 — 2026-07-22T01:48:35Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Merge conflicts resolved in commit `74208d2`. The conflict was in `certificate.rs` where both branches modified `compute_cid` — the feature branch had used `serde_json` (with a missing closing `}`) while main had switched to `ciborium` CBOR serialization. Resolved by taking the CBOR approach from main, which is more deterministic. Also fixed:
- Duplicate `ScoreQ` re-export in `mod.rs` (both `reference_state::ScoreQ` and `score_q::ScoreQ` were being exported after auto-merge)
- Missing closing `}` for `test_deterministic_transition_graph_rebuild` in the test file
- Unused imports in `deterministic_rebuild_test.rs`

All tests pass after the merge.


---
_Comment/review ids in this file: 5040824406, 5040824653, 5040865114, 5040914225, 4750256347, 3626811608, 3626811629, 3626811645, 3626811668, 3626811688_

