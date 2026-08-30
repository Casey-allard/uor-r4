# HELM-D R4 score-by-readout localization — issue #973

- **Status:** frozen executable contract; decision run `NOT_RUN`
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

The protected runner is
[`scripts/run_helm_d_score_centroid_localization_973.sh`](../scripts/run_helm_d_score_centroid_localization_973.sh).
It compiles one test binary from a clean tracked revision. That protected
binary first runs the exact target-commitment freezer and keeps its locally held
one-token CIDs only in the local evidence cache; it then runs the exact decision
test, which must match any admitted decoder target to that commitment. These
CIDs are commitments, not hiding: their small token space is enumerable. Their
scientific seal comes from creating them only after the protected code revision
is immutable and never publishing them in Git before that revision executes.
The runner writes exclusive evidence paths and applies the independent
80-minute decision-process watchdog. No unfiltered ignored-test launch is
authorized.

## Pre-execution clarification — 2026-08-30

This append-only clarification resolves implementation choices that were not
fully specified above. It is part of the frozen contract and precedes every
execution result.

- The two-document hard gate uses the untouched initialization: identity Q, K,
  and V adapters, layer scale `24`, and uniform bias `+0.0`. It performs no fit.
  Only after that gate passes may the remaining 14 construction documents be
  captured and the two declared 8-document score fits begin.
- The reported Lorentz denominator is the pre-normalization future-timelike
  norm `sqrt(t_sum^2 - ||x_sum||^2)`. It must be finite and at least
  `1 - 1e-12`. Its reciprocal readout multiplier is reported separately and
  must be finite and positive.
- Query and key radial norms use deterministic nearest-rank `p0`, `p50`, `p95`,
  and `p100` summaries. Query samples span every evaluated score row; key
  samples span every evaluated causal source pair. Fit and audit populations
  are reported separately.
- Replay has three separately reported scopes: two byte-identical refits of the
  two logical score bundles, byte-identical attention-level metric replay, and,
  when admitted, full-decoder replay for `L-M` and `L-T`.
- If `L-T` passes the eight-document aggregate-MSE criterion but misses the
  paired decoder-NLL improvement, the terminal is `REVISE_PROJECTION_OR_FITTER`.
  The fixed-curvature terminal remains available only when tangent readout
  misses the audit-level criterion and the frozen Euclidean score criterion
  passes.
- For donor aggregate `d` and learned readout `r` of width `D`, normalized
  aggregate MSE is
  `sum_l (r_l - d_l)^2 / (D * max(||d||_2, 1)^2)`, matching the predecessor
  fitter's measurement while excluding it from the new CE-only score fit.

The exact full-decoder forward ledger is also frozen. Donor trace capture uses
two forwards per token (ordinary plus traced): `64` forwards on a two-document
preflight rejection and `512` forwards after all 16 documents are captured.
An admitted paired decoder comparison adds `512` forwards (`2` arms times
primary/replay times `8` documents times `16` positions), for `1,024` total.
The release build is outside the run allowance; the exact decision process is
independently capped at 80 minutes.

Offline fitting and attention-level metrics use the canonical model-frame
gauge, as the predecessor fitter did. This is the gauge-fixed form of the
transported calculation, not a claim that an atlas transport ledger executed
for every offline row. Equivalence is a hard gate: both normalized and tangent
readouts must pass the exhaustive 120-frame covariance census, and the natural
token-derived `[5, 9, 2]` atlas path must keep the coherent/permuted transport
intervention live. If the decoder stage is admitted, it separately executes and
audits exact token-derived Spin/H4 transport on every causal Q/K/V row.

The protected runner first invokes the same compiled test executable to freeze
locally held article-span, 17-token, and one-token target CIDs for only the eight
localization-audit documents. That artifact binds the generator's contract,
source, executable, and protected Git revision. It remains in the local
evidence cache because a one-token CID has a small brute-forceable preimage
space; it is not committed to Git. The decision requires the exact same
implementation identity before tracing, seals the artifact and manifest CIDs
in the attention checkpoint, and verifies every committed article span and
materialized target only after that checkpoint admits the decoder. It reports
the source population as
`MANIFEST_ONLY_WITH_COMMITTED_TARGET_SPANS_VERIFIED`; it does not falsely claim
a live whole-corpus hash. Neither process reads the eight V2 validation spans or
D3.

The locally held target manifest binds the freezer's complete localization
implementation identity (contract, predecessor, compiled source, executable,
Git revision, and clean-tree status). The decision requires that identity to
equal its own before any donor trace begins. Corpus verification at this seal is
`MANIFEST_ONLY_WITH_COMMITTED_TARGET_SPANS_VERIFIED`: the freezer commits the
exact article-span byte CID for each of the eight audit documents, and the
admitted decoder recomputes all eight span CIDs. This deliberately does not
whole-hash or open excluded V2-validation or D3 corpus spans.
