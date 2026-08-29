# HELM-D learned-manifold R4 construction contract (#973)

- **Status:** `NOT_RUN`
- **Mechanism revision:** `HelmDLearnedManifoldR4ConstructionV2`
- **Owner:** #973 under programme root #820
- **Base revision:** to be bound before execution from the protected merge that
  contains this contract
- **Positive reference:**
  [`HELM-D-R4` full-decoder softmax parity](helm_d_r4_softmax_decoder_973.md)
- **Negative predecessor:**
  [`IntrinsicLorentzR4AttentionV1` attempt 02](intrinsic_lorentz_r4_attention_973.md)
- **Upstream architectural source:**
  `Graph-and-Geometric-Learning/helm@7501deca8f413848bfef804be64ce874b72a3cd7`
- **Result:** `NOT_RUN`

This is an append-only experiment record. The pre-outcome contract below is
frozen before construction fitting. Exact population identities, executable
and parameter identities, execution evidence, and the terminal are added as
dated sections after they exist; this section is not rewritten in response to
an outcome.

## Decision and scope

The qualified `HELM-D-R4` reference established that ordinary causal softmax
attention can run through coherent R4/Spin frames without changing the donor's
function. `IntrinsicLorentzR4AttentionV1` then changed the score and centroid
but learned only block coefficients and output scales. Its attempt 02 stopped
before D3 because its construction covariance missed the frozen bound, and its
diagnostic construction-validation loss trailed both donor and flat R4.

This successor asks exactly one new question:

> On a fresh, entirely non-D3 construction population, can learned R4-block
> Q/K/V projections followed by HELM-D's Lorentz inner-product score, ordinary
> causal softmax, and full-head normalized Lorentz centroid retain donor
> behavior and outperform an independently fitted, exactly equal-capacity
> Euclidean score/arithmetic-centroid arm, while geometry-destroying controls
> lose?

The experiment is construction-only. A positive authorizes only a separately
frozen held-out contract. It does not reveal or reuse D3, authorize the
multi-resonance sieve, recurrence, exact lowering, corpus/model scaling, E8
expansion, close #973, or unblock #954.

## Source-copy boundary

The pinned HELM-D operator supplies these semantics:

- learned affine Q/K/V projection before RoPE;
- one hyperbolic `H^64` value per 64-spatial-lane head;
- the Lorentz-inner-product score divided by a learned scale, plus a learned
  scalar bias;
- ordinary masked causal softmax; and
- a normalized full-head Lorentz centroid.

This contract copies those score, softmax, and centroid semantics. It does not
claim upstream checkpoint parity or inherit any paper or checkpoint result.
The learned UOR projection is deliberately bounded: it retains the donor's
full dense Q/K/V maps and learns a block-diagonal `4 x 4` affine adapter over
each donor-projected R4 block before RoPE. That adapter is a compact UOR
parameterization within the upstream affine family, not a copy of HELM-D's
full dense Q/K/V checkpoint maps. HELM-D's learned cross-head output map is
also not copied in this rung; the donor's `W_o` is frozen unchanged.

The deployed transformerless runtime contract is not exercised. Dense
all-prefix attention, softmax, floating point, multiplication, allocation, and
compiler-side fitting are permitted scientific-oracle work and receive no
runtime-efficiency credit.

## Frozen operator

### Pre-RoPE learned projections

The frozen donor is SmolLM2-135M: 30 layers, 9 query heads, 3 KV heads, head
width 64, and sixteen ordered R4 blocks per head. After the donor's dense Q/K/V
projection and before donor RoPE, role `r` applies

```text
z_(l,r,h,b) = A_(l,r,h,b) x_(l,r,h,b) + a_(l,r,h,b)
```

where `A` is `4 x 4` and `a` is R4. Query adapters are indexed by the 9 query
heads. Key and value adapters are indexed by the 3 KV heads before the donor's
unchanged GQA expansion. Query and key then receive the donor's unchanged
RoPE; value does not. No adapter may read a future position, observed next
token, validation metric, candidate label, document ID, or D3 identity.

