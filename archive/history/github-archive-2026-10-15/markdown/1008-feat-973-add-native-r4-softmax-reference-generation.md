# #1008 — feat(#973): add native R4 softmax reference generation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T07:52:51Z
- **Updated:** 2026-10-09T14:16:55Z
- **Closed:** 2026-08-30T07:54:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1008
- **Merged:** 2026-08-30T07:54:18Z
- **Merge commit:** 184bdb939ec8b14ba16c87de749b51735fecd960
- **Base / Head:** main / UOR-Foundation:issue-973-r4-softmax-generation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973

## Outcome

- publishes the protected score/readout localization result at `REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT` and parks intrinsic score/readout, resonance, recurrence, and lowering research;
- adds `R4SoftmaxReferenceGeneratorV1` and the native `r4 r4-softmax-generate` CLI;
- keeps the pinned SmolLM2 embeddings, RoPE, learned Q/K/V, stable causal softmax, value aggregation, residual/RMSNorm/MLP, final normalization, and LM head while transporting every selected attention layer through the qualified coherent R4/Spin seam;
- publishes the compact 12-report evidence aggregate and synchronizes the living programme documentation plus append-only historical records.

## Frozen result

- terminal: `PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`;
- decoded quality: 4/5 under the frozen rubric in both passes; G0-P3 honestly fails because it substitutes sunset for morning;
- deterministic replay: 5/5 exact after deleting only timing;
- all 30 decoder layers selected;
- causal, projection, and R4 observed/expected audits exact; future reads zero;
- donor behavior: G0-P1 matches through EOS; G0-P2 through G0-P5 match all 32 retained token IDs;
- evidence record BLAKE3: `34b6d7ddcdb858092ec7b192338fc236e5dfc3768fd8f4fb05dbe4ff2a23e82e`;
- aggregate BLAKE3: `580360f44fe7163e697043ab8d6f801f578a5e7ed3d905209a166e7d303af6d0`.

## Attribution and boundaries

HELM-D is credited as the MIT-licensed architectural reference at upstream commit `7501deca8f413848bfef804be64ce874b72a3cd7`. No HELM checkpoint or HELM generation implementation executes here. The decoder is UOR's existing pinned SmolLM2 `HuggingFaceLlamaOracle` path.

This result is provider-free native reference generation, but remains source-weight-backed, floating-point/matmul, allocating, and Transformer-compatible. It does not establish geometry advantage, replace softmax, produce a source-free/table-native runtime, unblock #954, authorize a release tag, or make the static WASM page a model host. The next bounded action is an explicit opt-in native HTTP/dashboard bridge with no default-engine change.

## Verification

- `cargo test -p uor-r4-wasm-router --lib r4_softmax_reference_generation --offline` — 5/5 pass
- `cargo test -p uor-r4-wasm-router --bin r4 parses_r4_softmax_generate_without_rewriting_prompt_bytes --offline` — pass
- `cargo clippy -p uor-r4-wasm-router --all-targets --all-features --offline -- -D warnings` — pass
- `cargo fmt --check` — pass
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline` — pass (pre-existing target warnings only)
- claim-wording gate, JSON parse, relative-link audit, and diff check — pass
- all 12 local raw-report BLAKE3 bindings and all five replay comparisons — pass


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

