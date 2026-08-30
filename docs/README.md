# R⁴ documentation

This is the map for understanding the repository without having to reconstruct
its history first.

R⁴ is currently pursuing one programme: a local, source-free language agent
whose predictive state, memory, inference, and reasoning are performed by
geometric recurrence and routing rather than a transformer, MoE, sparse learned
router, or dense learned matrix engine in the serving path.

The goal is real. Its success remains unproven. The current implementation has
a storage/recall and route-query foundation, one bounded causal path mechanism,
one corpus-scale geometric increment, and several narrow higher-scope
mechanisms. It is not yet a working geometric language model. The first bounded
transported gated-delta core is structurally implemented but was weaker than
plain delta on its sealed synthetic smoke. The literal reference,
`DirectCausalGeometricAttentionR4V1`, now exists. V2 is non-promotable because
of a raw-manifold-parameter mismatch; fresh equal-manifold-budget V3 returned full H4 3/12,
plain 12/12, current-only 6/12, and an inference-time coherent alternative-
connection swap 10/12; that alternative was not separately trained.
`ConnectionGaugeCovarianceV4` Phase I passed construction-scale representation
covariance, but its protected 24-case reveal returned 13/24 for H4,
alternative-tangent, and plain arms, with insufficient destructive-control
drop. It did not establish held-out attention. The positive reference is now
`HELM-D-R4`: pin HELM-D as the full-decoder architectural reference, then keep
a frozen ordinary decoder's learned Q/K/V, stable softmax, value aggregation,
and output projection
unchanged while splitting heads into R4 blocks, binding exact cumulative
Spin/H4 local frames, transporting K/V into the query frame, and mapping the
aggregate back before `W_o`. Bounded numerical and behavioral parity now
passes, establishing ordinary softmax attention in R4/Spin frames. HELM-D is
MIT licensed and pinned at
`7501deca8f413848bfef804be64ce874b72a3cd7`. The qualified
`R4SoftmaxReferenceGeneratorV1` credits and adapts HELM's attention seam and
provenance; it does not port HELM's remaining geometric decoder stack. UOR's
existing pinned SmolLM2 `HuggingFaceLlamaOracle` supplies embeddings, RoPE,
residual/RMSNorm, MLP, final normalization, and the language-model head. No
HELM checkpoint or upstream generation code was executed in this gate. The released HELM
generation/cache path is incomplete. Its checkpoint and full
geometric decoder remain an optional external baseline behind a separate
tokenizer and license gate, and are not directly an R4-block runtime.
Intrinsic
Lorentz V1 attempt 02 then stopped
`UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT`: barycenter covariance
was `9.121400701417315e-08` against the frozen `1e-08` ceiling, diagnostic curved
NLL was worse than donor and flat, and D3 remained sealed. Source-faithful
learned-manifold V2 then completed one valid non-D3 construction-validation run
at
`FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING`.
Donor/gauge parity, replay, causal work, and all three destructive controls
passed, but learned Lorentz failed donor retention and matched Euclidean parity;
the controls establish sensitivity only. The 8/8-contract localization attempt
stopped at its two-document preflight and returned
`REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`; tangent readout increased
normalized audit MSE on both documents. Ordinary dot-product/stable-softmax
causal attention in coherent R4/Spin frames is now the accepted baseline.
Intrinsic score/readout, resonance, softmax replacement, recurrence, and exact
lowering are parked. The provider-free-at-execution, source-backed native CPU
`R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation gate passes using the
credited attention seam and UOR's pinned SmolLM2 `HuggingFaceLlamaOracle`
decoder path in the CLI/evidence path. It passed 4/5 frozen quality in both passes, 5/5 exact
replay after deleting timing, all 30 layers with exact causal/projection/R4
audits and zero future reads, and source-donor reproduction. Its terminal is
`PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`. Its
explicit opt-in, loopback-only dedicated native HTTP endpoint now passes the
frozen eight-token sunlight canary with the same token sequence, decoded text,
decision CID, persistent-state CID, all-30-layer exact audits, and zero future
reads as the CLI. Dashboard wiring, native-readiness gating, and static/WASM
isolation checks pass; browser interaction/E2E is `NOT_RUN`. The feature is
disabled by default and does not change the default engine. This reference remains transformer-compatible and
`f32`/multiply/alloc/source-weight backed—not source-free, table-native,
multiply-free, transformerless, or a browser-WASM decoder. The next primary
rung is construction-only layerwise token/Q/K/V/attention/value/logit trace
capture from this exact behavioral oracle, followed by compilation and
evaluation of the first source-free student/attention-state artifact. Do not
resume resonance substitutes or promote a product/release until that passes.
No tag, release, hosted promotion, or static-WASM claim is authorized. D3
remains `NOT_RUN`; #973 remains open and #954 remains blocked.

Pinned-source provenance, ordinary-donor reproduction, transported-R4 parity,
the frame-permutation control, and the causal audit now pass; see the
[`HELM-D-R4` record](helm_d_r4_softmax_decoder_973.md). Upstream checkpoint
parity remains `NOT_RUN`; the intrinsic V1 outcome is recorded in the
[intrinsic Lorentz R4 record](intrinsic_lorentz_r4_attention_973.md). Resonance
replacement, recurrence, and exact lowering are parked; #954 remains blocked.
The binding autonomous-generation evidence is the
[generation record](r4_softmax_reference_generation_973.md) and
[compact attempt-01 aggregate](r4_softmax_reference_generation_attempt_01_result_973.json).
The endpoint and dashboard-wiring result is recorded in
[the native bridge record](r4_softmax_reference_http_bridge_973.md).

## Start here

Choose the shortest path that matches what you need:

- **Understand the project:** read the
  [Geometric Intelligence Programme](geometric_intelligence_programme.md).
- **Understand the current geometric mechanism:** read
  [ADR-0005](adr/0005-predictive-geometric-connection-memory.md), then
  [ADR-0004](adr/0004-geometric-intelligence-route-hierarchy.md), and use the
  [glossary](transformerless/GLOSSARY.md) for unfamiliar terms.
- **Contribute to the active build:** start from live issue
  [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) and take only the
  first unblocked stage.
- **Audit a result or claim:** use the [research ledger](RESEARCH.md), then open
  the exact issue-numbered evidence record it names.
- **Run the existing interface:** return to the root
  [README](../README.md#try-the-project).

If an older roadmap disagrees with the R4 Intelligence Completion Plan or live
GitHub dependency graph, the completion plan and live dependency graph win.

The qualified native source-reference CLI requires the already-local pinned
SmolLM2 snapshot:

```bash
cargo run --release --offline --bin r4 -- r4-softmax-generate \
  --source .uor-models/sources/smollm2-135m-instruct \
  --prompt "Explain in three short sentences why plants need sunlight." \
  --max-tokens 32 --workers 4 \
  --json-output /tmp/r4-softmax-reference.json
