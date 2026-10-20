# Intrinsic Lorentz R4 attention pre-run contract (#973)

- **Status:** `NOT_RUN`
- **Contract revision:** `IntrinsicLorentzR4AttentionV1`
- **Owner:** #973 under programme root #820
- **Base revision:**
  `f1ab400827bd226f6af97138a63628cfd49af115` (protected merge of PR #1002)
- **Positive predecessor:**
  [`HELM-D-R4` full-decoder softmax parity](helm_d_r4_softmax_decoder_973.md),
  result CID
  `blake3:05eaad210198fbe39a0645c25b0c890c55d5f3d3dd8a1710472e976a637e2a07`
- **Machine-readable contract:**
  [`intrinsic_lorentz_r4_attention_manifest_973.json`](intrinsic_lorentz_r4_attention_manifest_973.json)
- **Result:** `NOT_RUN`
- **Final held-out partition:** `FROZEN_PRE_FIT`; commitment-only manifest CID
  `blake3:964c643e9ffcfdb64bacfcaf04e74fa02923e242a7bf965c89619f8626365c55`,
  with exact identities recorded in the dated append below

This file is an append-only experiment record. The pre-run contract below is
frozen before construction fitting. Partition identities, execution evidence,
and the result are added as later dated sections; this section is not rewritten
after an outcome is known.

## Outcome sought

The #1002 result established that ordinary full-prefix causal softmax attention
can be represented through coherent UOR R4/Spin frames in the complete frozen
SmolLM2 donor decoder. It was deliberately gauge-equivalent to the donor and
therefore established no intrinsic geometric advantage.

This rung asks one narrower question:

> Can a separately fitted product-hyperbolic operator over the existing
> transported four-lane R4 blocks retain the frozen donor behavior and beat a
> separately fitted, equal-parameter flat-R4 distance/centroid control on
> document-disjoint next-token loss, while two geometry-destroying controls
> lose under equal causal support and matched operator-row budgets?

The positive advances only to the multi-resonance weighting experiment. It does
not remove softmax, establish bounded recurrence, authorize exact lowering,
close #973, or unblock #954.

## Architecture decision

### Context

An ambient Euclidean R4 distance and its weighted Frechet mean are exactly the
ordinary squared Euclidean distance and arithmetic weighted mean. Calling that
pair intrinsic geometry would make the proposed geometric arm identical in
kind to its required plain control. The new arm therefore needs a non-flat
four-dimensional manifold while preserving the repository's sixteen explicit
R4 blocks per 64-lane attention head and the already qualified exact Spin/H4
frame transport.

The symbol **H^4** in this record means four-dimensional hyperbolic space in the
Lorentz model. It is not the **H4 Coxeter/binary-icosahedral route group**. The
existing H4-derived cumulative frame supplies a spatial action `R in SO(4)`;
the chosen H^4 representation lifts that action to `diag(1, R)`.

### Decision: product H^4 over sixteen R4 blocks

Each 64-lane query, key, or value head remains sixteen ordered four-spatial-lane
R4 blocks. Every block is lifted independently to one unit-radius H^4 factor.
One head therefore uses the ordered product `(H^4)^16`; it is not collapsed
into one full-head manifold.

This choice retains:

- the #1002 donor, learned Q/K/V and `W_o`, RoPE, GQA, residual/FFN stack,
  final norm, LM head, tokenizer, and complete causal prefix;
- exact cumulative UOR Spin/H4 route frames as the source of the spatial
  `SO(4)` transport;
- ordinary stable causal softmax for this rung; and
- one explicit, auditable metric and value-aggregation contribution per R4
  block.

### Considered alternatives

| Option | Decision | Reason and trade-off |
| --- | --- | --- |
| Product H^4 over sixteen four-lane R4 blocks | **Chosen** | Non-flat and R4-block-native; `SO(4)` transport lifts isometrically to the Lorentz model; normalized Lorentz barycenters are deterministic and copied from the declared HELM-D operator family without claiming upstream checkpoint parity. It adds `sqrt`/`acosh` and timelike-domain failure modes in the offline oracle. |
| Ambient flat R4 squared distance plus arithmetic weighted mean | **Required matched control** | This is the exact flat geometry baseline. It has the same 32 fitted coefficients per layer/head and the same causal prefix and softmax, but no curvature. It cannot be promoted as intrinsic geometry. |
| S3 geodesic distance and spherical mean | **Deferred** | It is R4-block-native but discards or separately models radial magnitude, has antipodal/log-map singularities, and would add another chart/fallback decision. It is a later seam revision only if this frozen H^4 experiment fails for a localized geometric reason. |
| One full-head H^64 Lorentz manifold | **Rejected for this rung** | It abandons the sixteen registered R4 block actions, introduces cross-block geometry not supplied by the current exact frame substrate, and is not an equal local replacement of the #1002 seam. |

### Consequences

- A strict curved-over-flat result can be described as curvature-specific value
  for this bounded operator and population. A tie can establish functional
  geometric attention but earns no curvature-advantage claim.
- The dense all-prefix comparison and ordinary softmax remain offline scientific
  oracle work. No deployed efficiency claim is available.
- Compiler-side f64, multiplication, allocation, `sqrt`, `acosh`, and
  fixed-order serial reductions are permitted and separately counted. They are not
  credited to the eventual table-native kernel.
- Every non-finite lift, invalid Lorentz distance, non-timelike centroid, or
  transport-health fault fails closed. There is no arithmetic-mean fallback in
  the curved arm.
- A negative redirects only distance, barycenter, transport, or construction
  fitting. It does not authorize more documents, route families, recurrence,
  resonance, E8 expansion, or exact lowering.

## Frozen operator definitions

### Curved arm: `IntrinsicLorentzR4AttentionV1`

For one transported spatial R4 block `x`, define the unit-radius Lorentz lift
and inner product:

```text
Phi(x)          = (sqrt(1 + ||x||^2), x)
<X,Y>_L         = -X_0 Y_0 + sum_(a=1..4) X_a Y_a
z(Q,K)          = max(1, -<Q,K>_L)
d_H4(Q,K)^2     = acosh(z(Q,K))^2
```

`max(1, ...)` is only the declared roundoff clamp at the Lorentz-domain
boundary. Non-finite values fail the arm.

For layer `l`, query head `h`, R4 block `b`, and source `j <= i`, the curved
score is:

```text
feature_(i,j,b) = -d_H4(Phi(qhat_i,b), Phi(kbar_(j->i),b))^2
logit_(i,j)     = sum_(b=0..15) a_(l,h,b) * feature_(i,j,b)
a_(l,h,b)       >= 0
alpha_(i,*)     = stable_causal_softmax(logit_(i,*))
```

For each value block, form the normalized Lorentz barycenter:

```text
S_b             = sum_(j<=i) alpha_(i,j) * Phi(vbar_(j->i),b)
denom_b         = sqrt(-<S_b,S_b>_L)
M_b             = S_b / denom_b
read_b          = s_(l,h,b) * spatial(M_b)
s_(l,h,b)       > 0
```

This record calls `M_b` a **normalized Lorentz barycenter**, not a Karcher or
Frechet mean. `-<S_b,S_b>_L` must be finite and at least `1e-12`; otherwise the
transport reports a fault and the decoder step returns no logits.

The exact cumulative source-to-query spatial frame action `R` lifts to
`diag(1,R)`. The preflight must demonstrate Lorentz-distance invariance and
barycenter covariance under that lift before fitting.

### Flat arm: `FlatR4DistanceAttentionV1`

The matched control uses the same sixteen ordered R4 blocks:

```text
feature_(i,j,b) = -||qhat_i,b - kbar_(j->i),b||^2
logit_(i,j)     = sum_(b=0..15) a_(l,h,b) * feature_(i,j,b)
alpha_(i,*)     = stable_causal_softmax(logit_(i,*))
read_b          = s_(l,h,b) * sum_(j<=i) alpha_(i,j) * vbar_(j->i),b
```

It receives the identical 16 nonnegative metric coefficients and 16 positive
output scales per layer/head, construction rows, fitting sweeps, causal support,
softmax, decoder weights, and work ledger.

## Frozen construction fitting

The donor weights never change. Each curved and flat arm independently fits
exactly 32 scalar coefficients per layer/query-head:

- 16 nonnegative block metric coefficients `a_(l,h,b)`; and
- 16 positive block output scales `s_(l,h,b)`.

With 30 layers and 9 query heads, each fitted arm contains exactly
`30 * 9 * 32 = 8,640` new scalars. Neither arm receives a hidden bias,
temperature, adapter, curvature, projection, candidate feature, or extra
checkpoint-selection parameter.

### Metric fit

For every construction causal attention row, apply `libm::log` to the frozen
donor's exact live-f32 causal attention weights, then subtract that log-row's
mean and independently subtract the row mean from each of the sixteen
block-distance feature columns. Fit these row-centered log weights by
nonnegative least squares with this frozen deterministic solver:

- initialize all metric coefficients to zero;
- use f64 ordered accumulations;
- use cyclic coordinate order `layer -> head -> block 0..15`;
- apply exactly 128 complete NNLS coordinate sweeps;
- use ridge denominator `lambda = 1e-6`; and
- update one coefficient by the standard residual coordinate step, clamped to
  `max(0, candidate)` before the residual is updated.

No convergence-based early stop or validation-selected sweep count is allowed.
Row centering removes the softmax-invariant additive score offset rather than
fitting an unidentifiable bias.

### Output-scale fit

For each layer/head/block, fit one scalar from the arm's query-frame
construction barycenter block `x` to the donor's linear aggregate block after
that aggregate is expressed in the same query frame, `y = F_i^T y_model`:

```text
s = max(1e-6, dot(x,y) / (dot(x,x) + 1e-6))
```

Dots use the complete construction rows in canonical document, position, and
lane order with f64 accumulation. The rule is applied once after the 128 metric
sweeps; there is no iterative refit or held-out calibration.

The implementation source, arithmetic revision, parameter bytes, construction
objective, and fit replay receive content identities before construction
validation is read.

The complete curved and flat parameters, primary fit reports, construction
objectives, primary/replay CIDs, implementation-source CIDs (including the exact
executor), executable CID, compiled contract CID, compiled partition-byte CID,
and verified Git source revision are written with exclusive-create semantics to
a fit-checkpoint sidecar. The harness immediately reads and verifies that
checkpoint and reconstructs both fitted arms from it; only then may it
materialize the four construction-validation documents. A pre-validation
interruption may resume from that exact checkpoint, but may not refit it in
place.

## Deterministic SimpleWiki partition contract

The raw population remains the 3,000-document SimpleWiki corpus at
`blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`.
The frozen donor remains
`blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`.
The exact tokenizer CID is not copied from an unbound path; it must be measured
and added with the selected token identities at the partition-freeze append.

For document ID bytes `id`, define:

```text
selection_digest(id) =
    BLAKE3("uor-r4.intrinsic-lorentz-r4-attention/1\0" || UTF8(id))
```

Selection is performed without reading any scored target:

1. Stream and verify the complete raw corpus bytes and document count.
2. Apply the existing D3 rule `d3_is_held_out(id)`, whose held-out condition is
   `BLAKE3(UTF8(id))[0] mod 5 == 0`.
3. Encode exactly `title + "\n\n" + text` with the frozen tokenizer. A document
   is eligible only when it has at least 17 tokens.
4. Among non-D3 documents, construction-validation candidates satisfy
   `selection_digest(id)[0] mod 5 == 0`; construction-fit candidates satisfy
   `selection_digest(id)[0] mod 5 != 0`.
5. Sort each candidate set by complete selection-digest bytes and then UTF-8 ID
   bytes. Take the first 16 construction-fit documents and first 4
   construction-validation documents.
6. Among D3-held-out documents, exclude the #1002 parity document whose exact ID
   is `12`; sort by the same key and take the first 8 eligible documents.
7. Reject any duplicate ID, duplicate encoded-input CID, count mismatch, raw
   corpus mismatch, tokenizer mismatch, or overlap among the three partitions.

For every selected document, process exactly causal input positions `0..15`.
Score observed next-token targets only for query positions `8..15`, using token
`position + 1` as the target. Denominators are therefore:

- construction fit: `16 documents * 8 = 128` scored positions;
- construction validation: `4 documents * 8 = 32` scored positions; and
- final D3 held-out: `8 documents * 8 = 64` scored positions.

The selection policy is frozen now. Exact selected IDs, titles, input-token
CIDs, target commitment, partition manifest CID, and tokenizer CID remain
`NOT_FROZEN_SELECTION_NOT_EXECUTED`. They must be appended and committed before
either held-out targets or final held-out metrics are read. Construction fit may
not be rerun after the eight D3 targets are opened.

## Matched arms

All arms use the same token inputs, complete causal prefix `0..=position`, every
30 decoder layers, 9 query heads, 3 KV heads, 64-lane head width, learned donor
Q/K/V and `W_o`, RoPE, GQA mapping, stable softmax, residual/FFN blocks, final
norm, LM head, worker policy, vocabulary, and decode rule. No arm may use top-k,
change causal support, skip a selected layer, or read a future position.

| Arm | Role | Fitting/intervention |
| --- | --- | --- |
| `ordinary_donor` | Frozen language reference | #1002 ordinary dot score and linear value aggregate; no new parameters. |
| `gauge_r4_reference` | Frozen transported reference | #1002 gauge-equivalent coherent R4/Spin dot score and linear aggregate; no new parameters. |
| `intrinsic_lorentz_r4` | Curved treatment | Product-H^4 distance, normalized Lorentz barycenter, and its independently construction-fitted 8,640-scalar artifact. |
| `flat_r4_distance` | Equal-budget learned control | Flat squared distance, arithmetic centroid, and a separately fitted 8,640-scalar artifact under the identical fitting contract. |
| `source_frame_permuted` | Geometry-destroying control | Reuse the curved artifact; at query position `i`, cyclically assign source-frame identities by `j -> (j + 1) mod (i + 1)` while leaving query frame, K/V content, causal positions, and work unchanged. |
| `value_permuted` | Binding-destroying control | Reuse the curved artifact and curved score weights; cyclically assign transported value blocks by `j -> (j + 1) mod (i + 1)` after scoring, leaving keys, weights, source positions, and work unchanged. |

The two permutations are necessarily identity at prefix length one. The scored
positions begin at 8, where both interventions must record nonzero changed
actions. Controls are never refitted.

## Cheap preflight and construction-validation gate

Final D3 held-out execution is forbidden until every item below passes:

1. The pinned HELM-D Lorentz golden fixture reproduces under the declared f64
   arithmetic.
2. For every exercised R4 block, the Lorentz lift is finite and lies on the
   positive unit hyperboloid within absolute tolerance `1e-9`.
3. Under `diag(1,R)`, squared distance is invariant within `1e-9` and the
   normalized Lorentz barycenter is covariant within `1e-8`.
4. Every softmax row is finite and sums to one within `1e-6`; every curved
   barycenter is future-sheet timelike with denominator at least `1e-12`.
5. The NNLS and ridge fits complete the exact declared work, emit 8,640 finite
   coefficients per arm, and reproduce parameter and construction-report bytes
   exactly under an independent replay. One fitted invocation binds 34,560
   unique causal rows and 432,000 unique source pairs, evaluated in two full
   geometric passes: 69,120 row evaluations, 864,000 source-pair evaluations,
   13,824,000 distance-feature block evaluations, 6,912,000 centroid
   source-block evaluations, and 2,211,840 output-scale lane accumulations.
6. The construction-validation full-decoder mean NLL satisfies both
   `NLL_curved - NLL_donor <= 0.05` and
   `NLL_curved - NLL_flat <= 0.05` nats/token.
7. The curved metric is live: at least one construction-validation causal row
   has maximum absolute attention-weight delta of `1e-4` or more from the
   separately fitted equal-budget `flat_r4_distance` control.
8. Causal and implementation audits report zero future reads, zero transport
   faults, exact selected-layer/head/prefix and fit-update counts, and a
   separate arithmetic census for the curved and flat operators. Equal
   parameter/support/row budgets do not claim equal arithmetic cost.

Any miss stops before D3 held-out. The final partition is reported `NOT_RUN`,
but a valid NLL or metric-liveness miss is a construction-validation negative,
not `UNAVAILABLE`.

## Final held-out metrics and empirical criteria

If the cheap gate passes, execute the six arms once on the frozen eight-document
D3 partition and report:

- full-vocabulary target NLL in nats/token (primary), perplexity, top-1, and
  top-8 over all 64 scored positions;
- paired per-position and per-document NLL differences;
- donor-logit KL and maximum/mean absolute logit differences;
- attention-weight differences, coefficient activity, barycenter timelike
  margin/faults, intervention counts, and complete causal/transport work;
- fit parameters, sweeps, serial fixed-order scalar operations, full-decoder
  steps, eight-worker donor/full-decoder utilization, and wall time; and
- one deterministic eight-token greedy continuation from the first held-out
  document after its 16-token prefix, decoded with the frozen tokenizer.

The continuation must be valid UTF-8, reproduce exactly, and contain no
period-1 or period-2 token cycle. It is printed verbatim; no subjective
coherence threshold is introduced.

The only positive terminal requires all of:

1. Curved donor/reference retention:
   `NLL_curved - NLL_donor <= 0.02` and
   `NLL_curved - NLL_gauge <= 0.02` nats/token, with curved top-1 no more than
   `1/64` below either reference.
2. Curvature-specific separation:
   `NLL_flat - NLL_curved >= 0.01` nats/token, curved NLL lower on at least
   `7/8` document means (the exact one-sided sign probability is `9/256`), and
   curved top-1 at least flat top-1.
3. Destructive-control separation:
   `NLL_source_frame_permuted - NLL_curved >= 0.02` and
   `NLL_value_permuted - NLL_curved >= 0.02` nats/token, with each control worse
   on at least `7/8` document means.
4. Zero faults and future reads, exact support and work, live intervention
   counts, byte-identical fit-artifact/report replay, exact curved-arm and
   decode replay, a deterministic scientific-result payload CID, and the
   bounded decode criterion above.

No metric may be rounded before a threshold comparison. Top-1 is secondary to
the paired full-vocabulary NLL criterion and cannot rescue a failed NLL gate.

## Run contract

```text
metric to move:       final D3 document-blocked next-token NLL, current NOT_RUN
reachability ceiling: 64/64 scored positions traverse all selected layers,
                      heads, full causal score rows, and value aggregation;
                      there is no sparse-support ceiling
instrument + verdict: Lorentz invariance/numerics, deterministic fit replay,
                      metric-live delta >= 1e-4, construction-validation
                      curved <= donor+0.05 and <= flat+0.05, zero faults/future
                      reads; every item must PASS before D3
exit rule:            apply the four-part final held-out empirical criterion
if positive:          freeze the curved oracle and authorize only the bounded
                      multi-resonance weighting replacement inside #973
if negative:          freeze the terminal and revise only distance, barycenter,
                      transport, or construction fit; do not run resonance,
                      recurrence, scale, E8 expansion, or exact lowering
cost estimate:        45-70 minutes after build on the same M1; a 75-minute
                      cooperative deadline plus an independent 75-minute
                      process watchdog; donor/full-decoder execution uses eight
                      fixed workers, while fitter passes and reductions are
                      fixed-order serial
```

For one 16-position document, the complete causal work ledger expects 4,320
query and output head calls and 36,720 key plus 36,720 value source calls. Across
the eight final documents, each full-prefix arm expects 34,560 query/output
head calls and 293,760 key plus 293,760 value source calls. R4 block counts and
intervention counts must derive exactly from these logical calls. A mismatch is
invalid evidence, not a performance result.

## Terminal decisions

| Terminal | Binding consequence |
| --- | --- |
| `PASS_INTRINSIC_LORENTZ_R4_ADVANCE_TO_MULTI_RESONANCE` | Every final criterion passes. Freeze this curved softmax oracle and authorize only the matched multi-resonance weighting replacement. This is a bounded curvature-specific result, not a serving claim. |
| `RETAIN_INTRINSIC_FUNCTIONAL_PARITY_NO_CURVATURE_ADVANTAGE_STOP_BEFORE_RESONANCE` | Curved retains donor/reference behavior but fails strict curved-over-flat or destructive-control separation. Preserve functional intrinsic attention without a curvature-advantage claim; resonance remains blocked. |
| `FAIL_INTRINSIC_LORENTZ_R4_CONSTRUCTION_VALIDATION_STOP_BEFORE_HELD_OUT` | A valid curved fit fails the frozen construction-validation NLL or metric-liveness gate. Preserve the negative, leave final D3 `NOT_RUN`, and revise only distance, barycenter, transport, or fit under a new freeze. |
| `FAIL_INTRINSIC_LORENTZ_R4_REVISE_DISTANCE_CENTROID_OR_TRAINING_SEAM` | A complete, publishable revealed run misses donor/reference retention or deterministic replay/decode integrity. Revise only the named operator/fitting seam; do not add scale or recurrence. |
| `UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` | Required corpus/tokenizer/frame/fit/audit evidence is missing or the cheap gate cannot validly run. Final held-out remains `NOT_RUN`; no metric inference is made. |
| `INVALID_INTRINSIC_LORENTZ_R4_POST_REVEAL_EVIDENCE` | An error, timeout, or incomplete record occurs after D3 admission. The revealed run is invalid evidence; it must never be relabeled as having stopped before held-out or interpreted as a metric result. |

There is no branch in which a tie with the flat arm authorizes a
geometry-specific claim or multi-resonance work.

## Definition of done for this rung

- This contract and machine manifest are committed before fitting.
- The deterministic selection implementation emits and binds exact fit,
  construction-validation, and held-out IDs/token CIDs plus the tokenizer and
  partition identities before any final held-out execution.
- Curved and flat artifacts bind exactly 8,640 fitted scalars, the same 128
  fixed-order NNLS sweeps, the same construction rows, and no donor-weight
  change.
- Every cheap preflight and construction-validation item has a non-vacuous
  result before D3 is opened.
- All six final arms run on byte-identical causal inputs and report full-vocab
  NLL, document-blocked direction, top-1/top-8, decode, work, faults, and
  interventions.
- The immutable pre-validation checkpoint preserves both full 8,640-scalar
  artifacts, fit reports/objectives, replay identities, and exact implementation
  identity. Before D3 access, an exclusive durable reveal sidecar binds that
  checkpoint and partition; an existing reveal or result refuses every later
  refit/overwrite attempt.
- The result, checkpoint, reveal marker, and process lock live in one canonical
  ledger keyed only by the frozen partition CID, not by a caller-selected
  filename, under `.uor-models/research/issue-973-intrinsic-lorentz-r4/` in the
  frozen local model store. An operating-system exclusive lock is held from the
  initial guard through terminal publication, preventing same-partition
  concurrent runs.
- Every checkpoint, reveal, result, and failure is fully written and synced to
  a same-directory temporary file, then atomically published with a no-clobber
  hard link. A crash can leave an ignorable temporary file but cannot expose a
  partial final evidence file. On startup, a reveal marker without a valid
  terminal result is reconciled to
  `INVALID_INTRINSIC_LORENTZ_R4_POST_REVEAL_EVIDENCE` without reopening D3.
- Fit artifacts/reports and curved logits/state/decode replay exactly. The
  scientific result payload receives a deterministic CID; preparation,
  worker-snapshot, resume, and wall-time telemetry live outside that hash, so
  varying operational timing cannot falsify scientific replay.
- Exactly one terminal above is emitted, with no threshold reinterpretation.
- Multi-resonance, recurrence, exact lowering, scale, paired-E8 expansion,
  #954, correctness, reasoning, chat, and release remain `NOT_RUN` or blocked
  unless and until their own later contracts become eligible.

## Current evidence ledger

| Evidence | Status |
| --- | --- |
| #1002 ordinary donor and coherent R4/Spin softmax parity | `PASS`; frozen predecessor only |
| Architecture decision and selection policy | `FROZEN_PRE_RUN` |
| Exact selected document IDs/tokenizer/token CIDs/partition CID | `FROZEN_PRE_FIT`; commitment-only manifest below |
| Lorentz invariance/numerical preflight | `NOT_RUN` |
| Curved and flat construction fitting | `NOT_RUN` |
| Construction-validation full-decoder gate | `NOT_RUN` |
| Final eight-document D3 reveal | `NOT_RUN` |
| Autonomous eight-token continuation | `NOT_RUN` |
| Terminal result | `NOT_RUN` |
| Multi-resonance replacement | `NOT_RUN`; blocked |
| Recurrent factorization and exact lowering | `NOT_RUN`; blocked |
| #954 and downstream capability stages | blocked |

## Nonclaims

This pre-run contract establishes no intrinsic attention result. A later
positive would establish only one bounded, donor-retaining,
curvature-specific attention result for the frozen SimpleWiki population and
decoder. It would not establish softmax removal, subquadratic inference,
transformerless serving, broad language quality, correctness, reasoning,
energy advantage, E8 semantics, chat, or release readiness.

## Exact partition freeze — 2026-08-29

The deterministic selection-only pass completed before construction fitting.
The byte-complete frozen artifact is
[`intrinsic_lorentz_r4_attention_partition_973.json`](intrinsic_lorentz_r4_attention_partition_973.json).

```text
schema:             uor-r4.intrinsic-lorentz-r4-attention-partition/1
manifest CID:       blake3:964c643e9ffcfdb64bacfcaf04e74fa02923e242a7bf965c89619f8626365c55
partition CID:      blake3:cad3dfd17159fdacc5c40e38753109c11764117e3c960f42b9b198d5731272a1
ordered D3 targets: blake3:5543e39457c6a990eb1d2de0e4eddf724467750cfe54297a7c81614c0e65c2da
tokenizer CID:      blake3:70af0cb08bbcd3b323d3387ca1d7d33da39873820604d183711e8e99f9903fc1
```

The exact label-free document membership is:

- construction fit: `3466` Portugal; `4326` Denial; `8548` War communism;
  `7796` Thomas Dolby; `7636` Court; `7554` January 8; `431` Provinces and
  territories of Canada; `7593` Michael Moore; `453` Political divisions of
  China; `7905` Zookeeper; `9148` Minerva (automobile); `3890` Bolus; `195`
  Devil; `310` Google; `358` Home page; `309` God's eye view;
- construction validation: `9252` Relativity; `9438` Demand; `3762` Cricket;
  `7328` Enid Blyton; and
- final D3 held out: `4586` 1621; `6617` Province; `8561` Belfast; `4964`
  Homer; `6828` Quark; `8152` Tunnel; `4700` Fixed-wing aircraft; `8639`
  Socks (disambiguation).

Every document's selection digest, encoded-token/input/target CIDs, and exact
corpus byte span is committed in the frozen artifact. No raw token sequence is
serialized. Construction documents are materialized and CID-verified from
their spans before fitting; D3 spans cannot be read until the validation gate
passes, and their ordered target CIDs must reproduce the aggregate commitment
above after admission. Two independent freezes were byte-identical. The earlier
`NOT_FROZEN_SELECTION_NOT_EXECUTED` ledger entry records the contract-authoring
state; this append advances only exact partition selection to
`FROZEN_PRE_FIT`. Fitting, construction validation, held-out execution,
decoding, and the terminal remain `NOT_RUN` at this checkpoint.

## Prelaunch evidence-integrity amendment — 2026-08-29

Before fitting, the executable contract was tightened without changing the
frozen operator, partition, softmax, arms, thresholds, or decision branches:

- the metric target is precisely the row-centered `libm::log` of the donor's
  live-f32 causal attention weights—not an unavailable pre-softmax tensor;
- the fitter and reductions are fixed-order serial; eight workers apply to
  donor trace capture and full-decoder execution;
- validation documents remain unread until both complete fitted parameter
  arrays, fit reports/objectives, replay identities, the exact executable, the
  exact executor source, and a real Git commit equal to a clean tracked `HEAD`
  have been atomically written and verified from an immutable checkpoint;
- the output environment must name the one partition-CID-scoped canonical
  ledger. An exclusive process lock is held through final result/failure
  publication, and D3 admission first writes an exclusive reveal marker binding
  the checkpoint, implementation, manifest, and partition. Any existing result
  or reveal marker refuses later refitting or overwriting under every filename;
- checkpoint, reveal, result, and failure publication is atomic and
  no-clobber. A published reveal path is treated as post-reveal even if
  temporary-file cleanup or the following directory sync reports an error.
  Reveal-without-valid-result recovery emits the post-reveal invalid terminal
  without reading D3 again, and a restart reuses that no-clobber reconciliation;
- addressed marker and terminal payloads hash recursively key-sorted compact
  JSON. Post-reveal recovery recomputes the actual result CID and accepts only
  `PASS`, `RETAIN`, final `FAIL`, or `INVALID_POST_REVEAL` with matching schema,
  issue, manifest CID, partition CID, reveal-marker CID, and held-out shape;
  pre-reveal validation failures and `UNAVAILABLE` cannot suppress
  reconciliation; and
- the content-addressed scientific result excludes preparation, execution
  snapshot, resume status, and elapsed-time telemetry. Those operational facts
  remain in the enclosing record but cannot make a deterministic scientific
  payload appear non-reproducible.

The integrity amendment resolves pre-run evidence defects only. Construction
fitting, validation, D3 held-out execution, and every result terminal remain
`NOT_RUN` until the committed revision is bound on #973 and the bounded run is
launched.

## Exact launch procedure — 2026-08-29

The committed launcher is the only authorized invocation:

```bash
bash scripts/run_intrinsic_lorentz_r4_attention_973.sh
```

It refuses a dirty tracked checkout, binds the compiled executable to the exact
40-character `HEAD` revision, builds the one release test offline, extracts
that exact executable from Cargo's machine output, and runs only
`intrinsic_lorentz_r4_full_decoder_decision`. Runtime inputs are fixed to
canonical deterministic mode, the committed partition manifest, exactly eight
donor/full-decoder workers, and the partition-CID-scoped canonical result path.
The harness enforces a cooperative 75-minute deadline between bounded
operations; the launcher independently applies `SIGALRM` to the test process at
4,500 seconds, so an operation that fails to return cannot evade the wall
limit. A watchdog termination after D3 reveal is recovered on the next launch
as `INVALID_INTRINSIC_LORENTZ_R4_POST_REVEAL_EVIDENCE`, never as a metric
terminal and never by reopening D3.

## Attempt 01 unavailable before reveal — 2026-08-29

The first authorized execution preserved the complete construction fit, then
stopped while verifying the newly published fit checkpoint. Its append-only
terminal is:

```text
attempt:               01
result file:           result.json
result CID:            blake3:e180e240f3cfc490dbe04b2864184760e31355be940d33864bb165d182069a73
terminal:              UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT
stage:                 construction.fit.checkpoint_write
elapsed seconds:       858.992150625
held-out opened:       false
D3 reveal marker:      absent
declared checkpoint:   blake3:85b5251cff215c2480dac172cc351e09914493b40d58c59f007ca41c437b4bee
read-back checkpoint:  blake3:38407af4e77147d5b1ad70a61d70eade5135cba92febedffeb6059b105e3fd66
```

The failure was in JSON floating-point round-trip identity: the default parser
can reconstruct some fitted decimal values one ULP away from the value whose
bytes were originally hashed. It was not an attention metric outcome.
Construction validation remained unread, D3 remained sealed, and no loss,
accuracy, decode, curvature, or control inference is permitted. The attempt-01
result and checkpoint remain byte-for-byte in their original canonical paths;
they are not moved, deleted, overwritten, or adopted by a later executable.

## Attempt 02 append-only repair freeze — 2026-08-29

Exactly one repaired execution is authorized at the separately addressed,
code-fixed result path
`result.attempt-02-checkpoint-float-roundtrip.json`. It enables exact JSON
floating-point parsing and refits from the unchanged construction population;
it does not reuse or copy attempt 01's checkpoint. The frozen manifest,
16/4/8 data split, donor, intrinsic and flat mechanisms, arms, work budgets,
softmax, thresholds, and terminal branches are unchanged.

This amendment supersedes only the earlier single-result-file procedure. The
partition ledger is append-only: every predeclared attempt result and checkpoint
is independently no-clobber, while `d3-revealed.json` is one partition-global
irreversible reveal marker. Therefore attempt 01 can remain durable while
attempt 02 starts pre-reveal, but any D3 reveal blocks every later attempt on
this partition. The existing partition-root process lock still serializes the
entire execution. If attempt 02 stops before reveal, it is preserved and there
is no attempt 03 without a new explicit evidence amendment. If it is
interrupted after reveal, startup may only reconcile it to
`INVALID_INTRINSIC_LORENTZ_R4_POST_REVEAL_EVIDENCE`; it may never reopen D3.

Current status is `ATTEMPT_01_UNAVAILABLE_PRE_REVEAL` and
`ATTEMPT_02_REPAIR_FROZEN_NOT_RUN`. Multi-resonance, recurrence, lowering,
scale, #954, and all capability claims remain blocked.

## Attempt 02 construction-stage terminal — 2026-08-29

Attempt 02 completed from repair commit
`a348d7ec31f75524906a39cc04327bbcbdd47a56`. The correctly rounded checkpoint
readback passed, the fitted artifacts replayed exactly, and construction
validation ran. The complete canonical result is preserved byte-for-byte in
[`intrinsic_lorentz_r4_attention_result_973.json`](intrinsic_lorentz_r4_attention_result_973.json),
and its compact evidence summary is
[`intrinsic_lorentz_r4_attention_attempt_02_summary_973.json`](intrinsic_lorentz_r4_attention_attempt_02_summary_973.json).

```text
attempt:                    02
result file:                result.attempt-02-checkpoint-float-roundtrip.json
result bytes:               525754
result file BLAKE3:         blake3:3f8ba48d9830eca0636df37eafa8af167475005970e77f5af6f3bec20d191518
embedded result CID:        blake3:da2a63323d6211b8d581e5a4ed75d788eb919ff0f210d2e3beb8a749ee1bc64f
fit checkpoint CID:         blake3:0372ae31b6464c4967c07b70f7d4bd3cea437c971c34e16ab1b8048630144dc2
terminal:                    UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT
elapsed seconds:             1800.35884825
held-out opened:             false
D3 reveal marker:            absent
```

The unavailable terminal is required by the frozen precedence rule. The
Lorentz barycenter covariance diagnostic reached
`9.121400701417315e-8` against the frozen `1e-8` ceiling. The other geometric
health checks passed: hyperboloid residual `8.526512829121202e-14`, distance
invariance delta `5.5261351050717167e-14`, softmax-sum delta
`4.272442311048508e-8`, timelike denominator squared
`1.000000000089752`, and exact HELM-D golden reproduction. Parameter replay,
fit-report replay, work/shape, donor-trace identity, causal reads, and liveness
also passed; the curved-versus-flat attention-weight delta was
`0.988744561560452`.

The construction-validation measurements are retained as diagnostics, not a
held-out scientific terminal. On 32 construction-validation positions, donor,
curved, and flat mean NLL were `2.6513639557648623`, `3.9044978436394797`, and
`3.6955705255518287` nats. The curved arm therefore missed the frozen donor
margin by `1.2531338878746174` and the flat margin by
`0.20892731808765097`, far beyond the `0.05` ceiling. Its construction fit
objective was also higher than flat (`415554.66305666673` versus
`347686.69506399677`). These observations diagnose the next seam, but the
failed covariance audit means they are not relabeled as the clean
`FAIL_INTRINSIC_LORENTZ_R4_CONSTRUCTION_VALIDATION_STOP_BEFORE_HELD_OUT`
terminal. D3 loss, accuracy, controls, and decode remain `NOT_RUN`.

### Binding conclusion and next action

The positive #1002 result remains intact: ordinary dense causal Q/K/V softmax
attention is established on the bounded full decoder in coherent UOR R4/Spin
frames. Attempt 02 does not establish intrinsic curvature-specific attention.
It rejects neither ordinary attention nor R4 transport; it shows that this
post-hoc `acosh^2` score, normalized Lorentz centroid, and coefficient-only fit
is not ready for held-out use.

There is no silent attempt 03 and no tolerance-only rerun. The next #973
construction-only freeze must copy the pinned HELM-D semantic seam more
faithfully: manifold-valued learned Q/K/V projections, the declared
Lorentz-inner-product score with learned scale/bias, and a numerically stable
equivariant Lorentz centroid, with an equal-capacity Euclidean arm. It must use
fresh non-D3 construction validation, pass covariance without relaxing the
frozen mathematical bound, and retain donor behavior before any D3 reveal.
Multi-resonance, recurrence, exact lowering, scale, #954, correctness,
reasoning, and product claims remain blocked.

### Learned-manifold V2 outcome and current successor

The source-faithful V2 successor completed a valid non-D3
construction-validation run. Donor/gauge parity and all destructive-control
separations passed, but learned Lorentz failed donor retention and matched
Euclidean parity; the controls establish sensitivity only. The sole current
#973 action is the frozen 8/8
[score-by-readout localization](helm_d_score_centroid_localization_973.md).
D3 remains `NOT_RUN`; resonance, recurrence, lowering, scale, and #954 remain
blocked.

## Score/readout outcome and superseding direction — 2026-08-30

Protected score-by-readout Attempt 01 returned
`REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`: tangent aggregation was worse
on both untouched-parameter preflight documents; the lower flat-score
cross-entropy was diagnostic only. The maintainer has parked intrinsic
score/readout, score-radius, resonance, recurrent, and softmax-replacement
research. The accepted baseline is ordinary dot-product/stable-softmax causal
attention in coherent R4/Spin frames. The active #973 gate is provider-free
autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation with the
credited HELM attention seam and UOR's pinned SmolLM2
`HuggingFaceLlamaOracle` decoder path, CLI and replayable evidence first. It remains transformer-compatible,
f32/source-weight-backed, and not source-free, table-native, multiply-free, or
transformerless; web and release work wait for coherent generation.

## Generator qualification supersession — 2026-08-30

The bounded [`R4SoftmaxReferenceGeneratorV1` qualification](r4_softmax_reference_generation_973.md)
is now **PASS** at
`PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`: its
eight-token canary replayed exactly, the frozen five-prompt smoke passed at
4/5 in both passes, and all 5/5 run pairs replayed exactly after deleting only
timing. All 30 layers were selected; every recorded causal, projection, and R4
audit was exact with zero future reads. The source donor matched P1 through EOS
and P2-P5 for all 32 retained tokens. The
[compact aggregate](r4_softmax_reference_generation_attempt_01_result_973.json)
binds the outputs, CIDs, audits, timings, provenance, and nonclaims.

This supersedes only the active-next-action wording above, not this record's
historical outcome. HELM is the credited MIT architectural reference pinned at
`7501deca8f413848bfef804be64ce874b72a3cd7`; no HELM checkpoint or generation
code executed. The executed stack is UOR's pinned SmolLM2
`HuggingFaceLlamaOracle`. The result is a source-weight-backed `f32`/matmul,
Transformer-compatible ordinary dot-product/stable-softmax reference in
coherent R4/Spin frames. It does not establish geometry advantage, softmax
removal, source-free/table-native or transformerless inference, general
quality, correctness, reasoning, frontier capability, browser-WASM operation,
or release readiness. #973 remains open and #954 remains blocked. The next
authorized step is an **explicit opt-in native HTTP/dashboard bridge** for this
exact policy, with no default-engine change and latency qualification only as
needed for one real end-to-end prompt. It remains separate from tag/release,
hosted-page promotion, and static-WASM work.
