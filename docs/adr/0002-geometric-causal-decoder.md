# ADR-0002: Make the geometric causal decoder the active intelligence path

- **Status:** Accepted
- **Date:** 2026-08-25
- **Decision owner:** programme root
  [#820](https://github.com/UOR-Foundation/uor-r4/issues/820)
- **Implementation tracker:**
  [#949](https://github.com/UOR-Foundation/uor-r4/issues/949)
- **Roadmap:** [Geometric Causal Decoder Roadmap](../geometric_causal_decoder_plan.md)

## Context

UOR-R4's graph/table programme established scoped teacher-forced next-token
signal but did not establish coherent free-running generation. The measured
median first divergence was token zero (59 of 100 positions diverged there),
rollouts cycled, and behavior remained dominated by exact suffix-local rows.
Adding full-prefix trajectory regions made the mechanism reachable without
changing candidates or served tokens.

The repository also contains two assets with different proven roles:

- the original geometric router supplies identity-scoped state, memory,
  retrieval, and persistence; and
- the source-model runtime contains the pinned causal weights, tokenizer, KV
  path, and trace surfaces, and already owns its dense projections through
  `uor-matmul`; G0 must establish it as a coherent free-running control.

Continuing to scale, prove, or release the current graph representation would
not supply the missing causal language mechanism.

## Decision

The active programme will:

1. establish the existing local source decoder as a coherent
   `uor-matmul` control;
2. introduce a learned R⁴ causal mixer at one source-attention layer;
3. require non-vacuous real-versus-permuted geometry and student-prefix
   evidence;
4. replace remaining standard self-attention layers only while bounded
   free-running behavior survives;
5. connect persistent geometric memory directly to the mixer's causal support;
   and
6. optimize or consider integer/table lowering only after product viability.

The active decoder may use floating point, allocation, and multiplication.
Those choices do not weaken the existing P-4/`no_std`/allocation-free claims
of the frozen R4G1/TLA runtimes because the execution scopes remain separate.

## Alternatives considered

### Continue R4G1/TLA scaling

Rejected for current sequencing. The representation has pointwise signal but
the complete-prefix interventions were behaviorally inert in the product
selector.

### Promote the router's Markov generator

Rejected as the final decoder. Geometric reranking of trigram candidates does
not provide a learned causal syntax model.

### Promote dormant XOR/popcount route-attention

Rejected. It has no serving caller and its first teacher-fit attribution test
was vacuous.

### Prove or optimize the existing mechanisms first

Deferred. Formal and performance work resumes against the mechanism that
actually survives causal language evaluation.

## Consequences

- `docs/geometric_causal_decoder_plan.md` and GitHub #820/#949 replace the
  S0–S7 plan as the sequencing authority.
- R4G1/TLA, graph, proof, conformance, and negative research artifacts remain
  preserved as scoped evidence and comparators.
- ADR-0001 and `R4G1Runtime` remain the current production/default serving
  authority until G3 earns an explicit decoder promotion with a rollback path.
- “Transformerless” means zero source-attention calls and no dense full-prefix
  Q·K matrix/softmax kernel in the promoted decoder. The learned mixer forms
  declared geometric coordinates, selects bounded causal support, and must be
  load-bearing under disabled/permuted interventions. It may approximate
  teacher attention during distillation; the term does not mean
  multiplication-free.
- The source-tokenizer-to-manifold adapter is defined and qualified with the
  one-layer mixer before all-layer replacement; product integration wires that
  frozen contract rather than introducing a new memory distribution.
- `uor-matmul` remains the default learned projection backend and may remain
  in the final product.
- Product transcripts and student-prefix behavior are promotion evidence;
  teacher-forced top-1 alone is not.
- Routine development uses focused tests and a bounded product smoke.
  Certification is manual/nightly or promotion-triggered.

## Revisit conditions

Revisit this decision if:

- the G0 source-control composition cannot generate coherent local text;
- three bounded one-layer fits cannot distinguish real from permuted geometry;
- progressive replacement cannot preserve the accepted student-prefix result;
  or
- another architecture demonstrates stronger free-running behavior under the
  same local model, prompt, compute, and memory budgets.

Any revisit records the negative result and changes #820 before opening a new
mechanism programme.