Each adapter contains 20 fitted scalars. The exact map budget per arm is:

```text
Q maps: 30 * 9 * 16 * 20 = 86,400
K maps: 30 * 3 * 16 * 20 = 28,800
V maps: 30 * 3 * 16 * 20 = 28,800
map total:                    144,000
scale + bias: 30 * 2 =            60
total per learned arm:       144,060
```

There is exactly one learned positive scale and one learned scalar bias per
layer. The bias is retained because it is present in the copied operator, but
it is uniform across each causal softmax row and therefore predictively null;
it receives no evidentiary credit.

### Coherent R4/Spin transport and one full-head Lorentz point

After RoPE, each query, key, and value head is stored as sixteen ordered R4
blocks. The existing cumulative Spin/H4 frame supplies one coherent `SO(4)`
action per block. The block-diagonal action is an `SO(64)` isometry. Keys and
values are transported from each source frame to the current query frame, and
the sixteen transported blocks are concatenated before the manifold lift.

For unit curvature, define

```text
Phi(x)      = (sqrt(1 + ||x||^2), x),              x in R^64
<X,Y>_L     = -X_0 Y_0 + sum_(a=1..64) X_a Y_a
score(i,j)  = (2 + 2 <Phi(q_i), Phi(k_(j->i))>_L) / tau_l + beta_l
alpha(i,*)  = stable_causal_softmax(score(i,0..i))
S_i         = sum_(j<=i) alpha(i,j) Phi(v_(j->i))
M_i         = S_i / sqrt(-<S_i,S_i>_L)
read_i      = spatial(M_i)
```

`tau_l` is finite and positive. It is initialized to `24.0 = sqrt(9 * 64)`;
after each optimizer step it is projected to at least `1e-6`. `beta_l` is
initialized to zero. The centroid denominator uses compensated f64 sums and
the stable identity

```text
sqrt(-<S,S>_L) = sqrt((S_0 - ||S_spatial||) * (S_0 + ||S_spatial||)).
```

Any non-finite lift, score, weight, aggregate, nonpositive scale, or
non-timelike centroid fails closed. There is no Euclidean fallback. The
centroid's spatial R4 blocks are mapped from the query frame to model
coordinates and then passed through the frozen donor `W_o`, residual/FFN
stack, final norm, and LM head.

### Equal-capacity Euclidean arm

The matched learned control has its own 144,060 parameters with the identical
Q/K/V adapter shapes, GQA indexing, initialization, construction rows,
optimizer, step count, precision, causal support, frame transport, frozen
`W_o`, and full decoder. It changes only the score and value aggregate:

```text
score(i,j) = -||q_i - k_(j->i)||^2 / tau_l + beta_l
read_i     = sum_(j<=i) alpha(i,j) v_(j->i)
```

Its learned parameters are independent of the Lorentz arm. Parameters are
never shared after their identical initialization.

## Frozen arms and controls

Every arm uses the same selected causal prefixes, donor checkpoint, tokenizer,
decoder, support, scored positions, and ordered work ledger.

1. **Donor:** unchanged dense Q/K/V, RoPE, causal softmax, linear aggregate,
   and `W_o`.
2. **Gauge-equivalent R4 reference:** unchanged donor attention expressed in
   coherent R4/Spin frames with exact source-to-query K/V transport.
3. **Learned Lorentz:** the complete operator frozen above.
4. **Learned Euclidean:** the exactly equal-capacity learned control frozen
   above.
5. **Source-frame-permuted:** learned Lorentz parameters with a frozen
   non-identity permutation applied to source-frame identities only.
6. **Value-permuted:** learned Lorentz weights with a frozen non-identity
   permutation of causal values inside each row.
