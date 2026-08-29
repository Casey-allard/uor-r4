# #973 direct causal geometric-attention reference smoke

- **Date:** 2026-08-28
- **Issue:** #973
- **Programme root:** #820
- **Decision:** [ADR-0005](adr/0005-predictive-geometric-connection-memory.md)
- **Deliverable:** `DirectCausalGeometricAttentionR4V1`
- **Mechanism scope:** one-head dense H4/S3 attention-kernel scaffold
- **Evidence status:** `EXERCISED_FROZEN_SYNTHETIC_V3_EQUAL_DOF_NEGATIVE`
- **Structural result:** `PASS_R4_DIRECT_CAUSAL_ATTENTION_SMOKE`
- **Current functional verdict:** `FAIL_EQUAL_DOF_H4_DIRECT_ATTENTION_NOT_LOAD_BEARING_ON_FRESH_V3`
- **V3 result:** `FULL_H4_3_OF_12_PLAIN_FIXED_TANGENT_12_OF_12_CURRENT_ONLY_6_OF_12`
- **Historical V2:** `NON_PROMOTABLE_BUDGET_MISMATCH`
- **Connection-specific advantage:** `NOT_ESTABLISHED_ALTERNATIVE_CONNECTION_10_OF_12`
- **Paired-H4/E8 hierarchy binding:** `NOT_RUN_INPUT_NOT_IMPLEMENTED`
- **Natural corpus qualification:** `NOT_RUN`
- **Multi-resonance replacement:** `NOT_RUN`

## Current result — corrected equal-manifold-budget V3 is negative

Review found that V2 did not actually match raw trainable-manifold degrees of
freedom. Its geometric Q/K/V/O raw vectors were normalized R4 vectors with
three degrees of freedom, while its plain and current-only raw vectors were
forced into a unit 3-vector and therefore had only two. The 8/8 V2 result is
preserved below as an append-only historical reveal, but it is budget-mismatched
and cannot support promotion.

V3 corrects the operator without retuning the already-revealed population. All
four trained arms now store one unit-normalized raw R4 vector for every Q, K,
V, and O token placement: 156 raw trainable-manifold degrees of freedom and 208
stored f64 scalars per arm over this 13-token namespace. The full and
seed-disabled arms project each raw vector into its route-dependent tangent
frame. Plain and current-only project their raw vectors and update gradients in
one fixed R4 tangent frame with base `(1,0,0,0)`; they remain fixed-3D and
transport-free without reducing their raw parameter sphere from S3 to S2.
This is equality of stored parameters and raw trainable-manifold dimension,
not equality of functional rank: the deliberately weaker current-token-only
control discards prefix history and collapses its Q/K/V use by design.

A fresh 12-document V3 population was frozen after that correction and before
any V3 prediction. Its prefix inputs are disjoint from both construction and
V2 validation independently of labels. Every scored prefix contains exactly
one earlier `1 -> target` binding, ends with query token `1`, and balances
targets `5` and `6`. The predeclared gate required full H4 at least 9/12,
current-only at most 6/12, and a full-arm advantage of at least three decisions
over current-only and every binding-destroying control.

The test mechanically checks those identities and thresholds before its sole
V3 scoring call, and the task execution log recorded them before the reveal.
Because this is the first commit containing V3, repository commit history alone
does not independently time-attest that chronology. V4 must publish its frozen
contract to #973 before any validation scoring.

The single first reveal was negative:

| Corrected V3 arm/control | Frozen validation |
| --- | ---: |
| `FullGeometric` | 3/12 |
| `PlainEuclidean` | 12/12 |
| `GeometricSeedDisabled` | 7/12 |
| `CurrentTokenOnly` | 6/12 |
| `AlternativeConnection` | 10/12 |
| `KeyTangentIsometryPermuted` | 7/12 |
| `OrderShuffled` | 5/12 |
| `ValuePermuted` | 8/12 |

The full H4 arm missed its absolute floor, trailed current-only, and was beaten
by every required geometry-destroying control. Therefore V3 does **not**
establish load-bearing geometric attention. The fixed-frame plain arm's 12/12
does establish that the literal dense causal attention operator and frozen
binding task remain learnable under the corrected equal-manifold budget; the failure
is specific to the current H4 projection/connection/optimization combination,
not to attention as a whole. No configuration, seed, fixture, threshold, or
arm definition was changed after the reveal.

