# G0 local control and one-layer geometric decoder spike (#950)

Date: 2026-08-26

Verdict: **PROMOTE_TO_G1**

Claim scope: experimental off-serving local-control viability, one-layer causal
reachability, and bounded product smoke only. This record does not claim a
geometric quality advantage, all-layer transformerless execution, production
readiness, performance, or multiplication-free execution.

The machine-readable transcripts and per-token operator trace are retained in
[`geometric_decoder_spike_950_raw.json`](geometric_decoder_spike_950_raw.json).
The first, negative layer-0 treatment is retained in
[`geometric_decoder_spike_950_control_raw.json`](geometric_decoder_spike_950_control_raw.json)
rather than being rewritten as a pass.

## Frozen contract

- Source: `HuggingFaceTB/SmolLM2-135M-Instruct` revision
  `7e27bd9f95328f0f3b08261d1252705110c806f8`
- Weights CID:
  `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`
- Tokenizer CID:
  `blake3:944d1262d516abd56a8156dd3058a73a1bf3dc19419527592d854d162f288073`
- Memory adapter identity:
  `blake3:7fa11963f23f0e7f6b69a62129a2c01f8de71099d68a622b515a35e3ceffba44`
- Mixer checkpoint identity:
  `blake3:a63eed546dc6b88a6806f43a238f458f2ec276c7baf6034452ed21fc9baf1fc9`
- Projection owner: `uor-matmul exact GEMM` at pinned revision
  `b13c98449948174f590e337c4dc25dfc394a07d0`, four fixed workers
- Decode: deterministic greedy argmax, lower token ID on an exact tie, exactly
  32 retained decisions per completion
- Cycle rejection: periods 1, 2, 3, and 4
- Frozen review rubric: a response passes when it contains an intelligible
  English clause or structured list item without corruption/repetition and
  directly performs the requested task. A cap-truncated final clause is
  acceptable only when earlier material is complete and responsive. Four of
  five controls are required.

The local command loaded the pinned snapshot directly. It made no Ollama,
hosted-provider, or network inference call.

## Source-control review

| ID | Retained rendered response | Grammar | Responsive | Result |
| --- | --- | --- | --- | --- |
| G0-P1 | “Plants need sunlight to undergo photosynthesis, a process that converts light energy into chemical energy stored in glucose, which they use for growth and development.” | PASS | PASS: directly explains why sunlight is needed | PASS |
| G0-P2 | “Here are three practical tips for staying organized at work: 1. **Create a To-Do List:** Start by setting aside a few minutes each day to” | PASS: structured item; final explanation is cap-truncated | PASS: directly begins the requested organized-work tips | PASS |
| G0-P3 | “As the sun began to set, casting a warm orange glow over the bustling streets of the city, a gentle rain poured down in a relentless stream, washing away” | PASS: intelligible clause; cap-truncated | FAIL: substitutes sunset for the requested morning | FAIL |
| G0-P4 | “When you ride a bicycle, you're essentially using a system of gears and levers to move your body. The bicycle's balance is achieved through a combination of the” | PASS: one complete sentence before truncation | PASS: directly begins a simple balance explanation | PASS |
| G0-P5 | “Hi [TeamName], welcome to our team. We're excited to have you on board and look forward to working together to achieve our goals. We value” | PASS: two complete sentences before truncation | PASS: friendly welcome and “working together” satisfy collaboration | PASS |

Operator review: **4/5 PASS**. All five controls retained 32 decisions, none
entered a period-1 through period-4 cycle, and all five token sequences were
distinct.

## One-layer treatment

The initial layer-0 replacement was structurally reachable but produced a
period-2 EOS cycle. Its runtime verdict is `FAIL` in the retained negative
artifact. Under the issue stop rule, the seam was redesigned before any
training: G0 now replaces source attention only at final layer 29, retaining
the preceding 29-layer coherent source substrate. Earlier-layer fitting and
real-versus-permuted quality belong to #951.

The final treatment produced 32 decodable, cycle-free tokens:

> Plants need sunlight to undergo photosynthesis, a process that converts
> light energy into chemical energy, which is then used to produce glucose and
> other essential nutrients.

For all 32 retained treatment decisions, the mixer executed at layer 29 with
zero source-attention calls, no dense full-prefix Q/K or softmax operation, and
support bounded to four entries selected before aggregation. The candidate
prefix grew causally from 40 through 71 positions. One tokenizer-bound user
memory span (10 source tokens) was available throughout.

The controlled coordinate permutation changed the selected support CID from
`blake3:d75512e3d418fcece1b3e712160dbc6c00415e22236721ab80682b5c871da68f`
to
`blake3:cccf57aa4863f607a57817ec15a744a16b42a4f783611fc62516b63fb6a5e0bf`
at the first treatment decision. It changed 48,896 final logits with
`L_inf = 0.000025749207`. This is structural reachability, not evidence of a
quality advantage.

Disabled-mixer replay compared 71 positions bit-for-bit, retained the same 32
generated token IDs as the source control, and emitted no mixer trace.

## Persistence and terminal gates

The product command committed one user turn and the generated assistant turn,
exported router state, reloaded it into a new router, and retrieved both turns
exactly under the same identity. Token order, tokenizer CID, adapter identity,
and source binding were preserved. The isolation identity retrieved zero turns.

Terminal machine gates:

- five 32-token controls: PASS
- 4/5 frozen human control rubric: PASS
- short-cycle rejection and sequence diversity: PASS
- disabled-source equivalence: PASS
- per-decision causal/bounded mixer execution: PASS
- controlled-permutation reachability: PASS
- decodable 32-token treatment: PASS
- identity/tokenizer-bound persistence and reload: PASS

Decision: G0 is complete and #951 may begin fitted real-versus-permuted
advantage work. No #951 implementation was started here.