7. **Order/key-shuffled:** learned Lorentz values remain in causal order while
   a frozen non-identity permutation reassigns past keys to source positions.

The three destructive controls perform the same number of adapter, transport,
score, softmax, centroid, and decoder operations as learned Lorentz. A
permutation must be non-identity on every scored row with at least two causal
sources, or the control is unavailable.

## Frozen construction population

The raw population remains the 3,000-document SimpleWiki corpus at
`blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`.
The donor remains
`blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`.
The tokenizer identity is measured from bytes and bound in the partition
append; it is not inferred from a path.

For document ID bytes `id`, define

```text
selection_digest(id) =
  BLAKE3("uor-r4.helm-d-learned-manifold-r4-construction/2\0" || UTF8(id))
```

Selection is frozen as follows:

1. Stream and verify the raw corpus identity and count. The partition freezer
   may encode enough source text to establish the target-blind `>=17` token
   eligibility predicate and therefore transiently observes that token 16
   exists. It must not persist, hash, compare, or branch on that token's
   identity; only the boolean length predicate may affect selection.
2. Exclude every D3 document under
   `BLAKE3(UTF8(id))[0] mod 5 == 0`.
3. Exclude parity document ID `12`.
4. Exclude by both document ID and encoded-input CID every construction-fit,
   construction-validation, and D3 identity committed by
   `docs/intrinsic_lorentz_r4_attention_partition_973.json`, whose partition
   CID is
   `blake3:cad3dfd17159fdacc5c40e38753109c11764117e3c960f42b9b198d5731272a1`.
   Opening the predecessor's committed identities is permitted; opening or
   reusing its target bytes is forbidden.
5. Encode exactly `title + "\n\n" + text`. A document is eligible only when
   it contains at least 17 encoded tokens.
6. Sort eligible documents by complete `selection_digest`, then UTF-8 ID.
   Assign the first 16 to construction fit and the next 8 to construction
   validation. No overlap or replacement is allowed.
7. Bind every selected ID, title CID, first-16-input-token CID, corpus byte
   range, donor CID, tokenizer CID, selection-policy CID, and exact exclusion
   set before fitting begins. No target value or target CID is persisted or
   serialized in this pre-fit partition commitment. The decision runner cannot
   materialize any construction-validation source text or tokens before the
   exclusive fit checkpoint has been written and read back successfully.

Each document supplies exactly input positions `0..15`. Only query positions
`8..15` are fitted or scored, with token `position + 1` used only as the later
next-token scoring target. Denominators are therefore 128 construction-fit and
64 construction-validation positions.

This is not D3. No D3 ID, input, target commitment, reveal marker, recovery
artifact, or result may be created by this experiment.

## Frozen compiler-side fitter

Both learned arms start from identity `4 x 4` adapter matrices, zero adapter
biases, layer scale `24.0`, and layer bias zero. Frozen donor traces for the 16
fit documents supply only causal attention rows and donor value aggregates;
observed next-token targets are not optimizer inputs.

For each scored row, the fit objective is the sum of:

1. mean cross-entropy from the donor causal attention distribution to the
   learned arm's causal attention distribution;
2. mean per-lane squared error between the arm aggregate and donor aggregate,
   after dividing both by `max(1, ||donor aggregate||_2)` for that row; and
3. ridge `1e-6` from adapter matrices to identity and adapter biases to zero.

The layer scale and uniform bias receive no ridge. The uniform bias is recorded
but is mathematically null under the row-softmax objective.

Each arm is fitted separately by exactly 128 full-batch Adam steps with
learning rate `1e-3`, `beta1 = 0.9`, `beta2 = 0.999`, and `epsilon = 1e-8`.
There is no early stop, convergence-selected step count, learning-rate search,
validation selection, arm-specific schedule, retry with new initialization, or
post-validation refit. Compiler arithmetic and optimizer moments use f64.
Work is partitioned into eight deterministic shards and reduced in shard,
layer, head, query-position, source-position, block, and lane order. Both arms
must use the same shard schedule.

