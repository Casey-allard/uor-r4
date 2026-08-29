# ADR-0005: Direct geometric attention, resonance replacement, and recurrent lowering

- **Status:** Accepted as the current #973 research direction; unqualified as a
  language mechanism
- **Date:** 2026-08-28
- **Owner:** #973 under programme root #820
- **Supersedes for forward work:** another fixed componentwise prototype or
  scale-only repair after the #997 negative
- **Preserves:** [ADR-0003](0003-fixed-zeta-prime-route-attention.md) route
  identities and [ADR-0004](0004-geometric-intelligence-route-hierarchy.md)
  scope/transport boundaries
- **Evaluation:**
  [Geometric Intelligence Evaluation](../geometric_intelligence_evaluation.md)
- **Evidence:** [Research ledger](../RESEARCH.md)

## Decision

UOR-R4 will qualify the literal geometric analogue of ordinary causal
attention before attempting to compress it. The reference mechanism is named
`DirectCausalGeometricAttentionR4V1`. It learns separate query, key, value, and
output roles; projects them into H4/S3 tangent frames; transports every causal
key and value into the current query frame; applies stable causal softmax; and
aggregates the transported values over the unchanged lawful support admitted by
#953. The bounded H4-only scaffold is now implemented. Its V2 result is
`NON_PROMOTABLE_BUDGET_MISMATCH`; fresh equal-manifold-budget V3 returned full H4 3/12
against matched plain 12/12 and a coherent alternative tangent connection at
10/12. The current action is therefore separately trained
`ConnectionGaugeCovarianceV4`, not resonance or scale. The full qualification contract later consumes
the existing paired-H4/E8 hierarchy trace. Paired-E8 hierarchy features, fiber,
and torsion are `NOT_IMPLEMENTED` in the scaffold and cannot receive credit
until the repaired connection is qualified and those inputs are explicitly
bound.

This dense O(T^2) operator is an offline scientific oracle, not the deployed
architecture. Its purpose is to separate four questions that the first bounded
recurrent experiment confounded: whether the representation can learn, whether
geometric transport is useful, whether the weighting law works, and whether a
bounded recurrent factorization preserves the function.

After the direct reference qualifies, #973 replaces only its softmax weighting
law with the planned multi-resonance design: a finite, artifact-bound
spherical/hyperspherical mode expansion. The replacement must retain the S3
fiber/torsion state; an S2/R3 Hopf direction alone is many-to-one and cannot
represent the full R4 spin state. Those band-limited mode sums are then factored
into `GeometricGatedDeltaRetentionR4V1` or a narrower bounded recurrent cell.
Only an approximation that preserves the reference's frozen construction-
validation effect proceeds
to H4/Q29/ternary/integer-table lowering.

The deployed goal remains a local CPU engine with no Transformer, softmax
all-pairs attention, mixture of experts, learned sparse expert router, Ollama,
hosted provider, or source weights. Compiler-side fitting may use floating
point, multiplication, allocation, and parallel reduction. None of that work is
credited to the deployed kernel. Exact/table/ternary lowering is authorized
only after the direct reference and its resonance/recurrent replacement
qualify.

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
either ambiguity. The literal operator and plain learning control now work;
the smallest decisive next experiment is `ConnectionGaugeCovarianceV4`, not a
new representation family or resonance approximation.

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

### Direct causal geometric attention oracle

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

The next mechanism version is `ConnectionGaugeCovarianceV4`. It stores one
explicit three-coefficient local vector for each Q/K/V/O role and compares
three separately trained, identically initialized arms:

```text
B_H(g) = [g*i, g*j, g*k]                 # H4-compatible local frame
B_A(g) = deterministic_tangent_basis(g)  # coherent alternative frame
B_P(g) = fixed_frame                     # ordinary plain comparator
P_c(s -> d) = B_c(d) * transpose(B_c(s))
x_s = B_c(s) * theta
```

