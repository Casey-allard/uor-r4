# #635 — feat(#599): adapter conformance gates + source-executor parity fixtures

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T08:42:44Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-12T09:01:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/635
- **Merged:** 2026-08-12T09:01:55Z
- **Merge commit:** a30a719cf2b0fc3130e024f7156e791259fc8302
- **Base / Head:** main / UOR-Foundation:i599-adapter-conformance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#599): adapter conformance gates + source-executor parity fixtures

Typed AdapterFeatures declaration validated against raw config.json at
oracle construction - BEFORE typed parse, snapshot ingestion, or any
observation. Every assumed Llama-family feature enumerated with a
positive fixture or explicit rejection test (12 rows: model_type,
activation, norm epsilon - previously the executor silently ignored
rms_norm_eps and hardcoded 1e-5, now fail-closed - RoPE mode/theta/
scaling, GQA/MQA geometry incl. odd head size, QKV/MLP biases, tied/
untied embeddings, token-id policy, chat-template never-interpreted).
Failure classes ride the sanctioned SourceUnavailable as
UnsupportedConfigFeature{AdapterFeature,...} (R5-conformant). Fixture
schema uor-r4-adapter-fixture/1 (canonical JSON: prompts, tokenizer
ids/bytes, bounded per-layer captures, final hidden, logits, top-k,
documented tolerances, source-manifest binding + identities).
Deterministic runner: PASS / FAIL(per-check deltas) / UNAVAILABLE
(named missing prerequisite - never vacuous-skip; the absent pinned
SmolLM2 snapshot is asserted UNAVAILABLE by a non-ignored test);
double-run reports byte-identical. Llama::forward factored into
layer_forward/finish_forward with arithmetic order untouched
(bit-exact batch parity test still green); capture path host-gated.
27 new tests; no new deps; runtime and #307 surfaces untouched; wasm
per-item cfg gates; docs/MODEL_LIFECYCLE.md section added.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

