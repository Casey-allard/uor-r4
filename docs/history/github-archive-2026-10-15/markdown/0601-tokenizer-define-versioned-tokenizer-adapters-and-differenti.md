# #601 — tokenizer: define versioned tokenizer adapters and differential source fixtures

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T21:59:02Z
- **Updated:** 2026-08-12T11:45:58Z
- **Closed:** 2026-08-12T11:45:58Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/601
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The repository now has a correct, tested byte-level BPE implementation for the current SmolLM2 HF path. Compiling more permissively licensed model families requires the tokenizer boundary to carry family/version/policy identity without regressing the legacy tokenizer or pretending that unsupported normalization and pre-tokenization are equivalent.

# Current evidence

- `crates/uor-r4-core/src/transformerless/hf_bpe.rs` implements `HfBpeTokenizer` and `TokenizerKind`, including vocab/merge ranks, added-token matching, GPT-2 byte mapping, ByteLevel, and SmolLM2's Digits pre-tokenizer.
- The module explicitly rejects unsupported pre-tokenizer configurations; it has no general adapter trait or SentencePiece/Unigram implementation.
- Merged #242/#253 fixed the former bespoke HF tokenizer and recorded a real tokenizer address; #285 completed the serving-side BPE follow-through. Those issues are complete and must not be reopened as duplicates.
- `crates/uor-r4-graph-compiler/src/observation_text.rs` and `crates/uor-r4-graph-cli/src/lib.rs` select `TokenizerKind`; `src/tless_uor.rs` still documents a serving-side thread-through concern for broader source selection.

# Problem

Tokenizer family, normalization, pre-tokenization, added-token, BOS/EOS, chat-template, and raw-byte decoding policies are not represented by a versioned adapter contract. A future family can therefore either be rejected without a path forward or accidentally approximated.

# Goal

Define a versioned host-side tokenizer-adapter boundary and differential fixtures, extending the existing `TokenizerKind` selection instead of introducing a parallel tokenizer API.

# Scope

- Specify the adapter metadata and version fields needed for tokenizer identity and policy.
- Keep the current byte-level BPE adapter as V1 and add differential fixtures for encoding, decoding bytes, added tokens, normalization/pre-tokenization, BOS/EOS, and chat-template rendering where declared by the source snapshot.
- Add one separate, explicitly versioned SentencePiece/Unigram adapter only when its source fixture and license/revision are selected; otherwise leave the family as an explicit rejection.
- Ensure observation, evaluation, serving prompt encoding, and exported runtime tokenizer use the same selected adapter and CID.

# Non-goals

- Do not change the legacy llama2.c tokenizer or historical κ-pinned artifacts.
- Do not make tokenizer implementation a deployed floating-point or allocation-heavy runtime dependency.
- Do not bundle a model-family adapter or source snapshot without a pinned revision and license record.

# Dependencies and blockers

- Depends on #597 for tokenizer-file/special-token/chat-template snapshot identity and coordinates with #599 for source conformance.
- #515 owns dormant mechanism/feature gating; #531 owns corpus saturation; #589 blocks upstream integrations.

# Proposed implementation shape

Turn the existing `TokenizerKind`/`HfBpeTokenizer` surface into a versioned adapter registry with a small common encode/decode/provenance contract. Keep family-specific parsing in host/compiler code, reject unknown policies explicitly, and use differential fixtures against the source tokenizer implementation.

# Acceptance criteria

- [ ] The current SmolLM2 byte-level BPE path passes its existing tests plus versioned differential fixtures.
- [ ] Adapter identity commits tokenizer bytes, family, policy/version, special-token configuration, and chat-template policy.
- [ ] Unsupported normalizers/pre-tokenizers/tokenizer families fail explicitly rather than silently approximating.
- [ ] Observation, evaluation, serving, and exported tokenizer paths agree on token IDs and decode bytes for the selected adapter.
- [ ] A chosen non-BPE family is either implemented with fixtures or recorded as a bounded rejection with a follow-up issue.

