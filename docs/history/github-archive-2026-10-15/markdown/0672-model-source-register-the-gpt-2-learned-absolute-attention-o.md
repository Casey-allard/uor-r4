# #672 — model-source: register the GPT-2 learned-absolute attention operator (#668)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T18:58:55Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-08-13T19:19:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/672
- **Merged:** 2026-08-13T19:19:46Z
- **Merge commit:** 56feef50b72a9cf0b72fcc409dee9297247109a5
- **Base / Head:** main / UOR-Foundation:i668-gpt2-attn-operator
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #657 (item 3). Closes #668.

## Problem
`HuggingFaceGpt2Oracle::attention_operator_spec` returned `None` by design: the only registered operator, `standard-source-attention/1`, declares `positional_action = rope-rotation-of-q-and-k-before-scoring`, which GPT-2 does **not** do (learned absolute positions). Recording it would be a false operator identity, so the GPT-2 certificate operator row (#606) was absent rather than misdeclared.

## Change
Register a new versioned source operator `learned-absolute-source-attention/1` in `uor-r4-model-source::attention`, and wire the GPT-2 oracle to it.

The record is a **truthful inventory** read from the executor (`gpt2::Gpt2Model::layer_forward`), following the same implementation-lives-elsewhere discipline the #604 target operator uses:

- **projections**: fused `c_attn` Conv1D `[n_embd, 3·n_embd]` applied as `x @ W + b`, split into q/k/v thirds, **with bias** (not Llama's separate biasless `wq`/`wk`/`wv`).
- **positional_action**: NONE on q/k — GPT-2 adds a learned absolute position embedding (`wpe`) to the input residual; nothing rotates q or k. This is the field that forbids reusing `standard-source-attention/1`.
- **compatibility_relation / selector**: the same `scaled-dot-product` + max-subtracted softmax as the standard operator.
- **output_projection**: `c_proj` Conv1D with bias.
- **head selection**: plain multi-head (kv heads == query heads, no grouped-query).
- **score_scale**: precomputed `1/sqrt(head_size)` **multiply** (distinct from the standard per-score divide).

The versioned registry (`operator_spec`) still refuses unknown `(id, version)` on the sanctioned `UnknownAttentionOperator` surface. `MODEL_LIFECYCLE.md`'s operator inventory is updated in lockstep.

## Regression gate
**SmolLM2/Llama is byte-unchanged**: the Llama oracle still returns `operator_for_r4_switch`, and the two source operators + the #604 target operator are untouched. The GPT-2 tiny-fixture numpy parity test is unchanged and green — the executor arithmetic is not modified, only its provenance record is now declared.

## Verification (all local, pre-push)
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clean)
- `cargo build --target wasm32-unknown-unknown --lib` (clean)
- `cargo test -p uor-r4-model-source`: 66 lib + 15 attention tests pass, including:
  - `attention::learned_absolute_canonical_serialization_is_byte_stable` — pins the canonical bytes + blake3 declared-identity digest, distinct from the other operators.
  - `attention::registry_resolves_all_registered_operators` / `registry_refuses_unknown_id_and_version_by_name` — extended for the new `(id, 1)` and its unknown `(id, 2)`.
  - `gpt2::gpt2_oracle_reports_learned_absolute_operator` — the oracle reports the record and it round-trips through the registry (loaded from the committed tiny fixture).
- Register conformance R1–R5: `check-model`, `repo-conformance`, `audit-deferral`, `audit-limits` all green. No new dependencies (R6/cargo-deny dep-neutral).

Claim discipline unchanged: this makes the operator record truthful; it does not compile or certify GPT-2 (items 2/4/5 remain: #667, #669, #670).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