Before validation bytes are materialized, an exclusive-create checkpoint must
bind and immediately read back:

- contract, population, partition, donor, tokenizer, and upstream-source CIDs;
- exact operator and optimizer specifications;
- implementation-source and executable CIDs;
- worker count and ordered-work ledger;
- both complete parameter byte streams and parameter CIDs;
- fit objectives, gradients/work audit, fit report, and independent fit replay
  CIDs; and
- proof that no validation or future-token read occurred.

If any identity differs, a checkpoint already exists, or fit replay is not
byte-identical, construction validation remains sealed and the run is
`UNAVAILABLE_HELM_D_MANIFOLD_CONSTRUCTION_EVIDENCE`.

## Hard preflight and run contract

The run does not open construction validation unless all of these focused
checks pass:

- pinned HELM-D score/softmax/centroid golden vectors;
- identity pre-RoPE adapters reproduce donor post-RoPE Q/K/V and demonstrate
  that the hook is exercised before RoPE;
- central finite-difference gradients for Q, K, V, layer scale, softmax, and
  centroid agree with the analytic fitter within the frozen unit tolerance;
- a synthetic full-head census spanning every registered H4 frame preserves
  the Lorentz hyperboloid residual, score, weights, and centroid covariance;
  maximum centroid covariance error is at most `1e-8`;
- the ordinary donor and coherent gauge reference preserve full-decoder output
  over the frozen construction population within the empirical bounds below;
- source-frame permutation is live;
- the first two construction-fit documents lower the complete objective for
  both learned arms under the frozen optimizer and exercise all eight shards;
  and
- the canary's measured extrapolation is at most two wall-clock hours for the
  complete fit plus one validation pass on this host.

The only named checks activated by this decision are:

```text
cargo test -p uor-r4-model-source --offline pre_rope_projection
cargo test -p uor-r4-core --offline helm_d_learned_manifold_r4_construction
cargo test -p uor-r4-core --release --offline \
  --test helm_d_learned_manifold_r4_construction_973 \
  freeze_helm_d_learned_manifold_r4_construction_partition \
  -- --exact --ignored --nocapture --test-threads=1
cargo test -p uor-r4-core --release --offline \
  --test helm_d_learned_manifold_r4_construction_973 \
  helm_d_learned_manifold_r4_construction_decision \
  -- --exact --ignored --nocapture --test-threads=1
```

The freezer and decision runner are separate single-test launches. They must
never share one unfiltered `--ignored` invocation.

No workspace-wide QA, D3 runner, generation run, resonance test, recurrent
test, lowering test, scale test, or #954 test is activated by this contract.

The decision-bearing run contract is:

```text
metric to move:       64-position non-D3 construction-validation next-token NLL
reachability ceiling: not inferable from V1 because projections and full-head
                      manifold aggregation are newly learned; the two-document
                      objective canary is the binding reachability instrument
instrument:           all structural checks plus both canary objectives lower
exit rule:            the empirical criteria and terminals below
if positive:          freeze a new held-out contract; do not open D3 here
if parity only:       retain functional parity and stop the curvature claim
if negative:          revise only projection, score, centroid, or fitter
if unavailable:       repair evidence/preflight without reading validation
cost estimate:        at most two hours after release build, eight fixed shards
```

## Empirical criteria

All NLL comparisons are paired over the same 64 construction-validation
targets in nats per token.

1. **Donor/gauge replay:** independent donor and gauge-reference executions
   are byte-identical to their own replays; their top-1 tokens match on all 64
   positions; gauge minus donor NLL is at most `0.002`; mean absolute logit
   delta is at most `0.002`; and every logit satisfies
   `abs(delta) <= 0.02 + 0.001 * max(abs(donor), abs(gauge))`.