# Verification

- Differential encode/decode fixtures including Unicode, byte fallbacks, added tokens, BOS/EOS, and chat templates.
- Deterministic tokenizer CID and artifact-provenance tests.
- Existing `hf_bpe_tokenizer` and model adapter suites, workspace fmt/clippy/tests, no-std graph-format checks, and the claim-wording checker.

# Compatibility and migration

The existing HfBpe/legacy selection and tokenizer CIDs remain valid. New adapter versions or policy interpretations must produce new tokenizer/observation identities and must not mutate historical artifacts.

# Claim status

Definition, implementation, and empirical differential criterion. Tokenizer parity is a prerequisite for trustworthy observations, not a claim of model-quality parity.


## Comments (3)

### comment 5265660501 — 2026-08-12T10:45:00Z — Casey-allard

## Plan

1. **Versioned adapter contract by extension, not parallel API**: the existing `TokenizerKind`/`HfBpeTokenizer` surface becomes a versioned registry — current byte-level BPE = adapter `hf-byte-bpe/1` — with a typed identity record committing tokenizer bytes (CID), family, policy/version, added-token config, BOS/EOS, and chat-template policy (never-interpreted stays explicit, per #599).
2. **Differential fixtures**: encode/decode tables incl. Unicode boundaries, byte fallbacks, added tokens, BOS/EOS, digits pre-tokenization — run against the same adapter the observation, evaluation, serving-prompt, and exported-runtime paths select (agreement asserted, one adapter + one CID).
3. **Fail-closed families**: unsupported normalizers/pre-tokenizers/families remain explicit rejections; SentencePiece/Unigram is recorded as a **bounded rejection + follow-up issue** (no pinned source snapshot with license/revision is selected, which the issue makes the precondition for implementing it).
4. Compatibility: legacy llama2.c tokenizer and historical κ-pinned artifacts untouched; new identities are additive (serde(default)); existing hf_bpe suites stay green.

Six gates + full suite before the PR; record + docs on close.

### comment 5265894457 — 2026-08-12T11:09:35Z — Casey-allard

## Delivery record (PR #638 in queue)

Acceptance: **(1)** SmolLM2 byte-level BPE path passes existing suites + new versioned differential fixtures (full-256-byte-alphabet fixture; pinned ids/bytes across ASCII merges, Unicode incl. CJK + emoji ZWJ, byte fallback, added tokens, BOS/EOS, digits-boundary differentials — pinning the post-#242/#253 verified behavior as the baseline so any drift fails; HF snapshot absent from env, recorded honestly). **(2)** Adapter identity commits tokenizer bytes (existing blake3 CID rule unchanged), family `hf-byte-bpe/1`, policy (normalizer, pre-tokenizers, byte-fallback, added-tokens digest), BOS/EOS, chat-template policy — with a declared-identity digest over pinned canonical bytes. **(3)** Unknown families/versions fail by name through the sanctioned surface (R5-conformant `UnknownTokenizerAdapter`). **(4)** Observation, evaluation, serving-selection, and exported-runtime-tokenizer paths asserted to agree on adapter identity, token IDs, and raw decode bytes (seam-level for the server's private static; engine CID rule reproduced rather than full-engine-constructed — both bounds recorded in the test docs). **(5)** SentencePiece/Unigram = bounded rejection + follow-up filed: https://github.com/UOR-Foundation/uor-r4/issues/639. Provenance: `ObservationManifest.tokenizer_adapter` serde(default), auto-populated on observe-text paths, legacy bytes unchanged when unset. Legacy llama2.c tokenizer and historical κs untouched. Six gates green (127 suites). Closes on #638 merge; #602 (attention operator spec) next.

### comment 5266379908 — 2026-08-12T11:45:58Z — Casey-allard

PR #638 merged — delivered per the record (comment 5265894457); SentencePiece follow-up tracked in #639. Closing; #602 begins now.


---
_Comment/review ids in this file: 5265660501, 5265894457, 5266379908_