A post-reveal construction replay, recorded without changing the mechanism,
also found that the corrected full H4 arm fit only 13/16 construction recalls;
plain and seed-disabled fit 16/16 and current-only remained 8/16. This makes the
negative interpretation stronger: V3 is not merely an unexplained held-out
generalization miss. The current H4-seeded projection/optimizer combination did
not completely fit its own construction population under the frozen budget.

### Corrected V3 frozen identities

| Field | Frozen value |
| --- | --- |
| Artifact CID | `blake3:136f0bac7361ca77a30946d8f120843b2d42eacccedbb1f64ab604af3cdf50f3` |
| Synthetic support-table CID | `blake3:8938b0a3c1cadd7fb051bef937566bf6533a8a04e26f315dd1f30ff96e14535c` |
| Synthetic support-overlay CID | `blake3:7f8666e9ac177b1085f483c59ee18c559430e70ac46b731e131ee41b5d262fd4` |
| Construction-population kappa | `blake3:61dd6791af2fc115fc0fcecfa4778bd3e8213f77abc1ee635f8847dace10cd89` |
| Validation-input kappa | `blake3:c6c5d6d3ec1af4aaa419ce1857bfe5e389d4a3e7a963d6a87b16d2161809829d` |
| Validation-label kappa | `blake3:1ec85cd0956b1237df63595f95b525b4d0b9c86a25a47d0453145addbeb9d260` |
| Experiment kappa | `blake3:8ff5c7f584f82c8fc7cf39ebbd28274140f4de3faf05eff370bc22e7aa429785` |
| Construction documents | 16 |
| Fresh validation documents | 12 |
| Frozen fit | 80 epochs, learning rate `0.04`, temperature `0.30` |
| Frozen thresholds | full >= 9; current-only <= 6; required drop >= 3 |

The arm-policy identity bound by the experiment and artifact includes every
deterministic initialization domain, the normalized-R4 raw representation,
the route-dependent versus fixed-frame projections, transport choice, and the
fixed-frame projected-gradient rule.

## Historical V2 result — budget-mismatched, not promotable

The missing literal attention operator existed in V2, and its first frozen
validation appeared to require prior context. Every scored example presented the
same current query key, token `1`. Its correct value is balanced between tokens
`5` and `6` and changes by document according only to an earlier causal
key/value binding. The construction and validation document identities and
complete prefix/target cases are disjoint.

The full H4/S3 arm and the separately trained fixed-3D Euclidean arm both
selected 8/8 validation targets. A separately trained current-token-only arm
selected 4/8, exactly chance on the balanced contradictory labels. Reversing
the ordered prefix reduced the result to 4/8, permuting values reduced it to
0/8, and a norm- and tangency-preserving key isometry reduced it to 6/8.

Those observations are retained to make the audit trail complete. They no
longer establish load-bearing attention because the plain/current arms had one
fewer effective degree of freedom per vector, and corrected fresh V3 failed.

### Historical V2 frozen identities and first reveal

| Field | Frozen value |
| --- | --- |
| Artifact CID | `blake3:64d4187570e275864a9bf543ba9c27b9eb103426924971d8735bc582b8837c39` |
| Construction-population kappa | `blake3:550c303ac607ade900055d2e68690d62e173f284e4a02c5b8bd01868b879d784` |
| Validation-input kappa | `blake3:2b2448e51821b2c003ca5cdede0d667fd22def6880003da8f54c38c74a80c09c` |
| Validation-label kappa | `blake3:e5bc1d8e5e6e390e62c823e51a139ca13bf094e2858d51357ae94104eb3b838a` |
| Construction documents | 16 |
| Validation documents | 8, identity- and complete-case-disjoint from construction |
| Scored validation decisions | 8 dynamic-binding recall events |
| Query at every scored event | token `1` |
| Frozen fit | 80 epochs, learning rate `0.04`, temperature `0.30` |

The fixture, configuration, thresholds, validation inputs, labels, controls,
and their separate kappas were frozen before the first validation prediction.
The historical V2 result in this subsection is its append-only first reveal. Two compiles produced
byte-identical artifacts and repeated inference produced identical decisions.

## Operator and causal ledger

For each observed position `i <= t`, the bounded contextual input defines
`K_i` from the causal predecessor and `V_i` from the token at `i`. The current
repeated query token supplies `Q_t`. All are projected into their local S3
tangent spaces and transported into the current cumulative frame:

