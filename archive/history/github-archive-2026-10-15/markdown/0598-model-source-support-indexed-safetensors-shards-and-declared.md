# #598 — model-source: support indexed Safetensors shards and declared BF16/F16/F32 codecs

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T21:57:27Z
- **Updated:** 2026-08-12T08:15:12Z
- **Closed:** 2026-08-12T08:15:12Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/598
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The current Hugging Face source adapter is usable for the pinned SmolLM2 single-file BF16 fixture, but compiling permissively licensed open models requires deterministic ingestion of indexed shards and an explicit source-codec boundary.

# Current evidence

- `crates/uor-r4-model-source/src/lib.rs:1090-1158` loads only `source/model.safetensors` and flattens it into one large `Vec<f32>`.
- `append_tensor` at `crates/uor-r4-model-source/src/lib.rs:1180-1200` accepts only `safetensors::Dtype::BF16` and converts the complete tensor payload eagerly.
- `src/model.rs:519-544` downloads `*.safetensors` but does not resolve `model.safetensors.index.json` or verify tensor-to-shard completeness.
- The source snapshot/provenance work is tracked separately in #597. This issue must consume that manifest rather than inventing another identity format.

# Problem

Indexed sharded Safetensors models and declared F16/F32 sources are rejected or unrepresented, while unsupported/partial source layouts are not checked at a single deterministic boundary.

# Goal

Provide bounded, deterministic source ingestion for indexed Safetensors shards with explicit BF16/F16/F32 support and explicit rejection of unsupported quantized codecs.

# Scope

- Parse and validate `model.safetensors.index.json` and resolve every tensor name to exactly one shard.
- Detect missing, duplicate, unexpected, shape, byte-length, and dtype inconsistencies before model construction.
- Add source-codec conversion for BF16, F16, and F32 with tests against known bit patterns.
- Preserve the current single-file path through the same validation boundary.
- Keep an eager flattened path for the first implementation, with a bounded/streamed alternative recorded only where measurements justify it.

# Non-goals

- No runtime quantization, NAF representation, or `uor-matmul` adoption.
- No architecture-family adapter changes beyond the tensor-loading boundary.
- No silent approximation of GPTQ/AWQ/other quantized formats; reject them with a focused source error.

# Dependencies and blockers

- Depends on #597 for the source snapshot manifest and shard membership/CID.
- #515 governs dormant feature/claim registration; #531 owns corpus-scale saturation; #589 blocks upstream arithmetic/registry integrations.

# Proposed implementation shape

Add a model-source loader layer that accepts a validated source manifest, uses the Safetensors index when present, sorts tensor names deterministically, validates expected shapes/dtypes, and exposes a codec enum to the architecture adapter. Keep compiler-side allocation and floating point permitted; do not move any of this into the deployed runtime.

# Acceptance criteria

- [ ] A pinned indexed-shard fixture loads with identical logical tensors to an equivalent single-file fixture.
- [ ] Missing, duplicate, unindexed, malformed, and mismatched shard entries fail before forward execution.
- [ ] BF16, F16, and F32 fixtures decode deterministically; unsupported quantized dtypes fail explicitly.
- [ ] Peak source-ingestion behavior is documented for the eager path, and any streaming path is bounded by a testable budget.
- [ ] The source manifest records every shard and the loader consumes that identity.

# Verification

- Unit/integration fixtures for index resolution, dtype conversion, shape checks, and failure modes.
- Deterministic tensor-layout and double-load digest tests.
- Existing SmolLM2 tests plus `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`, `cargo test --workspace --offline`, and the claim-wording checker.

# Compatibility and migration

The loader must retain the current single-file BF16 behavior and legacy artifact readers. Any compiler-layout or source κ change requires an explicit era/re-pin note; no existing artifact CID is silently rewritten.

# Claim status

Implementation and empirical source-codec/conformance criteria. It makes no claim about target-model quality.


## Comments (4)

### comment 5262075235 — 2026-08-12T03:53:49Z — Casey-allard

## Plan (starting now, on top of #597's merged manifest)

One deterministic validation boundary in `uor-r4-model-source` before model construction: (1) resolve `model.safetensors.index.json` when present — every tensor name maps to exactly one shard; missing/duplicate/unexpected entries, shape, byte-length, and dtype inconsistencies are focused errors at this boundary, not later panics; (2) source-codec layer with declared BF16/F16/F32 conversion, bit-pattern unit tests (incl. subnormals, ±inf, NaN payloads, rounding-free widening); (3) the existing single-file path re-routed THROUGH the same boundary (no second path); (4) quantized formats (GPTQ/AWQ etc.) rejected with a focused source error naming the format — no silent approximation; (5) shard membership cross-checked against the #597 source manifest when one is present. Eager flattened load retained (bounded/streamed recorded as a measured follow-up only if numbers justify). No runtime changes, no uor-matmul adoption (its non-goals). Gates + full suite + claim-wording before the PR; record + docs on close.

### comment 5262240292 — 2026-08-12T04:21:28Z — Casey-allard

## Delivery record (PR #634 in queue)

Single deterministic boundary `SafetensorsSnapshot::open` (model-source lib.rs:1771): full shard-header validation pass before any decode; every tensor resolved to exactly one shard; **9 focused error variants** (missing split precisely into shard-file vs tensor; duplicate detection includes raw-JSON double-mapping via a duplicate-preserving visitor — serde_json silently dedups; unexpected/shape/byte-length/dtype; unsupported dtypes rejected BY NAME, GPTQ/AWQ never approximated). Single-file path = the synthesized 1-shard case of the identical code path — no fork; **single-file κ byte-identical to pre-#598 `blake3(model.safetensors)`** (synthetic test; the 257MiB pinned fixture isn't in the build env, so real-weight preservation is by construction + synthetic verification — recorded honestly). Codec module: exact-widening BF16/F16/F32 with 43 explicit hex bit-pattern rows incl. subnormals, ±inf, payload NaNs; rustdoc asserts no rounding path exists. #597 seam: index-referenced shard presence checked here; full manifest digest cross-check documented as the root crate's job. 21 new tests; no deps added; runtime untouched; all four gates + full suite green. Closes on #634 merge; #599 (adapter conformance) next.

### comment 5262888847 — 2026-08-12T05:49:00Z — Casey-allard

**CI fix pushed to PR #634 (force-with-lease, re-enqueued):** the register-conformance gate (R5 audit-limits) correctly refused the new `SourceIngestError` type — shipped crates may only name the four sanctioned error types in `Result` positions. Fixed WITH the discipline rather than around it: the 9 failure classes are now `SourceIngestKind`, a structured **data classification carried on the sanctioned `SourceUnavailable`** (whose documented condition class — a declared source artifact failing ingestion at construction — is exactly what #598 reports). No audit or model-registry change; per-class test specificity preserved (tests assert the exact kind); `cargo xtask validate` now exit 0 alongside the other four gates + full suite. Process lesson adopted for the rest of the campaign: **xtask validate joins the standing pre-PR gate list** — it was in CI but not in the delegation gate spec, which is how this slipped.

### comment 5264138912 — 2026-08-12T08:15:11Z — Casey-allard

PR #634 merged (incl. the R5 conformance fix and the wasm cfg-gate fix — records at comment 5262888847 and in the PR history). Closing; #599 implementation begins now on the merged ingestion boundary.


---
_Comment/review ids in this file: 5262075235, 5262240292, 5262888847, 5264138912_