```

It has no provider or network fallback and is not the source-free/table-native
runtime or a browser-WASM decoder.

Its native bridge is separately opt-in and loopback-only:

```bash
cargo run --release --offline --bin r4 -- \
  --host 127.0.0.1 --port 8000 serve \
  --enable-r4-softmax-reference \
  --r4-softmax-source .uor-models/sources/smollm2-135m-instruct \
  --r4-softmax-workers 8
```

The dashboard reveals the source-backed reference only after the native server
reports it ready. Static/WASM mode rejects it; the default engine and `/api/chat`
remain unchanged. Those wiring/readiness and isolation checks pass; browser
interaction/E2E remains `NOT_RUN`.

## Current authority

These are the small set of living documents that define the present work:

1. [R4 Intelligence Completion Plan](r4_intelligence_completion_plan.md) —
   authoritative post-v0.1 work order and readable mirror of programme root
   #820.
2. [Geometric Intelligence Programme](geometric_intelligence_programme.md) —
   goal, architecture, and claim boundaries.
3. [ADR-0005: HELM-D-R4 reference attention and autonomous generation](adr/0005-predictive-geometric-connection-memory.md)
   — the positive reference/parity operator, learned-manifold V2 negative,
   localization preflight rejection, accepted ordinary-attention baseline,
   qualified autonomous-generation gate, verified native HTTP endpoint plus
   dashboard wiring/isolation checks, source-free
   trace/compiler successor, and parked replacement research. The bounded
   [HELM-D-R4 full-decoder result](helm_d_r4_softmax_decoder_973.md) closes the
   first parity gate. The
   [intrinsic Lorentz V1 record](intrinsic_lorentz_r4_attention_973.md) preserves
   its unavailable boundary, while the
   [learned-manifold V2 record](helm_d_learned_manifold_r4_construction_973.md)
   and completed
   [score-by-readout localization](helm_d_score_centroid_localization_973.md)
   define the prior evidence. The
   [generation record](r4_softmax_reference_generation_973.md) and
   [compact aggregate](r4_softmax_reference_generation_attempt_01_result_973.json)
   bind the generation PASS; the
   [native bridge record](r4_softmax_reference_http_bridge_973.md) binds the
   subsequent endpoint CLI-parity canary; browser E2E is `NOT_RUN`.
   Construction-only oracle traces and the first
   source-free student/attention-state artifact are next.
   The bounded
   [multi-resonance reuse audit](multi_resonance_attention_sieve_audit_973.md)
   distinguishes the implemented sin/cos and Spin substrate from the still
   unimplemented normalized attention sieve.
4. [ADR-0003: fixed-zeta prime routes](adr/0003-fixed-zeta-prime-route-attention.md)
   — the retained storage/recall substrate.
5. [ADR-0004: recursive route hierarchy](adr/0004-geometric-intelligence-route-hierarchy.md)
   — attention scopes, geometric transport, and reconstruction requirements.
6. [Geometric Intelligence Evaluation](geometric_intelligence_evaluation.md) —
   the minimal decision-bearing evidence policy.
7. [Glossary](transformerless/GLOSSARY.md) and
   [formal vocabulary](formal_vocabulary.md) — shared language and disciplined
   claim types.

Living implementation documentation may explain a current component, but it
does not independently promote a capability. Storage is not attention;
attention is not inference; readable text is not correctness; correctness is
not reasoning.

## Programme at a glance

```text
reversible lexical geometry
  → pin and audit the HELM-D full-decoder architecture
  → preserve learned Q/K/V + ordinary causal softmax in R4/Spin frames
  → establish numerical and real-language behavioral parity
  → preserve intrinsic Lorentz V1 construction-unavailable evidence with D3 sealed
  → preserve learned-manifold V2 valid non-D3 construction-validation negative
  → preserve the two-document preflight rejection within the 8/8 localization contract
  → accept ordinary dot-product/stable-softmax attention in coherent R4/Spin frames
  → qualify provider-free-at-execution, source-backed R4SoftmaxReferenceGeneratorV1 (HELM-D-R4) native CLI generation [PASS]
  → verify the opt-in, loopback-only dedicated native HTTP endpoint; no default-engine change [PASS]
  → dashboard wiring/readiness + static/WASM isolation [PASS]; browser E2E [NOT_RUN]
  → capture construction-only layerwise oracle traces
  → compile and evaluate the first source-free student/attention-state artifact
  → park intrinsic/readout, resonance, softmax replacement, recurrence, and lowering
  → correctness and abstention
  → multi-step reasoning
  → chat / CLI / WASM product integration
  → measured optimization
  → release QA
