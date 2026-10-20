# #950 — G0: restore coherent local uor-matmul control and execute a one-layer R4 decoder spike

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T23:29:20Z
- **Updated:** 2026-08-26T02:54:20Z
- **Closed:** 2026-08-26T02:53:27Z
- **Labels:** enhancement
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/950
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #949

## Objective

Establish the shortest end-to-end path from the repository's existing source-model components and persistent geometric router to coherent local text, bind manifold memory to the exact source tokenizer, then execute one trainable R4 geometric mixer at the source-attention seam. This is the immediate engineering issue.

## Scope

- [ ] Expose one pinned local source bundle as a deterministic greedy control through the existing tokenizer/chat template, KV state, residual/MLP stack, LM head, and `uor-matmul` projections.
- [ ] Freeze a five-prompt, 32-token smoke set and a simple grammaticality/prompt-responsiveness review rubric.
- [ ] Define the minimal `GeometryContext`: identity, ordered memory spans encoded as real source-tokenizer IDs, tokenizer CID, session/route state, provenance, and per-position geometric state.
- [ ] Define a deterministic, content-bound memory-to-layer key/value adapter and checkpoint identity before training or all-layer replacement.
- [ ] Add an experimental one-layer R4 mixer at the existing source-attention seam:
  - current residual -> learned geometric query coordinates;
  - causal prefix and retrieved memory states -> geometric keys/values;
  - declared angular/geodesic compatibility;
  - bounded sparse support selected before value aggregation;
  - value contribution returned to the residual through `uor-matmul` projections.
- [ ] Execute that seam on every decoded treatment token and emit support/logit effects for disabled, real, and controlled-permutation treatments.
- [ ] Commit the user turn and generated answer back into the identity-scoped manifold and reload it through the same tokenizer-bound adapter.

## Acceptance criteria

- [ ] The pinned source control generates 32 local tokens for all five prompts with no Ollama/provider call; at least four responses are recorded as grammatical and prompt-responsive under the frozen rubric.
- [ ] No completion enters a period-1 through period-4 cycle, and at least two prompts produce distinct token sequences.
- [ ] The transcript binds source revision/CID, tokenizer CID, chat template, decode settings, and `uor-matmul` backend.
- [ ] Disabled-mixer mode reproduces the source control for the same inputs.
- [ ] The trainable mixer executes once per generated treatment token, is causal, and never invokes the source-attention operator or a dense full-prefix Q·K matrix/softmax kernel.
- [ ] A controlled coordinate or memory permutation changes bounded support or final logits. This is a reachability gate, not a pre-training quality-advantage claim.
- [ ] The one-layer treatment completes a decodable 32-token rollout.
- [ ] Persistence/reload preserves identity, tokenizer CID, ordered memory tokens, adapter identity, and same-identity retrieval; another identity receives none.
- [ ] The CLI or a narrow product-facing executable emits retained control/treatment transcripts and the compact operator trace.

## Stop rules

- If the source control fails the five-prompt gate, fix the source-serving/tokenizer/template seam and do not debug geometry.
- If memory cannot round-trip under the exact tokenizer/identity binding, repair the adapter before fitting.
- If the mixer is not causally reachable or controlled geometry/memory perturbation cannot change support/logits, redesign the seam before training.
- Quality advantage over permuted geometry is owned by #951. Do not claim it here and do not add layers, a graph format, or a larger harness.
- Before closing with a negative/control-failure verdict, reblock #951 on the repair/replacement issue or close #951-#955 as not triggered.
- Do not claim transformerless execution from a one-layer treatment.

## Non-goals

- Train the mixer or replace every attention layer.
- Match source logits or establish broad language quality.
- Enforce P-4, `no_std`, allocation-free, witness, or packed-artifact contracts on the experimental decoder.
- Add BDD, formal proof, conformance IDs, graph lowering, or corpus-scale certification.

## Focused verification

- `cargo fmt --check`
- `cargo check -p uor-r4-model-source --all-targets --offline`
- The narrow source-control, causality, geometry-context, and persistence behavior test added by this issue.
- One bounded local product smoke producing retained five-prompt control/treatment transcripts.

## Claim status

Experimental/off-serving control viability, structural reachability, and bounded product smoke. No general coherence, geometry advantage, transformerless, production, or performance claim.


## Comments (1)

### comment 5419924964 — 2026-08-26T02:54:19Z — Casey-allard

Completed through protected merge queue in #957.

Main merge commit: `80a68ed0c8e3924942b3757037fd8676981c60b4`

DoD evidence:

- pinned provider-free SmolLM2 source control through the exact tokenizer/chat template, causal state, residual/MLP/LM head, and pinned `uor-matmul` exact GEMM owner;
- five distinct 32-token controls, no period-1 through period-4 cycle;
- frozen operator review: 4/5 grammatical and prompt-responsive (G0-P3 is explicitly FAIL for substituting sunset for morning);
- disabled seam: 71 positions bit-identical and identical generated token IDs;
- final-layer treatment: 32 decodable, cycle-free tokens; mixer executed on all 32 decisions with zero source-attention calls at the seam, no dense full-prefix Q/K or softmax, and support bounded to four before aggregation;
- controlled coordinate permutation changed support plus 48,896 final logits (`L_inf = 0.000025749207`), recorded only as structural reachability;
- tokenizer/adapter/source-bound user and assistant turns reloaded exactly for the same identity; the isolation identity retrieved zero turns;
- initial layer-0 period-2 failure retained as negative evidence; the repaired G0 seam is layer 29, preserving the preceding 29-layer coherent source substrate.

Retained record and raw transcripts: [`docs/geometric_decoder_spike_950.md`](https://github.com/UOR-Foundation/uor-r4/blob/80a68ed0c8e3924942b3757037fd8676981c60b4/docs/geometric_decoder_spike_950.md)

Focused local checks passed: fmt, touched-package all-target checks, geometry-context/source-causality/persistence/product/CLI tests, exact-executor closure, and claim wording. PR and merge-group `fast build + product smoke` passed. BDD, Gate C, kappa reproduction, Kani, and corpus-scale certification were NOT_RUN locally because their contracts were not the G0 decision gate.

Stage verdict: `PROMOTE_TO_G1`. #951 may now begin fitted real-versus-permuted advantage work; no #951 implementation was started here.



---
_Comment/review ids in this file: 5419924964_