2. **Functional retention:** learned-Lorentz NLL is at most donor NLL plus
   `0.05`.
3. **Matched learned parity:** learned-Lorentz NLL is at most
   learned-Euclidean NLL plus `0.05`.
4. **Geometry-specific branch:** learned-Lorentz NLL is at most
   learned-Euclidean NLL minus `0.01`, and each source-frame-permuted,
   value-permuted, and order/key-shuffled control has NLL at least
   learned-Lorentz NLL plus `0.02`.
5. **Causality and replay:** every arm reports zero reads from positions later
   than its query, zero target-as-input reads, the exact declared work, finite
   outputs, and byte-identical parameter/report replay.

Top-1 and decoded snippets are reported diagnostics. They cannot override the
paired NLL rules.

## Frozen terminals

- `PASS_HELM_D_LEARNED_MANIFOLD_R4_CONSTRUCTION_AUTHORIZE_HELDOUT_FREEZE`
  requires all availability, donor/gauge, functional-retention,
  matched-parity, geometry-specific, causality, and replay criteria. It
  authorizes only a new pre-outcome held-out freeze.
- `RETAIN_HELM_D_MANIFOLD_FUNCTIONAL_PARITY_NO_CURVATURE_ADVANTAGE` requires
  availability, donor/gauge replay, functional retention, matched learned
  parity, destructive-control separation, causality, and replay, but the
  learned Lorentz arm misses the `0.01` Euclidean advantage. The copied
  mechanism may be retained as functional reference data but earns no
  curvature claim.
- `FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING`
  applies when evidence is available but functional retention, matched parity,
  destructive-control separation, causality, or deterministic replay fails.
  The next action is confined to the named learned-manifold seam; more data,
  routes, dimensions, recurrence, and lowering are not authorized.
- `UNAVAILABLE_HELM_D_MANIFOLD_CONSTRUCTION_EVIDENCE` applies when source,
  population, hook ordering, numerical health, gradient/covariance preflight,
  liveness, checkpoint identity, worker execution, or no-clobber evidence
  cannot support a valid construction result. Validation remains unopened when
  the unavailable condition occurs before the checkpoint gate.

Every terminal keeps D3, multi-resonance replacement, recurrent factorization,
exact/table lowering, corpus/model scaling, E8 expansion, and #954 blocked.
Even the construction PASS only authorizes writing and merging a separate
held-out pre-outcome contract.

## Outcome append

`NOT_RUN`. The target-blind construction partition is frozen below. Add the
protected implementation revision, executable identity, measurements, and
terminal only after the contract and implementation merge and the applicable
seal has been opened in the declared order.

### 2026-08-29 construction-partition freeze

The exact, target-blind freezer completed before fitting or decision
execution. Its tracked envelope is
[`helm_d_learned_manifold_r4_construction_partition_973.json`](helm_d_learned_manifold_r4_construction_partition_973.json).

- manifest CID:
  `blake3:359b3270aaa0d3ac157280c9206ad820d18ee93932e0530552ebbb7935ac6410`;
- partition CID:
  `blake3:5c5a7dab9d7a0fbc9d176faafd49b42094ef89138cc32699dfc1b4fe937d1bde`;
- tokenizer CID:
  `blake3:70af0cb08bbcd3b323d3387ca1d7d33da39873820604d183711e8e99f9903fc1`;
- selected population: 16 construction-fit plus 8 construction-validation
  documents, with 24 distinct IDs and 24 distinct input CIDs;
- predecessor exclusion census: 28 document IDs plus 28 predecessor-domain
  input CIDs; and
- serialized validation commitments contain only ID, selection digest, title
  CID, both versioned input CIDs, and byte range. They contain no target value,
  target CID, D3 identity, or token sequence.

The learned fit, validation materialization, D3, and decision terminal remain
`NOT_RUN`. The implementation base revision and executable identity will be
bound only after protected delivery of this contract and freezer.
