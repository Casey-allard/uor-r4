# #957 — G0: execute local control and one-layer R4 decoder spike

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-26T02:33:12Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-26T02:53:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/957
- **Merged:** 2026-08-26T02:53:26Z
- **Merge commit:** 80a68ed0c8e3924942b3757037fd8676981c60b4
- **Base / Head:** main / UOR-Foundation:issue-950-geometric-decoder-spike
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #950

## Outcome

- Adds a pinned, provider-free SmolLM2 local control through the exact tokenizer/chat template, causal state, residual/MLP/LM-head stack, and pinned `uor-matmul` projections.
- Adds exact-tokenizer/adapter-bound router turns with identity-scoped export/import and a deterministic memory-to-layer adapter identity.
- Replaces source attention at one experimental final-layer seam with a trainable R4 query/key/value mixer using causal prefix plus persistent memory, support selected to a hard bound of four before aggregation, and `uor-matmul` projections back to the residual.
- Adds the `r4 geometric-decoder-spike` product command and retains the control/treatment transcripts, operator trace, and issue record.

## Evidence

- Five distinct 32-token source controls; no period-1 through period-4 cycle.
- Frozen operator review: 4/5 controls grammatical and prompt-responsive. P3 is recorded FAIL because it substitutes sunset for the requested morning.
- Disabled seam: 71 positions bit-identical to the ordinary source path and the same generated token IDs.
- Final-layer treatment: one decodable, cycle-free 32-token rollout; 32/32 decision traces, zero source-attention calls at the replaced layer, no dense full-prefix Q/K matrix or softmax, support bounded to four.
- Controlled coordinate permutation changed support and 48,896 final logits (`L_inf = 0.000025749207`). This is reachability only, not a quality-advantage claim.
- Router reload preserved both ordered tokenizer-bound turns exactly for the same identity; the isolation identity retrieved zero turns.
- The initial layer-0 treatment's period-2 failure remains retained as negative evidence; the final-layer repair retains the preceding 29-layer coherent substrate.

Record: `docs/geometric_decoder_spike_950.md`

## Focused verification

- `cargo fmt --check`
- `cargo check -p uor-r4-model-source --all-targets --offline`
- `cargo check -p uor-r4-router --all-targets --offline`
- `cargo check -p uor-r4-wasm-router --all-targets --offline`
- focused geometry-context, source-control/causality, persistence, product-module, and CLI parser tests
- `python3 scripts/check_claim_wording.py`
- bounded local product smoke: PASS

BDD, Gate C, kappa reproduction, fuzz, Kani, corpus-scale work, and exhaustive certification were not run because this change does not alter those contracts.

## Claim boundary

Experimental/off-serving one-layer control viability and structural reachability only. No training, geometric quality advantage, all-layer transformerless, production, performance, or multiplication-free claim.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

