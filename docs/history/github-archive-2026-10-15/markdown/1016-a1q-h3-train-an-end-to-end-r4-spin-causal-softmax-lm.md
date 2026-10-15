# #1016 — A1Q-H3: train an end-to-end R4/Spin causal-softmax LM

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T19:08:27Z
- **Updated:** 2026-10-09T14:16:49Z
- **Closed:** 2026-08-30T20:46:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1016
- **Merged:** 2026-08-30T20:46:23Z
- **Merge commit:** 4b21f9af37bbe4918413cf52b3f9fc905dad58dd
- **Base / Head:** main / UOR-Foundation:issue-1014-end-to-end-r4-softmax-lm
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and records the single frozen #1014 end-to-end R4/Spin causal-softmax language-model campaign.

Ordinary causal attention is now established as load-bearing at the declared learned R4/Spin scope. The complete model-quality Definition of Done is negative.

## Delivered

- explicit enabled and post-Wo/pre-residual attention-off policy at the model-source seam;
- model-agnostic local HF Llama generation with deterministic seeded sampling;
- strict 32-token, 4,096-logit Python/Rust qualifier for enabled and attention-off policies;
- isolated pinned MPS trainer, content-addressed TinyStories split, sealed lifecycle, immutable selection, and HF export;
- authoritative evidence record plus synchronized README/research/roadmap/ADR/lifecycle/glossary claims.

## Binding result

- 64-sequence smoke: PASS, `80.8211%` loss reduction in `132.36s`;
- main campaign: one run, 7,155,360 parameters, 29,999,104 tokens, 1,831 steps, `70.34 min` on MPS;
- selected dev NLL: `2.1313565266`;
- enabled sealed NLL: `2.1274072772`;
- attention-off sealed NLL: `4.8047998381`;
- attention-off penalty: **`+2.6773925609` nats/token**, PASS versus `>=0.10`;
- enabled NLL quality ceiling: **FAIL**, `2.1274072772 > 1.50`;
- final Python/Rust max logit error: enabled `0.0000076294`, off `0.0000123978`, PASS versus `<=0.005`;
- all six R4/Spin layers, exact causal/projection/R4/output-policy audits, zero future/provider/Ollama/prior-trace reads: PASS;
- five autonomous Rust outputs: UTF-8 `5/5`, no period-1..4 loops `5/5`, exact replay `5/5`;
- subject/scene retention: **FAIL**, `3/5 < 4/5`.

Terminal: `ATTENTION_ESTABLISHED__FULL_QUALITY_DOD_FAIL`.

Per the frozen contract, this campaign will not be rerun or tuned. #1017 is the sole successor: continue the exact checkpoint once on the next 119,996,416 fresh deterministic training tokens and fresh confirmation tranches. It does not repeat attention-off or open another diagnostic ladder.

## Nonclaims

This does not establish geometry advantage over ordinary coordinates, transformerlessness, multiplication-free/table-native execution, dependable general generation, reasoning, chat, browser-WASM readiness, release readiness, or frontier capability.

References #1014
References #973
References #1017

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

