# ADR-0005: HELM-D-R4 reference attention, autonomous generation, and parked intrinsic replacement

- **Status:** Accepted; ordinary dot-product/stable-softmax causal attention in
  coherent R4/Spin frames is the current baseline. The localization attempt
  stopped at its two-document preflight and rejected tangent readout.
  Intrinsic/readout, resonance, softmax replacement,
  recurrence, and lowering are parked. Provider-free autonomous
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation now passes, and its
  dedicated opt-in, loopback-only native HTTP endpoint passes exact eight-token
  CLI parity. Dashboard wiring/static native-readiness and WASM-isolation
  checks pass; browser interaction/E2E is `NOT_RUN`. The proposed
  `R4SoftmaxTeacherTraceV1` and
  source-free trace compiler are the sole active successor
- **Date:** 2026-08-28; direction updated 2026-08-30
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
- **Proposed source-free trace/compiler result:** `NOT_IMPLEMENTED`, `NOT_RUN`

## Decision

UOR-R4 accepts ordinary dot-product/stable-softmax causal attention in coherent
R4/Spin frames as its current attention baseline. The first intrinsic R4
distance/centroid attempt did not qualify; source-faithful learned-manifold V2
failed retention and matched parity; and the 8/8-contract localization attempt
stopped at its two-document preflight and rejected tangent readout. Those results are preserved. Intrinsic score/readout,
resonance, softmax replacement, recurrence, and exact lowering are now parked.
Provider-free autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`)
generation now passes. It retains the HELM-D architectural citation and uses
UOR's existing pinned SmolLM2
`HuggingFaceLlamaOracle` for embeddings, RoPE, residual/RMSNorm, MLP, final
normalization, and the language-model head. Its dedicated opt-in, loopback-only
native HTTP endpoint now passes the frozen eight-token CLI-parity canary,
without changing the default engine. Dashboard wiring/static native-readiness
and WASM-isolation checks pass; browser interaction/E2E is `NOT_RUN`. The sole
active successor is the proposed, not-yet-implemented
`R4SoftmaxTeacherTraceV1` and trace compiler: construction-only layerwise
token/QKV/attention/value/logit traces from the exact reference, followed by the
first source-free student/attention-state artifact comparison on decoded tokens
and next-token loss. No tag, release, hosted promotion, or static-WASM claim is
authorized.
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
5. build the proposed `R4SoftmaxTeacherTraceV1`/trace compiler from
   construction-only layerwise reference traces, then compare the first
   source-free student/attention-state artifact against decoded tokens and
   next-token loss before any intrinsic or resonance replacement resumes.

### Qualified native endpoint, checked dashboard wiring, and active source-free trace rung

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

`R4SoftmaxTeacherTraceV1` and its trace compiler are proposed names, not an
implemented capability. Their construction side may read the exact reference's
layerwise token, Q/K/V, attention, value, and logit states. Their evaluation
side must compare a source-free student/attention-state artifact on decoded
tokens and next-token loss while auditing causal inputs. A negative repairs the
trace representation or compiler; it does not reactivate intrinsic score/readout
or replace softmax.

A positive trace/compiler rung establishes only a first source-free
student/attention-state baseline at its frozen decoded-token agreement,
next-token-loss, causal-input, and matched-control scope. It does not establish
table-native or multiply-free execution, a transformerless architecture,
geometry advantage, correctness, reasoning, efficiency, or release readiness.

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

## Current outcome branch

| Result | Required next action |
|---|---|
| Qualified provider-free `R4SoftmaxReferenceGeneratorV1` generation; exact dedicated loopback HTTP/CLI parity; passing dashboard wiring/static native-readiness and WASM-isolation checks; browser interaction/E2E `NOT_RUN`; accepted ordinary R4/Spin softmax baseline | Build the proposed construction-only `R4SoftmaxTeacherTraceV1` and trace compiler, then compare the first source-free student/attention-state artifact against decoded tokens and next-token loss. Keep intrinsic/readout, resonance, softmax replacement, recurrence, lowering, release, and static-WASM promotion parked. |

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

The active successor is the proposed, not-yet-implemented
`R4SoftmaxTeacherTraceV1` and trace compiler. It will record construction-only
layerwise token/QKV/attention/value/logit traces from the exact reference, then
compile and evaluate a first source-free student/attention-state artifact on
decoded-token agreement and next-token loss. Intrinsic/readout, resonance,
softmax replacement, recurrence, and exact lowering remain behind that
baseline.

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
