# Geometric Causal Decoder Roadmap

- **Status:** Authoritative for current intelligence sequencing.
- **Adopted:** 2026-08-25 through GitHub programme root
  [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) and roadmap reset
  [#948](https://github.com/UOR-Foundation/uor-r4/issues/948).
- **Execution tracker:** [#949](https://github.com/UOR-Foundation/uor-r4/issues/949).
- **Supersedes for sequencing:** `docs/r4_intelligence_completion_plan.md` and
  `docs/r4_graph_compiler_implementation_plan.md`.

The superseded plans and their evidence remain part of the repository's history.
They no longer decide what is built next.

## Objective

Build a local, CPU-first geometric language model that produces coherent
free-running text inside UOR-R4, uses the pinned `uor-matmul` backend for
learned dense projections, makes persistent R⁴ geometry causally load-bearing
in next-token selection, and progressively replaces every standard causal
self-attention block with a learned geometric mixer.

The project is successful when the local product—not only a certifier or
teacher-forced harness—can hold a prompt-responsive conversation through this
decoder and retain the conversation in its identity-scoped manifold.

## Why the architecture changed

The S0–S4 programme produced decisive evidence:

- The R4G1/TLA table and graph paths carry scoped teacher-forced next-token
  signal.
- That signal does not compose into coherent free-running generation. The
  measured median first divergence is token zero, short cycles dominate, and
  complete-prefix trajectory regions were reachable but behaviorally inert.
- The tested XOR/popcount route-attention, W(3,3), static graph, and planning
  mechanisms did not establish geometric language generation or reasoning.
- The original prime-router contribution survives and works as geometric
  context, identity state, retrieval, and persistent memory.
- The pinned source weights, tokenizer, causal KV path, trace taps, and dense
  `uor-matmul` projections are already present in this repository. G0 must
  establish that their local free-running composition is a coherent control;
  it is not assumed from the existence of teacher-forced forwards.

The strict measurements are not being weakened. They showed that the current
representation is a teacher-forced retrieval/continuation system. The response
is to change the architecture and shorten the development loop.

### Architectural origin

The reset returns to the working division of responsibility demonstrated by
the original [prime-router](https://github.com/Casey-allard/prime-router):
geometry maintained context and persisted both sides of a conversation, while
an external causal model supplied syntax. Its every-fourth-token injection was
context augmentation, not a standalone decoder, but it proved the value of a
durable geometric context layer.

UOR-R4 now removes that external dependency without asking static graph tables
or the word-Markov baseline to invent syntax. The pinned local source runtime
is first established as the behavioral control, then its causal
self-attention is replaced progressively by learned R⁴ mixing. The router's
identity-scoped manifold remains the memory and writeback substrate.

## Architecture decision

### Decision

Use the existing source-model forward path as a coherent behavioral control.
Introduce a learned, causal R⁴ mixer at its attention seam, qualify one layer
under real and permuted geometry, and replace the remaining self-attention
layers only while student-prefix coherence survives.

Persistent router memories enter the same causal geometric support as prior
token states. They do not remain only prompt decoration.

```text
token history + identity-scoped geometric memory
                         |
                         v
       learned causal R⁴ query/key/value geometry
       + bounded sparse prefix/memory neighborhood
                         |
                         v
      residual + norm + nonlinear layers + LM head
                 through uor-matmul
                         |
                         v
                  next-token logits
                         |
                         +--> commit the completed turn to the manifold
```

### Why this option

It preserves the language machinery that demonstrably produces syntax while
isolating the exact component the project intends to replace. It also reuses
the working geometric memory system instead of rebuilding it and puts geometry
on the actual pre-token causal path where its effect can be falsified.

### Options not selected

- **Continue scaling R4G1/TLA tables and graphs.** Rejected as current
  sequencing because complete-prefix additions were reachable but inert and
  free-running behavior remained suffix-local and cyclic.
- **Promote the word-level geometric Markov generator.** Rejected as the final
  decoder because its candidates come from bigram/trigram transitions.
- **Promote XOR/popcount route-attention.** Frozen as a comparator because its
  first teacher-fit attribution instrument was vacuous and it has no serving
  caller.
- **Formalize or optimize before coherent generation.** Deferred because it
  would stabilize the wrong mechanism.

### Consequences

- Floating point, allocation, learned projections, and `uor-matmul` are
  allowed in the active decoder lane.
- Existing P-4, `no_std`, allocation-free, witness, and packed-format
  guarantees remain valid only for the frozen runtime components that already
  own them.
- The first decoder checkpoint is experimental and off the production path.
- Transformerless promotion and multiplication-free promotion are separate
  decisions.

## Definitions

**Geometric decoder:** A causal token decoder in which a learned R⁴,
quaternion, angular, or geodesic state participates before token selection at
every replaced layer.

**GeometryContext:** A bounded, token-aligned decoder input containing identity,
session/route state, ordered memory spans as real tokenizer IDs, provenance, and
per-position geometric keys or affinities.

**Transformerless decoder:** A promoted decoder with zero calls to the source
self-attention operator and no dense full-prefix Q·K matrix/softmax kernel. Its
learned mixer must form declared geometric coordinates, select a bounded causal
neighborhood before value aggregation, and demonstrate through disabled and
permuted interventions that this geometry affects support or logits. The
geometric relation may deliberately approximate teacher attention during
distillation; rank-order non-equivalence is not assumed. The decoder may retain
the tokenizer, embeddings, residual stream, normalization, MLP/SwiGLU, LM
head, and `uor-matmul` projections.

**Multiplication-free runtime:** A separate deployment property. It is not a
synonym for transformerless and is not a prerequisite for coherent generation.

**Product viability:** Decodable, prompt-responsive, non-cycling free-running
text through the real CLI/HTTP path, with identity-scoped memory round trips.
Teacher-forced pointwise accuracy alone is not product viability.

## Existing mechanism disposition

| Disposition | Mechanisms |
|---|---|
| Reuse | `uor-r4-router` identity/session state, content-bearing memory, retrieval, Hopf/R⁴ math, persistence, turn writeback |
| Reuse | `uor-r4-model-source` tokenizer, source weights, causal KV/residual/MLP/LM-head runtime, trace taps |
| Reuse | `uor-matmul` as owner of learned dense projections |
| Reuse | Source bundles, corpus/model lifecycle, bounded reporting utilities |
| Freeze as comparator | R4G1/TLA compiler, packed runtime, EXCT/NGRAM selectors, witnesses, pointwise measurements |
| Freeze as comparator | XOR/popcount route-attention, W(3,3) planning, static trajectory regions, proof/conformance assets |
| Replace on promoted path | Word-level Markov generation and arrival-ordered word/token semantics |
| Remove from intelligence path | Hash-derived thought streams and visualization-only trajectories |
| Progressively replace | Standard causal self-attention |

Frozen work is preserved. Its historical evidence remains true at its declared
artifact, population, selector, and execution scope.

## Current GitHub roadmap

Native parent/sub-issue and `blockedBy` relationships are the source of truth.

| Order | Issue | Exact objective | Entry condition |
|---:|---|---|---|
| Now | [#948](https://github.com/UOR-Foundation/uor-r4/issues/948) | Publish this reset across GitHub and repository entry points | None; active |
| G0 | [#950](https://github.com/UOR-Foundation/uor-r4/issues/950) | Establish a five-prompt coherent control, tokenizer-bound memory adapter, and one trainable R⁴ mixer seam | #948 merged |
| G1 | [#951](https://github.com/UOR-Foundation/uor-r4/issues/951) | Train and qualify that mixer/adapter on teacher and student prefixes against disabled/permuted geometry and memory | #950 promotes |
| G1R | [#958](https://github.com/UOR-Foundation/uor-r4/issues/958) | Redesign the layer-29 representation after #951 learned support/memory but missed the held-out operator gate | #951 records `REDESIGN_REPRESENTATION` |
| G2 | [#952](https://github.com/UOR-Foundation/uor-r4/issues/952) | Progressively replace every standard causal self-attention block | #958 records `PROMOTE_TO_ALL_LAYERS` |
| G3 | [#953](https://github.com/UOR-Foundation/uor-r4/issues/953) | Integrate the all-layer decoder and persistent manifold memory into CLI/HTTP product paths | #952 accepted |
| G4 | [#954](https://github.com/UOR-Foundation/uor-r4/issues/954) | Profile and optimize only the dominant measured CPU/RSS bottleneck | #953 promoted |
| G5 | [#955](https://github.com/UOR-Foundation/uor-r4/issues/955) | Freeze the bounded capability and decide retain-`uor-matmul` versus optional lowering | #954 complete |

```text
#948 -> #950 -> #951 -> #958 -> #952 -> #953 -> #954 -> #955
          \________________________________________________/
                             tracker #949
```

Only the earliest unblocked leaf is assigned and worked. Downstream issues stay
unassigned until their blocker closes.

A negative or `REDESIGN` stage verdict does not make its successor executable.
Before closing that blocker, update the native graph by reblocking the successor
on the replacement-design issue or closing the downstream chain as not
triggered. A closed GitHub blocker alone is not a promotion decision.

## Stage contracts

### G0 — coherent control plus one-layer spike

The control uses a frozen five-prompt smoke set and must generate 32 local
tokens per prompt without Ollama. A recorded rubric review must mark at least
four responses grammatical and prompt-responsive, and no completion may
enter a period-1 through period-4 cycle. The transcript binds the exact source,
tokenizer, chat template, decode, and `uor-matmul` backend. If the control fails,
geometry work stops.

G0 also defines the memory adapter before mixer training: source-tokenizer CID,
ordered token spans, adapter/checkpoint identity, and a deterministic
memory-to-layer key/value projection. The trainable one-layer seam must execute
on every decoded treatment token and emit support/logit effects. Before fitting,
its gate is causal reachability only: a controlled coordinate or memory
perturbation must change support or logits. Quality advantage over permuted
geometry belongs to G1.

### G1 — bounded training and one-layer qualification

Reuse existing teacher trace surfaces and freeze all source-model weights.
Build only a mixer-specific trainer and checkpoint, not a generic optimizer or
autograd framework. A tiny synthetic batch must overfit before real trace work;
checkpoint save/reload must reproduce the same output. Then use at most 4,096
train and 512 held-out positions, include student-created prefixes and
real/permuted persistent-memory examples, and cap each of at most three fitting
rounds at one hour. Real geometry must improve the frozen primary held-out loss
by at least 5% relative to its permutation without worsening the bounded
rollout into short cycles. Otherwise redesign before adding layers.

**Outcome append (2026-08-26, #951).** The three-round frozen run improved the
primary held-out loss by 4.4416% versus coordinate permutation, below the 5%
gate. Teacher and student advantages were 4.4031% and 4.5190%, respectively.
The support term improved and memory-only permutation worsened the declared
memory metric, but operator-alignment and sampled-token terms remained
effectively flat. The exact terminal verdict is `REDESIGN_REPRESENTATION`;
[#958](https://github.com/UOR-Foundation/uor-r4/issues/958) owns the one-layer
representation redesign and blocks G2. See
[`geometric_mixer_qualification_951.md`](geometric_mixer_qualification_951.md).

### G1R — representation redesign

Diagnose the reachable operator subspace and per-loss gradient scale on the
frozen #951 positions before changing the layer-29 representation or
memory-value transport. Preserve source-weight freezing, bounded causal
support, tokenizer binding, student prefixes, matched nulls, and
`uor-matmul` ownership. A renewed bounded qualification must explicitly return
`PROMOTE_TO_ALL_LAYERS`; otherwise abandon layerwise replacement. No G2 layer
work begins inside G1R.

### G2 — progressive replacement

Replace layers incrementally and retain the last accepted checkpoint. Do not
label a hybrid or incoherent checkpoint transformerless. Do not mask a failed
operator with repetition penalties, response postprocessing, or graph fallback.
The final census must show zero source-attention calls, zero dense full-prefix
Q·K matrix/softmax kernels, and one bounded, intervention-qualified geometric
neighborhood selection at every replaced layer and decoded token.

### G3 — product integration

The CLI and HTTP surfaces share one decoder implementation. This stage wires
the tokenizer-bound, trained memory adapter already qualified in G0/G1; it does
not introduce a new memory representation or input distribution after
all-layer training. A two-turn exact-recall probe, disabled and permuted-memory
nulls, identity isolation, persistence across restart, and a five-turn
transcript decide promotion.

### G4 — measured optimization

Profile first. Optimize one component only if it owns at least 20% of the
measured target cost. Stop an intervention that yields less than 10%
improvement. Do not trade away the accepted checkpoint or product behavior.

### G5 — bounded release and lowering decision

Package the accepted decoder, reproduce the bounded product gates, and publish
its exact capability limits. Retaining `uor-matmul` is the default valid
outcome. Open a lowering successor only when measured reachability and a small
prototype justify it.

## Research requirements

Research exists to make the next product decision. Every new experiment must:

1. Name the active decoder issue and the exact decision it can change.
2. Show that the mechanism is reachable in the real token path before an
   expensive run.
3. Reuse an existing operator, trace, corpus, or reporting surface before
   adding infrastructure.
4. Compare with a non-degenerate disabled/shuffled/permuted control.
5. Include student-prefix free-running evidence when generation is in scope.
6. Start with the smallest prompt set or batch that can falsify the mechanism.
7. Predeclare positive, negative, and stop actions; negative must not mean
   “build a larger harness.”
8. Record unavailable fixtures as `UNAVAILABLE`, never pass.
9. Preserve negative results and revise live summaries without rewriting
   historical records.
10. Avoid a new graph format, proof lane, benchmark framework, or corpus-scale
    run until the active vertical slice demonstrates decision value.

For any run measured in hours, retain the existing run contract: reachability
arithmetic, binding cheap instrument, exit rule, distinct outcome actions, and
wall-clock/resource ceiling.

## Verification budget

Normal product work runs:

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> --lib <focused-test> --offline
```

It also produces one bounded transcript or operator report that exercises the
changed behavior. Docs/claim changes run
`python3 scripts/check_claim_wording.py`.

Workspace, BDD, doctest, `no_std`, deterministic-rebuild, κ, Gate C,
all-features, WASM, fuzz, Kani, audit, conformance, and corpus-scale suites are
nightly/manual certification unless the issue directly changes that contract.
No new suite becomes a merge-queue requirement without a separate maintainer
decision.

## Claim boundaries

Until G2 completes, the active decoder is hybrid and experimental. Until G3
completes, it is not the product default. Until G5 records its decision, no
release or multiplication-free claim is promoted.

Readable samples are necessary product evidence but do not prove general
language quality, reasoning, factuality, or geometric advantage. Those claims
require their own bounded evaluations after coherent generation exists.

## Historical programme disposition

- Closed S0–S4 issues and their evidence remain preserved with their recorded
  completed, LIMIT, INERT, NOT_RUN, or negative outcomes.
- S5–S7 issues #825/#827/#828 and leaves #847–#858 are closed not planned or
  not triggered. They scale, instruct, prove, or distribute the superseded
  graph product before coherent generation.
- #859 is closed not planned under the current architecture. Formalization
  resumes only after the learned causal geometry stabilizes.
- #940 remains a separate administrator-blocked CI cleanup, not an intelligence
  dependency.

## Completion

Programme tracker #949 closes only when a pinned all-layer decoder produces the
bounded product behavior, invokes zero source-attention operators and no dense
full-prefix Q·K matrix/softmax kernel, uses `uor-matmul` for learned projections,
demonstrates causal effect from real geometry and persistent memory over their
nulls, and publishes the post-viability lowering decision.

The root #820 then records the exact promoted capability and remaining open
limits. No general intelligence, general reasoning, teacher-equivalence, or
multiplication-free claim follows automatically.