```

The code scaffold implements only the H4/S3 portion of the direct reference,
and its current mixed-gauge H4 parameterization is negative on equal-manifold-budget V3.
`ConnectionGaugeCovarianceV4` Phase I is positive for representation
covariance, but its held-out attention result is terminal-negative at 13/24
for all main arms. Binding the actual paired-E8 hierarchy, fiber, and torsion
remains `NOT_IMPLEMENTED`; the diagram is the qualification sequence, not a
claim that every arrow exists today.

Only the first unblocked stage is active. Formalization, optimization, and
large test programmes are supporting tools, not substitutes for reaching the
next observable behavior.

## Research record and archive

The repository contains years of useful positive, negative, and incomplete
work. It is preserved because rigor and failed hypotheses matter, but it is
not required reading for a newcomer.

Use [RESEARCH.md](RESEARCH.md) as the archive index. It leads to:

- issue-numbered measurement records;
- the earlier TLA and R4G1 table/graph compiler and integer runtime;
- proof, conformance, certification, and performance work;
- the original prime-router and geometric-context evidence;
- superseded decoder and intelligence roadmaps; and
- teacher-derived comparison and reproduction procedures.

Those documents report what a particular artifact established at a particular
time. They do not silently become evidence for the current route-native engine.
Historical commands may still run, but they are research reproductions unless
the current programme explicitly adopts them.

## How to read claims

- **Implemented** means the code or artifact exists.
- **Observed** means a named bounded run produced the recorded result.
- **Qualified** means the declared falsifier and threshold were met.
- **Not run** and **unavailable** are never treated as success.
- **Goal** or **hypothesis** is not a present capability.

For exact definitions, use the [formal vocabulary](formal_vocabulary.md).

## Documentation maintenance

Keep the root README approachable. Put sequencing in the completion plan,
architecture here under the current programme, and exact measurements in the research ledger and their named
records. Preserve historical evidence; add a clear superseded or historical
banner when old present-tense language could confuse readers.

Prefer linking to one current authority over copying the same mechanism into
many pages. If a measurement changes a claim, update the living summary and
append the new evidence to the ledger.
