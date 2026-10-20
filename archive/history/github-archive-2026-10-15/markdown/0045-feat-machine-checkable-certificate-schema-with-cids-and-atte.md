# #45 — feat: machine-checkable certificate schema with CIDs and attestation (closes #20)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T01:26:28Z
- **Updated:** 2026-07-22T01:38:03Z
- **Closed:** 2026-07-22T01:37:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/45
- **Merged:** 2026-07-22T01:37:59Z
- **Merge commit:** 6d9f577c18e8a92913542bc8f350a3cb233b91de
- **Base / Head:** main / UOR-Foundation:feature/issue-20-certificate-schema
- **Reviews / inline review comments:** 1 / 6
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #20: Machine-checkable Certificate schema carrying source, corpus, graph, metric, op, and benchmark CIDs, empirical claims, 95% CIs, CBOR serialization, and Gate K protocol attestation verification.

## Timeline (9 entries; chronological: conversation comments, reviews, review comments)

### comment 5040796423 — 2026-07-22T01:26:31Z — Casey-allard

@github-copilot review

### comment 5040796560 — 2026-07-22T01:26:32Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3626808683 — 2026-07-22T01:36:29Z — Copilot

_on crates/uor-r4-core/tests/score_q_test.rs:24_

`ScoreQ::from_logprob` takes an `f32`, but these literals default to `f64`, which will fail to compile due to a type mismatch. Add `f32` suffixes (or cast) to the literals.

### review-comment 3626808694 — 2026-07-22T01:36:29Z — Copilot

_on crates/uor-r4-core/src/transformerless/transitions.rs:73_

`weight` is a `u32`, but `weight as i32` can wrap for values > `i32::MAX`, producing a negative raw `ScoreQ` and silently corrupting the score. Clamp before converting (or require callers to provide an explicit `ScoreQ`).

### review-comment 3626808703 — 2026-07-22T01:36:29Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_q.rs:24_

`from_logprob` quantizes via `as i32`, which truncates toward zero and introduces a systematic bias. Rounding to the nearest fixed-point value typically yields better and symmetric quantization error.

### review 4750252744 — 2026-07-22T01:36:30Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR introduces a machine-checkable “certificate” data model to carry empirical/structural claims (with CIDs), along with CBOR serialization and protocol attestation checks, and begins migrating scoring/graph plumbing toward a fixed-point `ScoreQ` representation.

**Changes:**
- Add `transformerless::certificate` with CID computation/verification, Gate K attestation checks, and CBOR encode/decode.
- Add `transformerless::score_q::ScoreQ` (Q16.16 fixed-point) and thread a `ScoreQ` field into `TransitionGraph::Edge`.
- Add new unit tests for `ScoreQ` behavior and certificate CID/CBOR/attestation behavior, plus add `ciborium` dependency.

### Reviewed changes

Copilot reviewed 7 out of 8 changed files in this pull request and generated 6 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/score_q_test.rs | Adds tests for `ScoreQ` conversion precision, saturating arithmetic, and descriptor decoding. |
| crates/uor-r4-core/tests/certificate_test.rs | Adds tests for certificate CID verification, CBOR roundtrip, and attestation failure cases. |
| crates/uor-r4-core/src/transformerless/transitions.rs | Extends `Edge` to carry a `ScoreQ` and adds an `add_edge_with_score` constructor path. |
| crates/uor-r4-core/src/transformerless/score_q.rs | Introduces the `ScoreQ` fixed-point type and `StorageDescriptor` decoding. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes `ScoreQ` (and `Certificate` for non-wasm targets) and registers new modules. |
| crates/uor-r4-core/src/transformerless/certificate.rs | Implements the certificate schema, CID computation/verification, CBOR serialization, and attestation validation. |
| crates/uor-r4-core/Cargo.toml | Adds `ciborium` dependency to support CBOR serialization. |
| Cargo.lock | Locks `ciborium` and its transitive dependencies. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626808712 — 2026-07-22T01:36:30Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_q.rs:115_

`StorageDescriptor::decode` uses `wrapping_shl`/`wrapping_shr`, so large shifts or overflow will wrap the value, which can break monotonicity and yield nonsensical decoded scores. Prefer checked shifts and saturate/clamp on overflow.

### review-comment 3626808727 — 2026-07-22T01:36:30Z — Copilot

_on crates/uor-r4-core/src/transformerless/certificate.rs:94_

The PR/issue describe a CBOR-serialized, canonical certificate, but `compute_cid` currently hashes `serde_json` output and also silently falls back to empty bytes on serialization error (`unwrap_or_default`). That makes the CID depend on JSON formatting and can incorrectly “verify” when serialization fails. Compute the CID over CBOR bytes and fail fast on serialization errors.

### review-comment 3626808735 — 2026-07-22T01:36:30Z — Copilot

_on crates/uor-r4-core/src/transformerless/certificate.rs:131_

`verify_attestation` never checks `deterministic_canonical_mode`, so an attestation can pass even when the canonical-mode requirement is explicitly marked false.


---
_Comment/review ids in this file: 5040796423, 5040796560, 4750252744, 3626808683, 3626808694, 3626808703, 3626808712, 3626808727, 3626808735_

