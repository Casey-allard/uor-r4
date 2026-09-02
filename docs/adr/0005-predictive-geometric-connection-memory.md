# ADR-0005: HELM-D-R4 reference attention, autonomous generation, and parked intrinsic replacement

- **Status:** Accepted; ordinary dot-product/stable-softmax causal attention in
  coherent R4/Spin frames is the current baseline. The localization attempt
  stopped at its two-document preflight and rejected tangent readout.
  Intrinsic geometric/readout alternatives, resonance-based softmax replacement,
  full-model recurrent lowering, and exact deployment are parked. Provider-free autonomous
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation now passes, and its
  dedicated opt-in, loopback-only native HTTP endpoint passes exact eight-token
  CLI parity. Dashboard wiring/static native-readiness and WASM-isolation
  checks pass; browser interaction/E2E for that #973
  `/r4-softmax-reference` bridge is `NOT_RUN`. The separate #1039
  `/r4-softmax-local` dashboard path was exercised by #1041: its mechanics
  passed, narrative continuation passed `2/3`, and both supplied-history
  bindings failed. Terminal `KEEP_RAW_CONTINUATION_ONLY` retains it as a raw
  single-turn reference and forbids a source-backed history/multi-turn adapter
  (see the [#1041 record](../r4_softmax_local_normal_use_1041.md)).
  `R4SoftmaxTeacherTraceV1` and the first source-free suffix compiler now pass
  their bounded distillation gate, while decoded continuation remains
  incoherent. `R4SoftmaxTraceStateStudentV1` completed negative at its frozen
  state/control gate. The subsequent construction-only trace-state
  observability ladder in
  [#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012) completed at
  `INSUFFICIENT_SUPPORT_COVERAGE` and cannot attribute a boundary. It will not
  be expanded or repeated.
  [#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014) then established
  load-bearing ordinary causal attention in a directly trained R4/Spin model:
  attention-off worsened sealed NLL by `2.6773925609275944` nats and both Rust
  policy arms matched Python. Its complete language-quality DoD is negative at
  enabled NLL `2.127407277216677` and prompt retention `3/5`. Close that exact
  campaign without rerun or tuning. [#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017)
  then completed the one independently frozen exposure continuation. At
  `149,995,520` cumulative tokens it passed enabled parity, all mechanical
  gates, retention `5/5`, and normalized replay `5/5`, but failed solely on
  fresh sealed NLL `1.5727521962806827` against the strict `<1.50` gate.
  [#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019) froze an
  optional increase: twelve layers, 13,130,784 parameters, seed 1019, 16,800 steps,
  and 275,251,200 tokens over the same mechanism and Rust path. Exact
  population, 400-step fixed-sequence overfit, and random-export
  all-twelve-layer Rust parity passed. The signed MPS gate stopped
  `UNAVAILABLE_HARDWARE_BUDGET` on time: its `20.66 h` safety projection
  exceeded the `8 h` ceiling, while memory passed at `21.03%`. That terminal
  applies only to the frozen offline PyTorch/MPS implementation. Full training,
  final parity, reveal, generation, and replay remain `NOT_RUN`. A single
  isolated exact-shape MPS fast-path test (10 warmup plus 40 measured steps)
  combined fused AdamW with deferred logging and measured `4.485223 s/step`,
  slower than the signed `3.491307 s/step`; `fused=True` was removed
  immediately. This is a bounded fast-path negative, not a model result. #1019
  tuning/full-run work stops and remains optional/paused; at #1019 close, the
  active product step was the working #1017 `r4 generate` path. UOR's deployed
  architecture/runtime remains CPU-native;
  Apple Accelerate/BLAS and MPS are local offline accelerators only; CUDA and
  external GPU execution are out of scope. The MPS stop is not a
  model-quality negative, leaves the full-scale capacity hypothesis untested,
  and does not revoke the established attention result. See the
  [#1019 observed preflight](../r4_softmax_parameter_capacity_preflight_1019_raw.json).
  More 7.15M exposure or LR tuning remains prohibited.
  #973's later compact retained-language path passed its frozen language,
  state-off, and matched-control gates and completed deterministic autonomous
  decoding. Its paired-H4 address-capacity successor failed, while its direct
  and layerwise-normalized zero-parameter readouts each ended `PARTIAL`; that
  ladder is closed. The separately frozen
  `R4LearnedCandidateLeafAssociativeReadoutV1` then completed
  `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`: its pooled control improved fresh
  language but remained below both prompt-capacity floors, and geometry
  attribution failed. Do not retry that readout or run generation from it. Its
  separately frozen write/binding-law successor,
  `R4PredictiveBlockDeltaBindingV1`, then completed
  `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`: geometric gain
  `0.03896945868086732` missed the absolute capacity floor, geometry versus
  independently fitted plain missed its margin and NLL gates, and full delta
  did not beat independently fitted additive. Fresh-language and integrity
  gates passed. The binding action is `STOP_WITHOUT_GENERATION`; this retires
  only that predictive block-delta law and does not revoke ordinary-softmax or
  qualified retained-attention evidence.
  #973 remains open and #954 remains blocked.
- **Date:** 2026-08-28; direction and result updated 2026-09-01
- **Owner:** #973 under programme root #820
- **Supersedes for forward work:** another fixed componentwise prototype or
  scale-only repair after the #997 negative
- **Preserves:** [ADR-0003](0003-fixed-zeta-prime-route-attention.md) route
  identities and [ADR-0004](0004-geometric-intelligence-route-hierarchy.md)
  scope/transport boundaries
- **Evaluation:**
  [Geometric Intelligence Evaluation](../geometric_intelligence_evaluation.md)
- **Evidence:** [Research ledger](../RESEARCH.md)
- **HELM-D source identity/license/hash audit:** `PASS_PINNED_SOURCE_PROVENANCE`
  ([manifest](../../third_party/helm-d-reference/UPSTREAM.toml),
  [audit boundary](../../third_party/helm-d-reference/README.md))
- **Upstream HELM-D checkpoint/executable parity:** `NOT_RUN`
- **Ordinary-donor deterministic reproduction:**
  `PASS_BOUNDED_HELD_OUT_FULL_DECODER_REPLAY`
- **Transported-R4 parity result:**
  `PASS_HELM_D_R4_GAUGE_SOFTMAX_FULL_DECODER_PARITY_ADVANCE_TO_INTRINSIC_R4`
  ([record](../helm_d_r4_softmax_decoder_973.md),
  [machine result](../helm_d_r4_softmax_decoder_result_973.json))
- **Intrinsic R4 attention result:** attempt 01 `UNAVAILABLE_PRE_REVEAL` from
  checkpoint JSON round-trip identity; attempt 02
  `UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` from the frozen
  covariance audit, with D3 still sealed
  ([record](../intrinsic_lorentz_r4_attention_973.md),
  [summary](../intrinsic_lorentz_r4_attention_attempt_02_summary_973.json))
- **Source-faithful learned-manifold construction result:** attempt 01
  `UNAVAILABLE_HELM_D_MANIFOLD_CONSTRUCTION_EVIDENCE` before validation;
  attempt 02
  `FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING`,
  a valid non-D3 construction-validation negative. Donor/gauge parity and all
  three destructive-control separations passed, but learned Lorentz failed
  donor retention and matched Euclidean parity; the controls establish
  sensitivity only
  ([record](../helm_d_learned_manifold_r4_construction_973.md),
  [machine result](../helm_d_learned_manifold_r4_construction_attempt_02_result_973.json),
  [localization preflight result](../helm_d_score_centroid_localization_973.md))
- **Score/readout localization result:**
  `REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`; tangent readout increased
  normalized audit MSE on both documents, with pooled ratio
  `1.0643688804269025`
  ([record](../helm_d_score_centroid_localization_973.md),
  [machine result](../helm_d_score_centroid_localization_attempt_01_result_973.json))
- **Multi-resonance replacement result:** `NOT_RUN`, parked
- **Recurrent factorization/lowering result:** `NOT_RUN`, parked
- **Autonomous reference-generation result:**
  `PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`
  ([record](../r4_softmax_reference_generation_973.md),
  [compact aggregate](../r4_softmax_reference_generation_attempt_01_result_973.json))
- **Native HTTP endpoint result:** exact frozen eight-token CLI token,
  decision-CID, and state-CID parity; all 30 layers audited exactly; zero future
  reads; dedicated, explicit opt-in and loopback-only; no default-engine change
  ([record](../r4_softmax_reference_http_bridge_973.md),
  [structured result](../r4_softmax_reference_http_bridge_result_973.json))
- **Dashboard integration result:** wiring/static native-readiness and
  WASM-isolation checks `PASS`; browser interaction/E2E `NOT_RUN`
- **Source-free trace/compiler result:**
  `PASS_SOURCE_FREE_TRACE_STUDENT_ADVANCE_GEOMETRIC_STATE_COMPILER`
  ([record](../r4_softmax_trace_student_973.md),
  [structured result](../r4_softmax_trace_student_973_raw.json)); decoded
  continuation repeats `, Scotland` and is not coherent generation
- **Source-free geometric state-student result:**
  `STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION`
  ([record](../r4_softmax_trace_state_student_1011.md),
  [structured result](../r4_softmax_trace_state_student_1011_raw.json)); exact
  matched-arm metrics and artifact identities are bound in the current outcome
  amendment below
- **Directly trained end-to-end attention result:** ordinary causal attention
  `PASS` at the learned-intervention scope; full language-quality DoD `FAIL`.
  Enabled sealed NLL `2.127407277216677`; attention-off NLL
  `4.804799838144271`; penalty `2.6773925609275944`; two-arm Rust parity
  `PASS`; exact seeded replay `5/5`; subject/scene retention `3/5`
  ([record](../r4_softmax_end_to_end_attention_1014.md),
  [structured aggregate](../r4_softmax_end_to_end_attention_1014_raw.json))

## Decision

UOR-R4 accepts ordinary dot-product/stable-softmax causal attention in coherent
R4/Spin frames as its current attention baseline. The first intrinsic R4
distance/centroid attempt did not qualify; source-faithful learned-manifold V2
failed retention and matched parity; and the 8/8-contract localization attempt
stopped at its two-document preflight and rejected tangent readout. Those results are preserved. Intrinsic score/readout alternatives,
resonance-based softmax replacement, full-model recurrent lowering, and exact deployment are now parked.
Provider-free autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`)
generation now passes. It retains the HELM-D architectural citation and uses
UOR's existing pinned SmolLM2
`HuggingFaceLlamaOracle` for embeddings, RoPE, residual/RMSNorm, MLP, final
normalization, and the language-model head. Its dedicated opt-in, loopback-only
native HTTP endpoint now passes the frozen eight-token CLI-parity canary,
without changing the default engine. Dashboard wiring/static native-readiness
and WASM-isolation checks pass; browser interaction/E2E is `NOT_RUN`.
`R4SoftmaxTeacherTraceV1` and its first source-free Q16 suffix compiler pass
at the frozen distillation/control/replay scope. Their autonomous continuation
loops on `, Scotland`, and the student does not consume the geometric trace
state. `R4SoftmaxTraceStateStudentV1` subsequently compiled that causal state
transition and readout, but completed negative: its tiny CE movement was below
the frozen control threshold, no top-1 decision changed, and its decoded output
retained the same short cycle. The subsequent construction-only,
leave-one-document-out observability ladder in
[#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012) completed at
`INSUFFICIENT_SUPPORT_COVERAGE`; its per-fold support gate forbids boundary
attribution. Support expansion and another localization ladder are not the next
step. #1014 then completed the direct end-to-end rung: attention-off worsened
sealed NLL by `2.6773925609275944` nats and the Rust enabled/off policies
matched Python, establishing load-bearing ordinary causal attention at that
learned R4/Spin scope. Its full quality DoD failed because enabled NLL was
`2.127407277216677 > 1.50` and subject/scene retention was `3/5 < 4/5`.
Close the exact campaign without rerun or tuning. The next action was one
separately frozen quality-capacity rung over the unchanged mechanism. That rung,
#1017, has now closed NLL-only negative at `1.5727521962806827`, with retention
`5/5` and every other gate passing. #1019 now freezes an optional 12-layer,
13,130,784-parameter capacity improvement. Population, 400-step overfit, and random-export
all-twelve-layer Rust parity passed; MPS is `UNAVAILABLE_HARDWARE_BUDGET` on
time (`20.66 h > 8 h`) with memory passing at `21.03%`. That terminal applies
only to the frozen offline implementation. Full training, final parity, reveal,
generation, and replay remain `NOT_RUN`. Its fused-AdamW/deferred-logging fast
path was slower, so #1019 is optional/paused and the active next step is the
#1017 `r4 generate` product path. CUDA and external GPU execution are out of
scope. No tag,
release, hosted promotion, or static-WASM claim is authorized.
Its architectural reference is the official MIT HELM-D source pinned at commit
[`7501deca8f413848bfef804be64ce874b72a3cd7`](https://github.com/Graph-and-Geometric-Learning/helm/tree/7501deca8f413848bfef804be64ce874b72a3cd7).
The active generator credits HELM-D as an architectural reference only; it does
not port HELM's decoder stack. No HELM checkpoint or code executed in the UOR
generator or bridge gates, and no upstream result is inherited. The released HELM
generation/cache path is incomplete. Its checkpoint and full geometric decoder
remain an optional external baseline behind a separate tokenizer and license
gate, and are not directly an R4-block runtime. This ADR does not authorize
vendoring upstream code or claim upstream checkpoint parity.

The current sequence is strict:

1. bind and audit the pinned HELM-D dense-decoder architecture and exact source
   semantics;
2. reproduce a frozen ordinary full-decoder donor, then preserve every learned
   Q/K/V, ordinary compatibility score, stable causal
   softmax, linear value aggregation, and output projection while splitting
   each head into R4 blocks, binding exact cumulative Spin/H4 local frames,
   transporting K/V into the current query frame, and mapping the aggregate
   back before unchanged `W_o`;
3. preserve the qualified numerical/behavioral parity result and the subsequent
   intrinsic/learned-manifold/localization negatives; and
4. retain the qualified provider-free autonomous
   `R4SoftmaxReferenceGeneratorV1` with the HELM-D architectural citation and
   UOR's pinned SmolLM2 `HuggingFaceLlamaOracle` decoder path, then preserve the
   exact dedicated opt-in, loopback-only native HTTP endpoint result without
   changing the default engine, while retaining the passing dashboard
   wiring/static native-readiness and WASM-isolation checks and the browser-E2E
   `NOT_RUN` boundary; and
5. retain the completed `R4SoftmaxTeacherTraceV1`/trace compiler and source-free
   suffix-student result, then preserve the completed negative
   `R4SoftmaxTraceStateStudentV1` comparison against suffix, equal-budget plain
   recurrence, and transport permutation; and
6. preserve [#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012) as a
   completed construction-only observability result at
   `INSUFFICIENT_SUPPORT_COVERAGE`, with no licensed boundary attribution and no
   support-expansion/localization retry; and
7. execute [#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014):
   directly train end-to-end causal-softmax attention in R4 coordinates on a
   fresh untouched split and require autonomous decoded generation in the same
   deliverable; **completed** with attention established at the intervention
   scope and the full language-quality DoD negative; and
8. freeze one exposure-only quality-capacity rung that reuses #1014's exact attention,
   population discipline, Rust generation, intervention, and replay path while
   changing only training exposure; **completed** as #1017, negative solely on
   sealed NLL; and
9. execute #1019's frozen 12-layer, 13,130,784-parameter increase over the same
   mechanism, with 16,800 steps and 275,251,200 tokens; **model subgates passed,
   MPS time budget unavailable, full run `NOT_RUN`**. Population, 400-step overfit,
   and random-export all-twelve-layer Rust preflight parity passed. MPS stopped
   `UNAVAILABLE_HARDWARE_BUDGET` at `20.66 h > 8 h`, while memory passed at
   `21.03%`. That terminal applies only to the frozen offline implementation.
   Full training, final parity, reveal, generation, and replay remain `NOT_RUN`.
   Its fused-AdamW/deferred-logging fast path was slower, so #1019 is optional/
   paused and the active next step is the #1017 `r4 generate` product path.
   CUDA and external GPU execution are out of scope.

### Qualified native endpoint and completed source-free trace rungs

The bounded CLI/evidence path now passes. `R4SoftmaxReferenceGeneratorV1` reuses the qualified
transported R4/Spin attention and UOR's existing pinned SmolLM2
`HuggingFaceLlamaOracle` supplies the local tokenizer, embeddings, RoPE,
residual/RMSNorm, MLP, final normalization, and language-model head. All
checkpoint and tokenizer bytes are local and content-bound. The released HELM
cache/generation code is not assumed complete and is not the active decoder
path; cache optimization remains deferred.

Before outcomes, the owning issue must freeze prompts, deterministic decoding,
token limit and stop rule, source/checkpoint/tokenizer identities, donor
comparison, causal-read audit, and the exact coherence criterion. The evidence
must include provider-call count zero, autonomous multi-token output with no
teacher-forced continuation, next-token/logit comparison at every generated
step, complete component provenance, exact replay, and a declared work ledger.
The frozen run passed 4/5 decoded quality in both passes, 5/5 exact replay
after deleting timing, all 30 layers with exact causal/projection/R4 audits and
zero future reads, and donor reproduction (P1 through EOS; P2-P5 all 32
tokens). The subsequent dedicated opt-in, loopback-only native HTTP endpoint
passed one frozen eight-token prompt with exact CLI token, decision-CID, and
state-CID parity, all 30 layers audited exactly, and zero future reads. It left
the default engine unchanged. Dashboard wiring/static native-readiness and
WASM-isolation checks pass; browser interaction/E2E is `NOT_RUN`.

`R4SoftmaxTeacherTraceV1` and its first trace compiler are completed bounded
capabilities. Their construction side read the exact reference's layerwise
token, Q/K/V, attention, value, and logit states; the source-free suffix student
passed its frozen loss/top-1/control/replay/source-call gate but retained the
short decoded cycle. `R4SoftmaxTraceStateStudentV1` then evaluated a causal
geometric state artifact against matched suffix, plain recurrent, and
transport-permuted arms. It completed negative without material loss or top-1
separation. A negative repairs or retires the tested state representation; it
does not reactivate intrinsic score/readout or replace softmax.

These completed trace/compiler rungs establish only deterministic bounded
source-free distillation and one valid negative state representation at their
frozen decoded-token, next-token-loss, causal-input, and matched-control scopes.
They do not establish table-native or multiply-free execution, a
transformerless architecture, geometry advantage, correctness, reasoning,
efficiency, or release readiness.

The source-faithful HELM-D reference retains its declared Lorentz implementation:
the code at the pin forms the invariant Lorentz-inner-product distance surrogate
`2c + 2c<q,k>_L`, divides by a learned scale, applies ordinary causal softmax,
and aggregates by a normalized Lorentz centroid. It does not compute an
`arcosh` geodesic-distance square, and this ADR does not claim that it does.
The later intrinsic R4 arm may declare a negative squared R4 distance and a
geometric weighted centroid as its own separately trained operator.

These dense O(T^2) paths are reference architecture, not the final deployed
architecture. The active provider-free generator uses the pinned SmolLM2
`HuggingFaceLlamaOracle` source weights and transformer-compatible
`f32`/multiply/alloc decoder components.
Provider-free does not mean source-free, table-native, multiply-free, or
transformerless. Numerical parity is not geometric advantage or a serving
claim. Intrinsic score/readout, multi-resonance replacement,
`GeometricGatedDeltaRetentionR4V1`, and H4/Q29/ternary/integer-table lowering
remain parked research, not the active dependency chain.

The deployed goal remains a local CPU engine with no Transformer, softmax
all-pairs attention, mixture of experts, learned sparse expert router, Ollama,
hosted provider, or source weights. Compiler-side fitting may use floating
point, multiplication, allocation, and parallel reduction. None of that work is
credited to the deployed kernel. Exact/table/ternary lowering remains the
destination, but it is not current work unless the maintainer reactivates the
parked lane after the source-free trace/student baseline is established.

## Why the direction changed

The project has established useful but sharply bounded pieces:

- #989's source-free lexical table scored 99,362/446,342 held-out known targets
  (22.261404%) against 5.413561% unigram;
- #953's one accepted geometric count-radius intervention scored
  103,604/446,342 (23.211797%), +4,242 correct and +0.950392 percentage points
  at equal support and declared work;
- #969 and #973 produced bounded order-sensitive, paragraph, conversation, and
  noncommuting-global causal route witnesses; and
- #997 showed that causal geometric activity is not enough: its
  componentwise-Frechet document placement scored 8.367592% on 35,028
  construction-fitted held-out targets, below frozen #953 at 12.221651% and
  below both order-shuffled (8.376156%) and operator-permuted (8.467512%)
  controls; and
- the first bounded `GeometricGatedDeltaRetentionR4V1` core passed eight unit
  and three integration structural checks but, on its sealed synthetic
  construction fixture, full geometric scored 16/28 next-token and 55/112
  association wins while plain delta scored 23/28 and 98/112; and
- independent review made direct-attention V2 non-promotable because its
  comparator had fewer effective placement degrees of freedom; corrected,
  pre-reveal-kappa-bound V3 returned full H4 3/12, plain 12/12, current-only
  6/12, and an inference-time coherent alternative-connection swap 10/12; the
  alternative was not separately trained.

The diagnosis is therefore specific. R4/spin routes remain valid identities,
state carriers, and transport. A fixed marginal center of identity-derived
coordinates is rejected as semantic placement. The bounded recurrent negative
does not isolate attention: it simultaneously changes representation, training,
soft weighting, and compression. More documents or more rows cannot repair
either ambiguity. The literal operator and plain learning control work. V4
showed that its local coefficients could be represented covariantly on
construction, but its protected one-time reveal returned 13/24 for all three
main arms and at most a two-case loss under its destructive controls. That is a
held-out functional negative, not an unavailable run. A hand-designed V5
transition-binding fixture is not the active repair. The smallest current
decision is the source-pinned `HELM-D-R4` full-decoder parity path on real causal
language.

## Reference and lowering representation

### Immutable address and learned predictive roles

Every lexical unit retains its immutable registered route, prime, spin/Hopf,
torsion, payload CID, and kappa identities. The predictive artifact adds four
separate versioned roles:

- `Q(x_t)`: a query for the current causal position;
- `K(x_i)`: a key for each observed causal prefix position;
- `V(x_i)`: the information carried by each observed prefix position; and
- `O(c)`: a candidate-relative output placement for one already-admitted
  candidate `c`.

These placements are compiler outputs with provenance identities. They
do not mutate immutable addresses or payloads. A digest, token rank, prime
index, modulo class, or hexadecimal spelling may seed or identify a row, but it
cannot be interpreted as learned meaning. The paired-H4/E8 coordinate and
hierarchy trace are inputs or initializers; the next-token objective must still
learn these four predictive roles.

### Historical one-head direct causal geometric-attention oracle

For a current route frame `G_t` and prior frame `G_i`, V1 uses the declared
orthogonal H4 frame connection

```text
P_(i -> t) = LeftQuaternion(G_t * inverse(G_i))
```

and names it `H4FrameConnection`. It must not be called a Levi-Civita or
shortest-geodesic transport until that stronger equality is proved. Query,
key, and value vectors are projected into their local S3 tangent spaces. The
reference then mirrors ordinary one-head causal attention:

```text
logit(t,i) = <Q_t, P_(i -> t) K_i> / sqrt(d),  i <= t
alpha(t)   = stable_softmax(logit(t,0..t))
R_t        = sum_i alpha(t,i) P_(i -> t) V_i
score(c)   = <P_(leaf(c) -> t) O(c), R_t> + bias(c)
```

The causal mask must report zero reads from `i > t`. The paired-H4/E8 path must
be bound explicitly. A single R4 projection is a bounded V1 experiment, not a
claim that all eight E8 coordinates or both independent phase directions fit
faithfully in one quaternion block.

This is one bounded attention-kernel reference, not a complete Transformer
block. It intentionally excludes multi-head structure, a residual stream,
normalization, a pointwise feed-forward/MLP sublayer, and layer stacking. If the
kernel qualifies, the corresponding geometry-native operations are separate:
multiple resonance/chart channels; transported tangent residual addition plus
retraction; metric/tangent RMS normalization; and pointwise tangent or geodesic
channel mixing. None is allowed to hide a failed attention kernel.

### V3 connection/gauge diagnosis and V4 repair

V3 does not prove that the exact H4 group action is wrong. Its norm, tangency,
composition, and orthogonality checks pass. It isolates the combined placement-
gauge and conditioning seam:

- the H4 initializer mixed left- and right-quaternion gauges across Q/K/V/O;
- normalized ambient-R4 parameters were then projected into a tangent plane,
  whose local Jacobian loses rank near exactly tangent raw seeds; and
- V3's 10/12 `AlternativeConnection` was an inference-time transport swap over
  the full arm's trained placements, not a separately trained connection arm.

The repaired mechanism version is `ConnectionGaugeCovarianceV4`. It stores one
explicit three-coefficient local vector for each Q/K/V/O role and compares
three separately trained, identically initialized arms:

```text
B_H(g) = [g*i, g*j, g*k]                 # H4-compatible local frame
B_A(g) = deterministic_tangent_basis(g)  # coherent alternative frame
B_P(g) = fixed_frame                     # ordinary plain comparator
P_c(s -> d) = B_c(d) * transpose(B_c(s))
C_c(s -> d) = d * transpose(s) + P_c(s -> d)
x_s = B_c(s) * theta
```

`P` is the rank-three tangent transport; `C` is its full orthogonal extension.
For the H4 frame, `C` reproduces the existing left action and `P` agrees on
tangent vectors. Phase I passed all 120 H4 frames, 14,400 ordered connections,
central-finite-difference gradients, live controls, and gauge-covariant logits,
weights, scores, and update deltas. Its evidence root is
`blake3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e`.
No V4 validation input or label was used in Phase I. Phase II bound a fresh
balanced 24-case population, disjoint by prefix input from construction, V2,
and V3, plus a salted label commitment in PR #1001. The protected Phase-III
one-time reveal reproduced every identity and then scored H4-compatible,
alternative-tangent, and fixed-frame plain at 13/24 each. Current-only was
12/24; order-shuffled, value-permuted, and source-gauge-mismatch were 13/24,
12/24, and 11/24. Terminal:
`FAIL_CONNECTION_GAUGE_COVARIANCE_V4_HELD_OUT_FUNCTIONAL_PARITY_STOP_BEFORE_PAIRED_H4_E8`.

This rung established construction-scale representational covariance, not
held-out attention or geometric advantage. V4 remains append-only negative
evidence and will not be retuned or rerun.

### Pinned HELM-D architectural reference

`HELM-D-R4` binds the source repository, commit, license, architecture/config,
and source-faithful attention/centroid semantics before any R4 substitution.
This audit is a hard gate. It does not require or inherit the upstream gated
checkpoint. A source-identity or semantic mismatch stops; it is not evidence
about R4 geometry.

The pin is a research reference. No upstream checkpoint, paper metric, learned
weight, tokenizer mapping, Transformer block, or mixture-of-curvature/expert
claim is inherited merely by naming or reading the source.

### Gauge-equivalent R4/Spin ordinary-softmax reference

For head block `b` at causal query position `i` and donor position `j <= i`, let
`F_i^b` and `F_j^b` be exact cumulative Spin/H4 orthogonal model-frame bases.
The compiler-side vector action is floating point. Model coordinates are
encoded locally by transpose, so the declared frame transport is

```text
P_(j -> i)^b = transpose(F_i^b) * F_j^b
qhat_i^b     = transpose(F_i^b) q_i^b
khat_j^b     = transpose(F_j^b) k_j^b
vhat_j^b     = transpose(F_j^b) v_j^b
```

The parity reference transports both K and V into the query frame:

```text
kbar_(j -> i)^b = P_(j -> i)^b khat_j^b
vbar_(j -> i)^b = P_(j -> i)^b vhat_j^b
logit(i,j)       = ordinary_compatibility(qhat_i, kbar_(j -> i))
alpha(i,*)       = stable_causal_softmax(logit(i,*))
rhat_i           = sum_(j<=i) alpha(i,j) vbar_(j -> i)
r_i              = F_i rhat_i
output_i          = W_o r_i
```

The frozen ordinary donor binds its source weights, configuration, tokenizer,
and causal-language population. Every learned Q/K/V and `W_o`, causal mask,
compatibility scale, softmax,
aggregation order, and decoder operation remains unchanged. Frames and
transport are a declared gauge representation whose expected positive is
numerical and behavioral parity, not predictive advantage. Transport overhead
is reported explicitly.

### PARKED: trained intrinsic R4 successor

Only after the gauge-equivalent reference qualifies may #973 vary the attention
geometry. Its separately frozen intrinsic arm may use

```text
logit_R4(i,j) = -d_R4(qhat_i, kbar_(j -> i))^2 / tau
rhat_i        = GeometricWeightedCentroid({vbar_(j -> i)}, alpha(i,*))
```

The exact R4 distance, chart, centroid algorithm, tolerance, failure behavior,
and training objective must be artifact-bound. This is an R4 objective, not a
claim about the pinned upstream HELM-D logit implementation.

V1 implemented this objective with sixteen product-H4 blocks, an `acosh^2`
score, normalized Lorentz centroid, and coefficient-only construction fit.
Attempt 02 reached construction validation but its covariance audit measured
`9.121400701417315e-8` against the frozen `1e-8` ceiling, so its terminal is
`UNAVAILABLE` and D3 remains `NOT_RUN`. Its diagnostic NLL was also
`1.2531338878746174` above donor and `0.20892731808765097` above flat R4,
making a tolerance-only rerun decisionless. The next freeze therefore copied
the upstream learned-manifold attention seam more faithfully. Its valid non-D3
construction-validation result was
`FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING`:
learned-Lorentz NLL `7.71061809923296` failed donor retention
(`3.667626465210025`) and matched learned-Euclidean parity
(`4.483153905078387`). Donor/gauge parity, replay, causal work, and all three
destructive-control separations passed, so the controls establish that the seam
was exercised but not that the Lorentz operator was useful. D3 remains
`NOT_RUN`. The 8/8-contract attempt subsequently stopped at its two-document
preflight and rejected tangent readout. Score-only radius work is retained as a parked future
contract, not the next action.

### PARKED: multi-resonance replacement

After the trained intrinsic R4 softmax oracle qualifies, freeze its data, roles, transport, support,
and outputs and vary only the weighting law. The target positive kernel is

```text
K_tau(q,k) = exp(<q, Transport(k)> / tau)
```

A finite fiber-aware spectral amplitude first approximates its positive square
root. Pointwise positivity and exact normalization are then structural rather
than hoped-for properties of a truncated harmonic sum:

```text
A_M(q,k)   ~= exp(<q, Transport(k)> / (2*tau))
K_hat(q,k)  = weight_floor + abs(A_M(q,k))^2
D_t(q)      = sum_(i<=t) K_hat(q,k_i)
N_t(q)      = sum_(i<=t) K_hat(q,k_i) * Transport(v_i)
read(q)     = N_t(q) / D_t(q)
```

The amplitude may use a Fejer-windowed S3/SU(2) expansion, or S2 harmonics
tensored with explicit fiber/torsion modes. Expanding its modulus-square gives
a finite compound feature map
`K_hat(q,k) = sum_mode phi_mode(q) * phi_mode(k)`. The recurrent normalized
form must retain both its value numerator and exact normalization denominator:

```text
N_t[mode] = retain(Transport(N_(t-1)[mode])) + phi_mode(k_t) * v_t
Z_t[mode] = retain(Z_(t-1)[mode])            + phi_mode(k_t)
D_t(q_t)  = sum_mode phi_mode(q_t) * Z_t[mode]
read(q_t) = sum_mode phi_mode(q_t) * N_t[mode] / D_t(q_t)
```

`phi_mode` may be an S3/SU(2) mode or an S2 spherical harmonic paired with the
retained fiber/torsion phase. The contract must predeclare positivity, the
pointwise weight floor or deterministic uniform fallback, denominator floor,
uniform kernel error, and decision-error tolerances. Adding epsilon only after
summing the denominator is not exact normalization. Sin and cos are natural
bounded basis machinery. Tan is permitted only inside a named bounded chart
with a pole-switch contract; it is not used as a global basis. The sieve output
must preserve the oracle's frozen construction-validation decision; resonance
activity alone is not attention.

`T_G S3` is three-dimensional but is not the same object as the Hopf-projected
S2 embedded in R3. Trigonometric operations in the tangent chart retain the S3
basepoint/fiber when that anchor is kappa-bound. Using only the Hopf direction
discards it.

Replacing only `exp` while retaining every query-to-prefix comparison would
still be quadratic. The efficiency claim begins only when the finite feature
map permits the numerator and denominator mode sums above to be accumulated
once and read recurrently. Compiler-side experiments may evaluate sin/cos and
other floating-point basis functions. The deployed kernel may not: qualified
mode values, connection actions, reciprocal normalization, and chart switches
must later lower to artifact-bound H4/Q29/integer lookup tables under the
runtime operation contract.

### Recurrent factorization after the reference

The recurrent state contains fixed-capacity banks for four causal horizons:

```text
M_t = (M_t^local, M_t^short, M_t^scope, M_t^long)
```

The intended readings are current/previous route, last two or short suffix,
open sentence/paragraph scope, and bounded conversation/global retention. They
are channels of one mechanism, not independent attention claims. A scope
boundary can reset, checkpoint, or change a gate according to a frozen policy;
it cannot scan the complete prefix or corpus.

Before a bank is read or updated at step `t`, its state is moved from the prior
frame to the current frame by an artifact-declared connection transport:

```text
M_bar_t^s = Transport(A_(t-1 -> t), M_(t-1)^s)
```

`A` binds exact frame, orientation, chart, quantization, and transition-law
identities. In the first host-side prototype it may be evaluated in a declared
floating representation. A later exact runtime lowering must reproduce its
frozen reference semantics and satisfy the repository kernel contract.

### Gated delta update

For each bank `s`, the construction reference has the following semantic form:

```text
r_t^s = Read(M_bar_t^s, K(x_t))
e_t   = V(y_t) - r_t^s
M_t^s = Retain(M_bar_t^s, lambda_t^s)
        + WriteDelta(K(x_t), eta_t^s * e_t)
```

Here `x_t` is observed prefix data and `y_t` is the observed next route in the
construction partition only. `lambda` controls forgetting and `eta` controls
targeted overwrite. This equation is an offline reference objective, not an
assertion that multiply or float is allowed in serving.

At validation, test, and inference time, `y_t` is unavailable. State is updated
only after the selected or externally observed route becomes part of the causal
prefix. Actual future routes, evaluation answers, teacher continuations, source
weights, and provider text are forbidden.

### Candidate-relative readout

The accepted #953 policy first freezes lawful support `A_t`. For each `c` in
that same support, the predictive mechanism reads all enabled banks using
the candidate output role `O(c)` and returns one deterministic score:

```text
score(c | M_t) = Readout(O(c), M_t^local, M_t^short,
                         M_t^scope, M_t^long)
```

Only a unique qualified winner can replace #953's choice. Missing state, a tie,
or a failed margin returns exactly to #953. Runtime work is O(1) per state-bank
update and O(|A_t|) per decision under fixed bank capacity. No all-prefix
attention matrix, corpus scan, unbounded prompt replay, or candidate injection
is permitted.

## What qualifies `HELM-D-R4`

The first bounded implementation is complete only if:

- the official HELM-D source commit, MIT license, architecture, and
  source-faithful semantics are bound;
- the frozen ordinary donor configuration, data/tokenizer identities, source
  weights, and outputs reproduce under predeclared numerical tolerances;
- ordinary-donor and R4-frame arms hold learned Q/K/V and `W_o`, parameter budget,
  training data/updates, causal support, decoding, and aggregation order fixed;
- every R4 block has a bound exact cumulative Spin/H4 frame and both K and V are
  transported into the query frame before comparison or aggregation;
- the gauge-equivalent R4-frame arm reaches the predeclared numerical parity
  tolerance and retains real held-out next-token loss, top-1, and exact decoded
  behavior against matched donor/plain controls;
- the causal audit reports zero future reads, transport work is reported
  separately, and replay reproduces; and
- the equal-work source-frame-permuted intervention breaks numerical parity,
  proving that the R4 transport seam was exercised rather than bypassed; and
- failure of ordinary-donor parity or language-behavior retention terminates before
  intrinsic distance, centroid, resonance, recurrence, or scale work.

Parity establishes only that the exact R4/Spin gauge representation can carry
the donor's ordinary attention function. It is not geometric predictive
advantage. V1's separately trained intrinsic R4 distance/centroid arm did not
produce admissible evidence before D3. Its source-faithful learned-manifold V2
successor then failed to retain the donor or match its Euclidean control on
valid non-D3 construction validation, even though destructive controls
established sensitivity. The 8/8-contract score/readout attempt stopped at its
two-document preflight and rejected tangent readout. No intrinsic repair is active. Strict
improvement remains the only geometry-specific advantage claim if that lane is
reactivated. Neither parity nor
intrinsic success establishes correctness, reasoning, coherence, chat,
efficiency, transformerless serving, or release readiness.

## Parked fixed-route diagnostic: `PredictiveConnectionRetentionGate0V1`

Gate P0 was designed as one bounded host/compiler-side diagnostic. It reuses
the frozen D3 documents, #989 table, #953 admission/support, payload inversion,
and work ledger. It does not modify `CorpusInducedDocumentSpinPlacementR4V1`;
#997 remains immutable negative evidence.

Construction documents are split deterministically by document identity into
fit and construction-validation partitions before training. For each admitted
candidate, Gate 0 exposes twelve integer relations: H4 shell rank, wrapped
fiber distance, and wrapped torsion distance against current, previous,
ordered-last-two, and complete-prefix taps. A deterministic candidate-specific
integer readout is trained against the actual co-admitted distractor. This
probe does not implement learned Q/K/V/O, attention weighting, value
aggregation, recurrent banks, or connection transport. Its ignored corpus run
remains `NOT_RUN`. It is parked because it cannot establish or falsify the full
mechanism and is no longer in the active dependency chain.

## PARKED: resonance, recurrence, and held-out promotion

The following contract is retained for possible future reactivation; it is not
current work. Only a qualified trained intrinsic R4 reference authorizes the resonance
replacement. Freeze its construction split, parameters, support, transport,
and evaluation before changing softmax. The multi-resonance arm must preserve
the reference's direction on loss/top-1 and remain weaker under mode,
fiber/torsion, order, and value permutations. It may not earn credit from a
sparser candidate set or different work ledger.

Only a qualified resonance replacement authorizes recurrent factorization.
Compare the bounded geometric recurrence with the frozen direct/resonance
operators, #953, matched plain recurrence, no-delta, last-only, state-disabled,
and transport/order controls. Report approximation loss as well as next-token
loss. Only a recurrent positive may attach the frozen D3 held-out next routes
once. The final arm must then:

- improve held-out next-route loss and top-1 over #953 and every matched
  control at the position level;
- retain the direction under exact document-blocked analysis;
- beat the matched non-geometric recurrence to earn geometry-specific credit;
- cause one predeclared bounded decoded-output divergence;
- preserve byte-identical support and declared work across arms;
- report zero forbidden target/future/source/provider reads; and
- reproduce artifact and report bytes exactly.

A positive establishes only one held-out direct-to-resonance-to-recurrence
geometric-attention path inside #973. It does not establish correctness,
reasoning, general coherence, chat, performance advantage, exact runtime
lowering, or product readiness.
#954 stays blocked until the complete #973 hierarchy terminal is earned.

## Historical state-student launch branch

This row records the decision that launched #1011 after the suffix-student
result. It is preserved as history and superseded by the completed #1011
outcome amendment below.

| Result | Required next action |
|---|---|
| Source-free Q16 suffix trace student passes its bounded distillation/control/replay gate but enters a short decoded cycle | Build `R4SoftmaxTraceStateStudentV1`: first add fail-closed trace/bundle reload and tamper coverage, then compile construction traces into a causal recurrent R4/Spin state and compare it with frozen suffix, equal-budget plain recurrent, and transport/state-permuted controls. Keep intrinsic/readout alternatives, resonance-based softmax replacement, full-model recurrent lowering, exact deployment, release, and static-WASM promotion parked. |

## Outcome amendment — 2026-08-30 (EDT)

The frozen generator gate reached
`PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`: 4/5
decoded-quality prompts passed in both passes; all 5/5 report pairs replayed
exactly after deleting only timing; all 30 layers had exact causal,
projection, and R4 audits with zero future reads; and the source donor matched
P1 through EOS and P2-P5 for all 32 retained tokens. HELM remains the credited
MIT architectural reference at commit
`7501deca8f413848bfef804be64ce874b72a3cd7`; no HELM checkpoint or generation
code executed. The executed stack is UOR's pinned SmolLM2
`HuggingFaceLlamaOracle`.

This is a source-weight-backed `f32`/matmul Transformer-compatible reference,
not evidence of geometry advantage, softmax removal, source-free/table-native
serving, correctness, reasoning, frontier capability, release readiness, or a
static-WASM decoder. #973 remains open and #954 remains blocked. The next
authorized action is the explicit opt-in native bridge above; no tag, release,
or static-web promotion is authorized. See the
[generation record](../r4_softmax_reference_generation_973.md) and
[compact aggregate](../r4_softmax_reference_generation_attempt_01_result_973.json).

## Native bridge outcome and trace-rung amendment — 2026-08-30 (EDT)

The next action recorded in the outcome amendment above has now completed at
its exact declared scope. The dedicated opt-in, loopback-only native HTTP
endpoint matched the CLI on all eight generated token IDs, decision CID, and
persistent state CID for the frozen prompt. All 30 layers retained exact
causal, projection, and R4 audits; future reads were zero. The default engine
was unchanged. Dashboard wiring/static native-readiness and WASM-isolation
checks passed; browser interaction/E2E was `NOT_RUN`. See the
[bridge record](../r4_softmax_reference_http_bridge_973.md) and
[structured result](../r4_softmax_reference_http_bridge_result_973.json).

This establishes only an operator-access path to the already qualified
source-weight-backed reference. It does not establish source-free,
transformerless, geometry-advantaged, general-generation, reasoning, release,
hosted, or WASM capability. HELM-D remains an MIT architectural reference only
at `7501deca8f413848bfef804be64ce874b72a3cd7`; no HELM checkpoint or code
executed and no upstream result is inherited.

At the bridge checkpoint, the then-active successor was the proposed,
not-yet-implemented `R4SoftmaxTeacherTraceV1` and trace compiler. It would record construction-only
layerwise token/QKV/attention/value/logit traces from the exact reference, then
compile and evaluate a first source-free student/attention-state artifact on
decoded-token agreement and next-token loss. Intrinsic/readout, resonance,
softmax replacement, recurrence, and exact lowering remain behind that
baseline.

This paragraph records the then-current bridge decision. The trace compiler
and subsequent state-student work completed later on 2026-08-30 and supersede
it for forward sequencing.

## R4 state-student outcome and observability amendment — 2026-08-30 (EDT)

At implementation revision
`25569057f2d770dd2ffb0f10b6d2af0a985a6bd4`, #1011 compiled a 40,692-byte
state artifact with CID
`blake3:b617fc38e7bef1cdea76991f6e5e7cc653118451d63bcbd595f8ffd7e247ae7b`.
The construction freeze, source-free seal, and structured result CIDs were,
respectively,
`blake3:67cf67bb46b94cf5644b8dde286e89adb7e49159b3749790dffb500d8047fedb`,
`blake3:64587526f7883ab046e884a28b6af7e9e89818c9ead2039f8c995de7fb483060`,
and `blake3:dc04a8a8b21750799db2d451c8237d1e62cf90ffa74561fb54272b1e9704c824`.

Every arm covered the same nine context positions and `422,875` Q16 teacher
mass:

| Arm | Covered CE (nats) | Teacher top-1 | Actual-next top-1 |
|---|---:|---:|---:|
| Frozen suffix | 2.660721032 | 3/9 | 2/9 |
| Plain recurrent | 2.660770919 | 3/9 | 2/9 |
| Geometric recurrent | 2.660705367 | 3/9 | 2/9 |
| Transport-permuted | 2.660729215 | 3/9 | 2/9 |

The geometric arm improved CE by only `0.000015665` nats over suffix and
`0.000023848` over the permuted control, below the frozen `0.10`-nat threshold.
No teacher or actual-next top-1 decision changed, and the permuted control lost
no top-1 decision. All nine geometric states and distributions differed from
their permuted counterparts, but the differences were not selection-bearing.
All four arms emitted the same token IDs
`[281, 216, 28, 9291, 28, 9291, 28, 9291, 28, 9291, 28, 9291, 28, 9291, 28, 9291]`,
which decode to ` in , Scotland, Scotland, Scotland, Scotland, Scotland,
Scotland, Scotland`. Exact replay passed; every forbidden runtime counter was
zero; the geometric ledger contained 57 observations and 56 transports; the
permuted ledger added exactly 56 permutations.

The binding terminal is
`STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION`. It
falsifies the current 4D signed-reduction/token-derived state cell at this gate,
not ordinary R4/Spin softmax attention. See the
[authoritative record](../r4_softmax_trace_state_student_1011.md) and
[structured result](../r4_softmax_trace_state_student_1011_raw.json).

The next action at #1011 close was
[#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012), the native
child/blocker of #973: one construction-only,
leave-one-document-out observability audit using the same teacher-relative
candidate loss at four boundaries:

1. full ordered final-layer Q/K/V trace blocks;
2. the fixed 576-to-4 signed reduction;
3. token-derived role maps and recurrent state features; and
4. the fitted residual readout/logit scale.

If full traces transfer but the reduction does not, replace only the reduction
with a structured per-head/multiscale geometric representation. If the
reduction transfers but the state features do not, repair context-conditioned
K/V induction. If the state transfers but the logits remain inert, repair
readout calibration. If even the full traces do not transfer, stop trace
distillation and train the recurrent cell end to end under a new independently
frozen holdout. Revealed document 13 cannot promote a repaired mechanism
again. Corpus scale, added state dimensions, exact lowering, resonance, WASM,
release, correctness, and reasoning claims remain parked.

#1012 subsequently completed at `INSUFFICIENT_SUPPORT_COVERAGE`. Aggregate
primary coverage was `0.6202622204224402`, but the minimum fold covered only
`0.3469116829611222`, below the frozen 50% floor, so none of the boundary
branches above is licensed. On the covered rows full Q/K/V CE was
`2.215410922655504` versus suffix `2.215064603216862`; the required improvement
direction appeared in `0/4` folds. The fixed label control separated by
`1.3807454322642605` nats in `4/4`, and exact replay plus zero
source/document-13 reads passed. The project will not expand support or run
another localization ladder. It advances to direct end-to-end causal-softmax
attention training in R4 coordinates on a fresh untouched split with autonomous
decoded generation under #1014. That campaign is now complete. Enabled
sealed-test NLL was `2.127407277216677`, and the attention-off NLL was
`4.804799838144271`, a `2.6773925609275944`-nat penalty versus the frozen
`0.10` minimum. The final enabled and attention-off Rust arms preserved Python
top-1 within the `0.005` logit tolerance, all six layers passed exact R4/causal
audits, and all five seeded reports replayed exactly. Prompt subject/scene
retention was `3/5`, below the frozen `4/5` gate. This establishes ordinary
causal attention as load-bearing at the declared learned R4/Spin scope while
closing the full #1014 quality DoD negative. Do not rerun or tune the campaign.
See the [#1012 record](../r4_softmax_trace_observability_1012.md) and the
[#1014 record](../r4_softmax_end_to_end_attention_1014.md), followed by the
[#1017 record](../r4_softmax_quality_capacity_continuation_1017.md).

### #1014 outcome amendment — attention established, quality unresolved

The attention claim and the quality verdict are deliberately separate:

| Criterion | Frozen threshold | Result | Verdict |
|---|---:|---:|---|
| Enabled sealed-test NLL | `<= 1.50` | `2.127407277216677` | **FAIL** |
| Attention-off penalty | `>= 0.10` | `2.6773925609275944` | **PASS** |
| Final Python/Rust parity | same top-1; max delta `<= 0.005` | enabled `0.00000762939453125`; off `0.00001239776611328125` | **PASS** |
| Coherent R4/Spin execution | all six layers; exact causal/R4 work; zero future reads | exact | **PASS** |
| Prompt subject/scene | `>= 4/5` | `3/5` | **FAIL** |
| Decode/replay | UTF-8; no period-1..4 loop; exact reload | `5/5` | **PASS** |

The full issue result is therefore negative, but the attention mechanism is no
longer ambiguous. #1017 then completed the separate exposure continuation with
sealed NLL `1.5727521962806827` as its sole failed gate and retention/parity/
audits/replay passing. #1019 is an optional/paused 12-layer,
13,130,784-parameter campaign over the same attention/runtime path,
with population, 400-step overfit, and random-export all-twelve-layer Rust
parity passed. MPS is `UNAVAILABLE_HARDWARE_BUDGET` on time
(`20.66 h > 8 h`) while memory passed at `21.03%`. That terminal applies only
to the frozen offline implementation. Full training, final parity, reveal,
generation, and replay remain `NOT_RUN`. Its fused-AdamW/deferred-logging fast
path was slower, so the active next step is the #1017 `r4 generate` product
path. CUDA and external GPU execution are out of scope. It must not reopen
folds, probes, transport permutations, alternative attention architectures,
intrinsic geometry, resonance, or exact lowering. The
[#1014 structured aggregate](../r4_softmax_end_to_end_attention_1014_raw.json)
binds the five exact outputs, rubric grades, CIDs, audits, and replay results.

## Retained-language and paired-H4 capacity amendment — 2026-09-01 (EDT)

#973 subsequently completed a smaller matched language-path experiment rather
than extending the #1017/#1019 Transformer-compatible model. The
252,160-parameter `R4RetainedLanguagePathV1` exact-H4 retained arm and equal-size
ordinary causal-softmax arm both generalized on the frozen nonsealed language
population. Retained state-off worsened NLL by `0.334987556` nats and lost
16,660 top-1 decisions; retained final NLL was `0.003532495` nats better than
ordinary and its top-1 was `0.073814` percentage points lower. Every frozen
language, state-off, matched-control, causal, replay, and forbidden-read gate
passed. A separately frozen five-prompt smoke then completed provider-free,
fresh-load deterministic autonomous retained decoding. This qualifies a compact
retained-attention language path, not H4-specific superiority, reasoning,
correctness, exact lowering, browser readiness, or release readiness. See the
[V1 record](../r4_retained_language_path_v1_973.md).

The next frozen rung changed only V1's token addressing. It gave the two layers
separate canonical exact-H4 radix coordinates while preserving parameters,
state, projections, gates, training data/order, seed, and optimizer dose. A
construction-only token census reduced repeated cumulative joint addresses by
`97.5477123%`; this is collision evidence, not a learned-capacity or H4-advantage
result. The canonical prompt population contained 256 pairs and 512 directions
under CID
`blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340`.
The earlier provisional `blake3:9e041283383713a2ce48037774adb1022f6137d63dedfa4c587bdbee9e9f47c1`
scan omitted canonical whitespace normalization and mostly overlapped training;
it was rejected before freeze, never scored, and carries no evidence claim.

The candidate slightly improved fresh-slice language fit: NLL/top-1 were
`3.8832293739`/`0.2978017102` versus frozen V1
`3.8901151940`/`0.2970635689`. It nevertheless failed the independent
prompt-capacity decision. Candidate prompt gain was `0.0062477543` nats per
target token versus V1 `0.0063672952`, a `-0.0001195409` delta, and candidate
wins were `282/512` against the required 308. Both state-off arms collapsed to
exactly zero prompt gain and paired-logit difference; replay, causal, and
forbidden-read audits passed. The terminal is
`PAIRED_H4_PROMPT_CAPACITY_FAIL`, result CID
`blake3:508a4ff352f1e533d669d9616f65b972b0f13e8efe35867b7b095281ad940274`.
See the [canonical result](../r4_paired_h4_prompt_capacity_973.md) and
[structured aggregate](../r4_paired_h4_prompt_capacity_result_973_raw.json).

This negative is local to the paired-address capacity seam. It does not revoke
the established ordinary attention intervention or V1's qualified retained
language path. Do not promote or retune the paired candidate and do not rerun
generation. Preserve V1. The next independently frozen #973 experiment must
isolate the prompt-state-to-logit readout seam. #973 remains open, #954 remains
blocked, and C1-SB6 remains unauthorized.

## Historical intrinsic/replacement outcome branches — parked

The table below preserves the pre-localization decision tree. It is not the
active queue and requires an explicit maintainer reactivation.

| Result | Required next action |
|---|---|
| Pinned HELM-D source identity/semantics do not reproduce | Stop at the architecture audit; do not infer an R4 result. |
| Frozen ordinary donor does not reproduce | Stop at donor/reference parity; do not infer an R4 result. |
| Gauge-equivalent R4/Spin arm misses numerical or real-language behavioral parity | Stop before intrinsic geometry; repair only frame/block/transport/map-back integration. |
| Gauge-equivalent R4/Spin arm reaches parity | Freeze it as the ordinary-softmax R4 reference; this is not geometric advantage. Train the separately bound intrinsic R4 distance/centroid arm. |
| Intrinsic R4 attention retains behavior but does not beat matched controls | Freeze functional parity without an advantage claim; the predeclared gate decides whether resonance work has value. |
| Intrinsic R4 attention strictly improves over matched controls and survives destructive controls | Record the geometry-specific result and replace only softmax with the fiber-preserving multi-resonance sieve. |
| Intrinsic R4 evidence audit is invalid before D3 | Preserve `UNAVAILABLE`, keep D3 sealed, and repair the exact numerical/representation seam under a new construction-only freeze; do not infer a held-out metric result. |
| Intrinsic R4 attention loses the reference effect | Do not tune recurrence or scale. Revise only its distance, centroid, transport, or training seam. |
| Multi-resonance preserves the direct reference construction-validation effect | Freeze the band/mode/fiber contract and factor its accumulated modes into bounded recurrence. |
| Multi-resonance loses the effect | Revise the weighting/kernel approximation without changing the qualified Q/K/V/O reference. |
| Recurrent factorization preserves the resonance/reference effect and passes D3 | Freeze exact/table lowering and requalify the bounded #973 scopes. |
| Recurrent factorization loses the effect | Revise retention/update capacity against the frozen oracle; do not call the negative a failure of geometric attention. |
| A required frame, population, or causal audit is unavailable | Stop `UNAVAILABLE`; do not infer a metric result or open D3 labels. |

## Research basis and limits

This design combines ideas whose published results do not themselves prove a
UOR implementation:

- [HELM: Hyperbolic Large Language Models via Mixture-of-Curvature Experts](https://arxiv.org/abs/2505.24722)
  and the pinned MIT source motivate the dense causal geometric donor. Their
  paper/checkpoint results do not establish `HELM-D-R4`, UOR transport, or a
  transformerless serving path.
- [Gated Delta Networks](https://arxiv.org/abs/2412.06464) motivates combining
  adaptive forgetting with targeted delta updates.
- [Retentive Network](https://arxiv.org/abs/2307.08621) and
  [Mamba-2/structured state-space duality](https://arxiv.org/abs/2405.21060)
  show recurrent low-cost inference as a serious sequence-modeling design
  space.
- [Zoology](https://arxiv.org/abs/2312.04927) makes associative recall a
  necessary explicit stress test for efficient sequence models.
- [From Self-Attention to Connection Laplacian](https://arxiv.org/abs/2607.10677)
  supplies the useful operator view of attention as aggregation plus transport;
  the direct reference tests that operator before attempting its bounded
  factorization.
- [RiemannFormer](https://arxiv.org/abs/2506.07405) demonstrates a related use
  of tangent spaces, metric tensors, and parallel transport inside attention.
  Its reported results and transport choices are not UOR evidence.
- [Geometric Deep Learning](https://arxiv.org/abs/2104.13478) supplies the
  general gauge-equivariant rule that features from different local frames must
  be transported to a common frame before aggregation.
- [Transformers are RNNs](https://arxiv.org/abs/2006.16236) supplies the exact
  numerator/denominator recurrence for attention kernels with a factored
  feature map. It does not supply UOR's geometric modes or transport.
- [Rethinking Attention with Performers](https://arxiv.org/abs/2009.14794)
  shows that positive feature maps can approximate the softmax kernel with
  linear rather than quadratic sequence scaling. Its random features and
  Transformer architecture are reference evidence, not the UOR design.
- [Computational mechanics](https://arxiv.org/abs/cond-mat/9907176) motivates
  judging a state representation by retained predictive information rather
  than by its coordinate elegance.
- [Scalable MatMul-free Language Modeling](https://arxiv.org/abs/2406.02528)
  demonstrates that removing matrix multiplication is a credible systems goal,
  but its architecture and reported results are not UOR evidence.

No cited work establishes a geometry-native, transformer-free, causal local
language model with the UOR runtime contract. That combination remains the
research gap this ADR turns into a falsifiable implementation sequence.

## Direct and layerwise-normalized retained-readout amendment — 2026-09-01

The independently frozen `R4DirectRetainedReadoutLanguagePathV1` tested the
readout seam identified after the paired-H4 negative. It left qualified V1's
exact-H4 recurrence, learned parameters, state, training data/order, seed,
optimizer, 2,730-step dose, and one tied vocabulary matmul unchanged. Only the
head input changed from `N(h)` to `N(h) + g*N(a1+a2)`, with fixed `g=1` for the
candidate and `g=0` for the matched V1 control.

The intervention is causally useful: prompt gain rose from `0.0076304198` to
`0.0215897894` nats/token, directional wins rose from `313/512` to `343/512`,
fresh held-out NLL improved by `0.1636410364`, fresh top-1 improved by
`1.909487` percentage points, and state removal cost `1.1234286047` nats plus
20,179 correct decisions. All causal, forbidden-read, artifact/reveal-binding,
state-off, replay, and independent-verification checks passed. It is not a full
capacity result: the candidate missed the frozen absolute `0.0433216988` and
incremental `0.0253415693` prompt-gain floors. The terminal is
`DIRECT_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`, result CID
`blake3:71dd85e610dcc50b74cb2bb2068e5a1a433ac5df5db2a4f8fde22fb41735889c`.
No generation, retry, widened readout, exact lowering, or post-reveal gain
tuning is authorized.

At that checkpoint, one final parameter-free readout hypothesis remained and
was frozen as
`R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` with
`E @ [N(h) + (g/sqrt(L))*sum_l N(a_l)]`, `L=2`, fixed `g=1` versus `g=0`,
zero new learned parameters/state, the same recurrence/data/dose, and the same
one tied vocabulary matmul. Its V3 prompt population and fresh held-out slice
must be CID- and story-disjoint from all previously scored data; V2 is never
reused for scoring or tuning. Any unchanged prompt, language, state-off,
causal, or replay gate miss ends parameter-free readout work and redirects the
programme to a learned associative binding/readout architecture.

## Layerwise-normalized terminal and binding pivot — 2026-09-01

The final parameter-free hypothesis above completed exactly once under its
frozen contract. `R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` preserved
qualified V1's recurrence, exact-H4 addresses/transport, parameters, state,
training data/order, optimizer, `2,730`-step / `5,241,600`-presentation dose,
and tied vocabulary projection. The only intervention was
`N(a1+a2) -> (N(a1)+N(a2))/sqrt(2)` in the retained-state readout.

The intervention remained useful but below the predeclared capacity effect.
On the CID- and story-disjoint V3 population, candidate prompt gain was
`0.0286980210` versus matched V1 `0.0073316237`, delta `0.0213663973`, with
`339/512` wins and own-prompt NLL `3.4798765288` versus `3.6930405921`.
State-off prompt gain and paired-logit difference were exactly zero. The
candidate missed the absolute `0.0433216988` and incremental `0.0253415693`
gain floors.

The miss was not a language-fit or disconnected-state result. On 247,920
fresh decisions, NLL/top-1 were `3.7126411677` / `31.661826%` versus frozen V1
`3.8850003883` / `29.728138%`; state removal cost `1.3495375637` nats and
`20,595` correct decisions. Every language, causal, state-off, artifact,
forbidden-read, and replay gate passed. The canonical terminal is
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`, result CID
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`.
Independent fresh-process verification reproduced all 13 comparisons without
constructing an optimizer or reading training batches; verification CID
`blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.

The predeclared negative branch is binding: the parameter-free readout ladder
ends here. No `g` tuning, third normalization placement, retry, generation,
widening, resonance substitution, or exact/geometry-native lowering is
authorized by this result. The sole active #973 successor is a new,
independently frozen learned associative binding/readout over the preserved V1
retained-attention substrate, with its own matched non-geometric and state-off
controls. Candidate generation, reasoning, and lowering are `NOT_RUN`; #954
remains blocked.

## Learned candidate-leaf associative-readout freeze — 2026-09-01

The successor above is now frozen before implementation, V4 population
creation, optimization, or outcome access as
`R4LearnedCandidateLeafAssociativeReadoutV1`, under campaign
`R4LearnedAssociativeReadoutPromptCapacityV1`. Its status is
`FROZEN_ARCHITECTURE / POPULATIONS_NOT_CREATED / NOT_RUN`.

The qualified `R4RetainedLanguagePathV1` artifact remains immutable. The
geometric learned arm adds one zero-initialized `[2,4096,12,4]`
candidate-query table and reads each candidate's strict-prior transported value
at its canonical exact-H4 leaf. The equal-parameter learned control has its own
unshared, byte-identically initialized query table but reads the
occupied-address mean. A fixed-leaf cyclic derangement reuses the trained
geometric table while destroying candidate/address binding without selecting
unused slots. Head-off must reproduce qualified V1 byte-identically; full
state-off must zero both V1 retention and the added score.

Each effective learned arm adds exactly 393,216 trainable values over the
frozen 252,160-parameter V1 substrate, for 645,376 total parameters. V1's
23,040-f32-value recurrent state and 240 validity bits do not change. Both arms
receive the same predecessor 43,680-window / 5,241,600-decision order, seed
9738, and 2,730-step AdamW schedule. There is one trajectory, with no sweep,
alternate seed, table/rank change, continuation, scalar tuning, or scientific
retry.

The V4 prompt policy retains 256 pairs / 512 directions / 8,192 target tokens,
begins strictly after revealed V3 source ordinal 324,230, and excludes the
CID-bound 1,536-story V1+V2+V3 union. Its separately fixed fresh-language slice
contains 247,920 decisions at token range
`[156,032,138, 156,282,124)`. Both populations must be created once and sealed
together in a mode-`000` directory. Qualified V1 and both final learned-head
artifact CIDs must be fixed before the single reveal marker; optimization is
permanently closed after reveal.

Capacity and geometry attribution are distinct decisions. Either learned arm
must independently pass the unchanged absolute/incremental prompt-gain, win,
own-NLL, fresh-language, state-load, mechanics, causal, work, replay, and
forbidden-read criteria. `GEOMETRY_ATTRIBUTED` additionally requires the
geometric arm to clear the same frozen incremental effect and directional/NLL
rules against both the address-blind pooled arm and the fixed-leaf derangement.
A generic learned-head gain is not geometric advantage.

The outcome branches are fixed before data creation: geometric capacity plus
attribution permits one separately frozen disjoint smoke; learned capacity
without attribution permits only one smoke from the passing arm with the lowest
frozen fresh-language NLL; prompt capacity with language regression stops at a
joint objective; a double capacity miss rejects this exact readout law and
returns only to retained-value representation/binding; invalid mechanics carry
no model claim; and unavailable compute preserves only an identical
pre-reveal trajectory. See the
[canonical freeze](../r4_learned_associative_readout_prompt_capacity_973.md).

No V4 population, learned artifact, reveal, result, or verification exists at
this amendment. A future positive would establish only bounded learned
associative prompt capacity, and geometry attribution only if its separate
controls pass. Coherent generation, reasoning, correctness, intrinsic Spin/H4
superiority, exact/table/`no_std` lowering, browser or release readiness, #973
closure, and #954 unblocking do not follow.

## Learned associative-readout terminal and value-binding pivot — 2026-09-01

The frozen campaign above completed once and independently verified at
`LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`. This amendment records the outcome
without changing the historical pre-population freeze.

On the V4 prompt contrast, frozen V1, geometric exact-leaf, pooled
address-blind, and fixed-leaf-deranged gains were respectively `0.00642365`,
`0.00637679`, `0.01026323`, and `0.00666565` nats/token. The geometric arm had
`299/512` wins and own NLL `3.71038302`; its capacity decision failed. The
pooled arm had `324/512` wins and own NLL `3.68289051`; it was partial because
it missed both the absolute `0.04332170` and incremental `0.02534157` gain
floors. Geometry attribution also failed: geometric-minus-pooled gain was
`-0.00388645` with `209/512` paired improvements, and
geometric-minus-deranged gain was `-0.00028887` with `251/512`, against frozen
requirements of `+0.02534157` and at least `308/512` for each.

The negative is localized to associative prompt capacity in this readout law,
not to state reachability or ordinary language fit. Both learned arms passed all
fresh-language gates. Frozen V1, geometric, and pooled fresh NLL were
`3.90363602`, `3.90141233`, and `3.87375622`; top-1 was `29.6285%`, `29.6342%`,
and `30.0428%`. State-off NLL was `4.23919176`, costing the geometric arm
`0.33778` nats and 16,795 correct decisions and the pooled arm `0.36544` nats
and 17,808 decisions. The retained state is load-bearing, while the stronger
pooled result is explicitly a non-geometric control signal.

All ten mechanics gates and the fresh-process verifier passed. The final
artifact/reveal chain is geometric arm/head
`blake3:3983416d7936c3fc02bab19f711cfab69adaf3607077df7f3407515a8057eb60` /
`blake3:85a33965a7cd9ee952948ed6e6c5a925585edb9496377baa56a22ffaca40175f`,
pooled arm/head
`blake3:ca6f713a22a67b6a5749c9ebef374abeeb9d22a232d0ab4f77043ae09c69a08f` /
`blake3:4eeba8bb99d200e77558d89529a1e9f33d7c1ea6f4439ec3cae64c79d0b0f0d1`,
reveal
`blake3:0fcbeffa06ed2ef7496a5ead77ff9a81320c44a4e4aec2d29082f86c0b8634a9`,
result
`blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`,
and verification
`blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`.
The complete lifecycle ledger is in the
[canonical record](../r4_learned_associative_readout_prompt_capacity_973.md).

The predeclared branch now controls the programme: do not retry or tune this
candidate-query readout, do not add another readout over the same frozen V1
value field, and do not run generation. Preserve the pooled fresh-language
signal as the matched control. At that checkpoint, the next separately frozen
#973 architecture had to change the retained value write/binding law so prompt-
specific key-value information existed before readout, then compare it against
the pooled result and geometry-destroying controls. That successor is recorded
below. Reasoning, coherent generation, exact or geometry-native lowering,
transformerless general-model capability, and release readiness remain
`NOT_RUN` or `NOT_ESTABLISHED`.

## Predictive block-delta write/binding terminal — 2026-09-01

The independently frozen successor changed the retained write/binding law as
required. `R4PredictiveBlockDeltaBindingV1` used four transported, multiscale
R4 matrix-state banks and separate learned key, value, query, and candidate
maps over immutable qualified V1. Three equal-parameter arms were fitted
independently: canonical-H4 full delta, identity/plain full delta, and
canonical-H4 additive/no-overwrite. Each completed the same `2,730`-step dose.

On sealed V5, geometric prompt gain was `0.03896945868086732`, wins were
`375/512`, and own NLL was `3.5419674206289073`. The arm passed the incremental
V1/pooled, wins, NLL, state-load, fresh-language, and integrity gates, but
missed the absolute `0.04332169878499658` capacity floor. The terminal is
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`.

Geometry was not attributed. Geometric-minus-plain gain was
`0.023929811749894725`, below `0.025341569256760274`, and geometric own NLL was
worse than plain. The transport-permuted comparison separately passed, but the
contract required both controls. Delta overwrite was not attributed either:
geometric-minus-independently-fitted-additive gain was
`-0.006512463228773413`, with `234/512` paired improvements. Fresh geometric
NLL/top-1 was `3.84055165318221` / `30.979348%`, and all fresh/integrity gates
passed.

The original scoring attempt stopped before a scientific result because its
work audit compared a final two-row batch with full sixteen-row batches as if
their raw counters must match. Recovery CID
`blake3:7b76e36e44798bebf184ece08fdd8a2065bdd370106b5d64d5fae4c59dc6d88b`
bound unchanged fitted artifacts and authorized scoring only; it created no
optimizer and executed zero fit steps. Result CID
`blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`,
scoring CID
`blake3:44f8941d24a99fc230710fd700e7a7b13cee87587bfbe4e13bf7b095222e2ee6`,
and independent exact-replay verification CID
`blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`.

The binding action is `STOP_WITHOUT_GENERATION`. Retire this exact predictive
block-delta write/binding law; do not enlarge its corpus, tune its threshold,
generate from it, or lower it into the exact runtime. This result does not
revoke established ordinary causal-softmax attention or qualified retained
attention. Coherent generation from this cell, reasoning, integer/table
lowering, browser/release promotion, and #954/C1-SB6 progression remain
`NOT_RUN`, not established, or blocked. See the
[canonical terminal record](../r4_predictive_block_delta_binding_prompt_capacity_973.md).
