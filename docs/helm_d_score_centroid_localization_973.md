# HELM-D R4 score-by-readout localization — issue #973

- **Status:** frozen design; implementation and run `NOT_RUN`
- **Mechanism:** `HelmDScoreCentroidLocalizationR4V1`
- **Question:** did learned-manifold V2 fail primarily because of its
  Lorentz compatibility score, its normalized Lorentz value centroid, or both?
- **Scope:** compiler-side construction evidence only; D3 remains `NOT_RUN`

## Why this is next

[`HelmDLearnedManifoldR4ConstructionV2`](helm_d_learned_manifold_r4_construction_973.md)
completed a valid non-D3 construction-validation run. The ordinary donor and
coherent R4/Spin gauge reference retained parity, but learned Lorentz scored
`7.71061809923296` NLL against donor `3.667626465210025` and matched Euclidean
`4.483153905078387`. All three destructive Lorentz controls were worse, so the
operator is sensitive to frame, value, and order interventions; that does not
establish useful learned geometric attention.

V2 changed score and value aggregation together. Its Lorentz score adds a
full-head radial/time penalty absent from donor dot-product attention, while its
normalized Lorentz centroid applies a context-dependent contraction before the
unchanged frozen Euclidean `W_o`. The completed result cannot attribute the
failure to one seam. This localization makes that attribution before another
full qualifier.

## Frozen operator

For transported query and key coordinates, retain ordinary complete-prefix
causal softmax and compare the existing Lorentz and Euclidean scores. For
values, compare the current normalized Lorentz centroid with a transported
tangent-fiber sum:

```text
normalized: r_M = normalize_L(sum_j a_ij Phi(v_j)).spatial
tangent:    r_T = sum_j a_ij P_(j -> i) v_j
```

The tangent arm keeps keys and queries on the Lorentz base manifold. Values are
parallel-transported vector sections and are not lifted and renormalized as a
second hyperboloid point before `W_o`.

Four arms cross exactly two score and two readout policies:

| Arm | Score | Value readout |
|---|---|---|
| `L-M` | Lorentz | normalized Lorentz centroid |
| `L-T` | Lorentz | tangent arithmetic sum |
| `E-M` | Euclidean | normalized Lorentz centroid |
| `E-T` | Euclidean | tangent arithmetic sum |

## Frozen population and fitting

Reuse only the 16 documents already designated construction-fit by partition
`blake3:5c5a7dab9d7a0fbc9d176faafd49b42094ef89138cc32699dfc1b4fe937d1bde`.
Do not read or rescore the eight revealed V2 construction-validation documents,
and do not open D3.

- score-fit eight, in existing order: `8503`, `7754`, `3956`, `7315`, `476`,
  `4749`, `7525`, `8141`;
- localization-audit eight, excluded from the new 32-step fit and kept in
  existing order: `6309`, `271`,
  `6749`, `7604`, `8384`, `7183`, `3621`, `3799`.

Freeze every value adapter at identity and share it byte-for-byte across all
four arms. Fix the uniform source-row bias to zero because it cancels under
softmax. Fit exactly two Q/K/temperature bundles for 32 full-batch steps using
donor-attention cross-entropy only: `theta_L` is shared by `L-M` and `L-T`, and
`theta_E` is shared by `E-M` and `E-T`. Preserve the V2 optimizer, learning
rate, ridge, R4 block capacity, RoPE, exact Spin/H4 transport, positions,
causal support, and deterministic ordered shard reduction.

The score-paired arms must have bit-identical logits and weights. Aggregate
loss cannot influence Q/K fitting. This makes `L-M` versus `L-T` an exact
centroid intervention rather than a coupled retraining comparison.

## Metrics and cheap hard gate

Report donor-attention cross-entropy and normalized aggregate MSE separately,
both per document and pooled. Also report attention entropy, Q/K radial-norm
quantiles, Lorentz normalization-factor minimum/median/p95/maximum, exact
work/causal ledgers, covariance, and replay.

Before the full trace, capture only fit documents `8503` and `7754` with
identity values. Require all of:

1. bit-identical logits/weights within `L-M`/`L-T` and `E-M`/`E-T`;
2. tangent aggregation satisfies the existing 120-frame covariance tolerance;
3. every normalization factor is finite and at least `1 - 1e-12`; and
4. `L-T` aggregate MSE is at least 10% below `L-M` on each document.

Failure of item 4 rejects the tangent-readout hypothesis in about four minutes,
terminates `REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`, and stops without
the remaining trace. Any identity, covariance, causality, arithmetic, work, or
replay failure is `UNAVAILABLE_OPERATOR_LOCALIZATION_EVIDENCE`.

If the cheap gate passes, run the 8/8 attention-level localization. Only if its
attention-level decision criteria pass may `L-M` and `L-T` enter the frozen
decoder for paired 64-position NLL/top-1 plus deterministic replay.

## Frozen decisions

- `SELECT_TANGENT_VALUE_READOUT_FOR_FRESH_CONSTRUCTION` requires `L-T`
  aggregate MSE below `L-M` on all eight audit documents and by at least 10%
  pooled, plus paired decoder NLL at least `0.05` below `L-M`, with every
  identity, covariance, causality, arithmetic, work, and replay gate passing.
  It authorizes only one fresh construction freeze using tangent readout.
- `REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT` applies when the two-document
  cheap gate rejects tangent readout. It authorizes only a separately frozen
  score-only preflight; it is not evidence that Euclidean score is better.
- `SELECT_FIXED_CURVATURE_SCORE_CONTINUATION` applies only after the full audit
  when tangent readout misses its audit criterion and Euclidean score
  cross-entropy beats Lorentz by at least `0.01` pooled and on all eight audit
  documents. It authorizes one bounded continuation between the Euclidean
  limit and unit curvature.
- `REVISE_PROJECTION_OR_FITTER` applies after the full audit when neither factor
  separates cleanly.
- `UNAVAILABLE_OPERATOR_LOCALIZATION_EVIDENCE` supports no scientific
  inference and authorizes only repair of the named evidence defect.

No terminal here establishes attention, curvature advantage, autonomous
generation, transformerless serving, correctness, reasoning, or D3. Resonance,
recurrence, E8 expansion, exact/table lowering, scale, and #954 remain blocked.

## Run contract

```text
metric to move:        audit normalized aggregate MSE, then paired decoder NLL
reachability ceiling:  every captured row traverses the selected score/readout;
                       score-paired logits and weights must be identical
cheap instrument:      two-document identity/covariance/tangent-MSE preflight
exit rule:             the five frozen terminals above
if tangent positive:   freeze one fresh tangent-readout construction qualifier
if score selected:     freeze one bounded curvature-score continuation
if neither separates:  revise projection or fitter only
cost estimate:         about 4 minutes fail-fast; 34-36 minutes attention-only;
                       66-70 minutes including paired decoder/replay after build
```

The implementation must use a separate score-metric enum and centroid-policy
enum. It must not encode tangent readout by pretending that Lorentz scores are
Euclidean geometry.