```text
logit(t,i) = <Q_t, H4FrameConnection(i -> t) K_i>
             / (sqrt(3) * temperature), i <= t
alpha(t)   = stable_softmax(logit(t,0..t))
R_t        = sum_i alpha(t,i) H4FrameConnection(i -> t) V_i
score(c)   = <H4FrameConnection(leaf(c) -> t) O(c), R_t>
```

There is no hidden recency bias. The public trace records every logit, weight,
transported value contribution, aggregate value, candidate score, projection,
transport, causal-token read, and parameter count. A full-buffer API reads
token values only through the supplied query index. Mutating suffix values
leaves the trace unchanged; `future_token_reads` is zero; and normalized
softmax weights sum to one.

The exact transport object is the relative H4 group element. Its compiler-side
representation is an f64 left-quaternion matrix with tested norm, tangency,
composition, and numerical orthogonality. It is an `H4FrameConnection`, not a
claim of Levi-Civita shortest-geodesic parallel transport.

## Historical V2 arms and controls — not an equal-manifold-budget comparison

| Arm | Validation | Interpretation |
| --- | ---: | --- |
| `FullGeometric` | 8/8 | H4-seeded Q/K/V/O with correct H4 frame connection |
| `PlainEuclidean` | 8/8 | Separately trained fixed-3D dense-attention comparator; raw S2 budget mismatch |
| `GeometricSeedDisabled` | 8/8 | Separately trained non-H4 seeds with H4 tangent/transport operations |
| `AlternativeConnection` | 8/8 | Coherent orthonormal tangent trivialization; H4 connection is not uniquely required here |
| `KeyTangentIsometryPermuted` | 6/8 | Equal-shape tangent/norm-preserving key intervention |
| `OrderShuffled` | 4/8 | Same current query and token multiset; causal binding order destroyed |
| `ValuePermuted` | 0/8 | Attention logits retained while values are rebound incorrectly |
| `CurrentTokenOnly` | 4/8 | Separately trained stored-scalar-matched arm with a raw S2 budget mismatch |

Full and geometric control traces preserve tangent residuals within `1e-9`.
The coherent connection controls execute the same projection and transport
counts as the full arm. Plain and current-token arms report their lower
transport work explicitly; operation counts are not falsely called equal.
Every V2 trained arm owned a separate Q/K/V/O placement table with the same
stored scalar count. Equal stored scalars did not mean equal effective degrees
of freedom: forcing plain/current raw vectors into the unit fixed subspace made
them S2 parameters. V3 replaces this with normalized raw R4 vectors for every
arm and projects the raw vectors into the fixed tangent frame only at operator
evaluation and gradient update time.

## Historical V2 pre-delivery review correction

An earlier draft fixture made the answer deterministic from the current token
and returned 8/8 for both dense arms. Independent review rejected that result
before delivery: it tested a current-token mapping, not attention. The draft
also used tangent-breaking controls and reused plain weights in its seed
ablation. None of those observations is promoted here.

The V2 fixture uses one repeated query key with document-dependent
answers, a separately trained current-token control, separately trained model
arms, a fixed-3D comparator, coherent tangent/norm controls, and outcome gates.
No validation was opened until those repairs and their kappas were frozen.

## Post-reveal equal-manifold-budget review correction

Subsequent review found the S3-versus-S2 parameter-budget mismatch described
above. V3 was therefore created instead of changing V2 after reveal. The V3
test binds the normalized-R4 arm and seed policy, corrected artifact,
construction population, validation inputs, validation labels, decision
thresholds, and complete first results. The source validates unit R4 raw
vectors for every arm; it no longer forces or validates a zero fourth
component. Fixed-frame evaluation traces verify Q/K/V/O tangency, and
plain/current gradient updates are projected into that same fixed tangent frame
before unit-R4 retraction.

## E8 and multi-resonance boundary

The repository already serializes the canonical paired `H4 + phi*H4` E8
coordinate and exposes per-scope hierarchy, S3/Hopf, fiber, torsion, phase, and
chart traces through `AttentionLevelTrace`. This scaffold does not consume
those fields. V3 now blocks promotion into that step: adding more geometry
before isolating why the current H4 connection arm scored 3/12 while fixed-frame
plain attention scored 12/12 would compound an unidentified mechanism defect.

The frozen artifact policy literal says
`paired-H4-E8-hierarchy-input=NOT_RUN`. That literal is retained to preserve
the already-revealed artifact and experiment identities. Its implementation
meaning is narrower and explicit: this scaffold exposes no paired-E8 input, so
the input was not run because it is `NOT_IMPLEMENTED`, not because an available
paired-E8 arm was silently skipped.