For the H4 frame, this endpoint-basis connection must reproduce the existing
left action on tangent vectors. Before labels, V4 exhaustively checks all 120
H4 frames, central-finite-difference gradients, and gauge-covariant logits,
weights, scores, and update deltas. It then freezes a fresh balanced 24-case
population disjoint by prefix input from construction, V2, and V3. The main
arms must fit 16/16 construction cases, agree numerically and in decisions, and
reach at least 18/24 validation; current-only is capped at 12/24 and order,
value, and gauge-mismatch controls must each trail by at least six decisions.

This rung tests representational covariance, not geometric advantage. Paired-
E8/fiber binding begins only after it passes.

### Multi-resonance replacement

After the softmax oracle qualifies, freeze its data, roles, transport, support,
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

## What qualifies direct geometric attention

The dense reference earns a geometry-specific claim only if all of the
following are causally necessary under identical support, equal stored-
parameter and raw-manifold budgets for the matched main arms, and an explicit
operation ledger. Current-only is deliberately lower functional rank because
it removes history:

- both transported Q/K/V/O and equal-raw-manifold-budget plain Euclidean
  attention lower construction-validation next-token loss and improve or
  preserve top-1 versus frozen #953;
- transported geometry is non-inferior to plain attention under a predeclared
  paired margin, establishing that the geometric representation can carry the
  same attention function;
- a separately trained coherent tangent-frame connection determines whether
  the failure is specific to the H4 left action;
- tangent/norm-preserving key isometry, order-shuffled, and value-permuted
  controls lose the effect;
- current-token-only loses the effect;
- disabling the paired-H4/E8 input loses any claimed E8/H4 contribution;
- a strict causal audit reports zero future-position reads; and
- two compiles and inference replays reproduce exactly.

The H4-only scaffold's `GeometricSeedDisabled` arm tests only the learned H4
seed initialization while retaining tangent projection and H4 connection. It
is not a geometry-disabled or paired-E8-disabled control. The latter becomes
mandatory only when the actual paired hierarchy trace is bound.

Strict improvement over plain attention is a separate geometry-specific
predictive-advantage result, not required for functional parity. If plain
attention works but geometric attention falls outside the non-inferiority
margin, the learning/data path is viable but the geometric input/connection
seam is not qualified. If neither attention arm beats #953, revise the
representation or causal objective before tuning recurrence, resonance bands,
route families, or corpus scale.

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

## Resonance, recurrence, and held-out promotion

Only a qualified dense geometric reference authorizes the resonance
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

## Outcome branches

| Result | Required next action |
|---|---|
| Direct geometric attention beats #953, is non-inferior to plain attention, and beats destructive controls | Freeze it as the geometric oracle and replace only softmax with the fiber-preserving multi-resonance sieve. |
| Direct geometric attention strictly beats plain attention | Record the stronger geometry-specific predictive advantage; follow the same resonance-replacement path. |
| Plain attention beats #953 but geometry falls outside the parity margin | Retain plain attention as a learning-path control; redesign only E8/H4 input, tangent projection, or frame transport. |
| Neither direct attention arm beats #953 | Do not tune recurrence or scale. Revisit representation, training objective, or support binding. |
| Multi-resonance preserves the direct reference construction-validation effect | Freeze the band/mode/fiber contract and factor its accumulated modes into bounded recurrence. |
| Multi-resonance loses the effect | Revise the weighting/kernel approximation without changing the qualified Q/K/V/O reference. |
| Recurrent factorization preserves the resonance/reference effect and passes D3 | Freeze exact/table lowering and requalify the bounded #973 scopes. |
| Recurrent factorization loses the effect | Revise retention/update capacity against the frozen oracle; do not call the negative a failure of geometric attention. |
| A required frame, population, or causal audit is unavailable | Stop `UNAVAILABLE`; do not infer a metric result or open D3 labels. |

## Research basis and limits

This design combines ideas whose published results do not themselves prove a
UOR implementation:

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