Softmax is only the offline oracle. After a corrected fresh H4 rung and then a
paired-E8 direct reference pass their document-disjoint gates, freeze Q/K/V/O,
transport, causal mask,
support, and outputs and replace only the normalized weighting kernel with the
multi-resonance attention sieve. The sieve must preserve pointwise positivity,
exact numerator/denominator normalization, the full S3/fiber state, and
predeclared kernel and decision error. The focused
[reuse audit](multi_resonance_attention_sieve_audit_973.md) confirms that the
sin/cos, harmonic, and fixed-point Spin ingredients exist, but the normalized
attention sieve does not yet.

Tan remains bounded chart-local machinery with a pole-switch contract. It is
undefined at the quarter-turn cases where signed sine is maximally informative
and is not a global harmonic basis. Compiler-side sin/cos may later lower to
Q29/H4 lookup tables; no transcendental function is authorized in the deployed
kernel.

## Next decision gate

V3 and its labels are quarantined from all further mechanism choices. The next
bounded rung is `ConnectionGaugeCovarianceV4`, not paired-E8 or resonance:

1. parameterize every Q/K/V/O placement by the same explicit local
   three-coefficient vector rather than a normalized ambient-R4 vector followed
   by projection;
2. compare the H4-compatible frame `B_H(g) = [g*i, g*j, g*k]`, the existing
   deterministic Gram-Schmidt tangent frame, and one fixed-frame plain arm;
   train all three separately from identical coefficient initialization;
3. prove over all 120 H4 frames that base mapping, orthogonality, tangency, and
   composition hold, compare analytical Q/K/V/O gradients with central finite
   differences, and require gauge-covariant logits, weights, scores, and update
   deltas before any validation label is opened;
4. retain current-only, order-shuffled, value-permuted, and deliberately gauge-
   mismatched controls. The V3 `AlternativeConnection` was an inference-time
   transport swap over the full arm's weights, not a separately trained arm;
5. freeze one balanced 24-case V4 population whose prefix inputs are disjoint
   from construction, V2, and V3. Keep support, 80 epochs, learning rate,
   temperature, and causal mask unchanged; and
6. require 16/16 construction fit for all three main arms, numerical/decision
   parity between them, at least 18/24 fresh validation, current-only at most
   12/24, each binding/gauge-destroying control at least six decisions lower,
   zero future reads, and byte-identical replay.

If all three main arms agree and pass, V3 isolated a parameter-gauge/
conditioning defect and #973 may bind real `AttentionLevelTrace` paired-H4/E8,
hierarchy, S3/Hopf, fiber/torsion, phase, and chart inputs with a matched
paired-E8-disabled arm. If the alternative passes but H4 still fails, audit
quaternion multiplication side/order. If both geometric gauges fail while
plain passes, repair the tangent representation/optimizer. Only after a
paired-E8 direct reference passes may Q/K/V/O, transport, mask, support, and
outputs freeze for replacement of softmax's normalized weighting law with
`MultiResonanceSieve`.

Strictly beating plain attention would establish the stronger geometry-specific
predictive advantage. Parity can establish that geometry carries the attention
function only if named geometric inputs and controls are load-bearing. V3 does
not meet either standard.

## Reproduction

```bash
cargo test -p uor-r4-core --lib direct_causal_geometric_attention \
  --offline --no-fail-fast -- --nocapture

cargo test -p uor-r4-core \
  --test direct_causal_geometric_attention_973 \
  --offline --no-fail-fast -- --nocapture
```

Observed focused results after the V3 correction: 6/6 unit tests and 7/7
integration tests passed. The integration suite passes by reproducing and
asserting the frozen negative verdict; a green harness is not a positive
research result.

## Claim boundary

This smoke does not establish an H4-connection advantage, paired-E8 attention,
hierarchy/fiber/torsion attention, #953 corpus value, natural-language transfer,
multi-head attention, a residual/norm/FFN block, resonance replacement, bounded
recurrence, autonomous generation, exact/table runtime legality, correctness,
reasoning, chat, CPU or energy advantage, formal proof, product readiness, or
release readiness. It establishes one deterministic, causal, compiler-side,
one-head H4/S3 direct-attention implementation, an equal-raw-manifold-budget plain attention
positive control, and a frozen negative result for the current H4
projection/connection/optimization combination.
