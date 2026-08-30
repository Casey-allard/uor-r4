# HELM-D R4 score-radius preflight — issue #973

- **Status:** `PARKED`; frozen future contract, implementation and run `NOT_RUN`
- **Mechanism:** `HelmDScoreRadiusPreflightR4V1`
- **Question:** does any finite Lorentz score radius improve donor-attention
  agreement beyond the exact flat-distance endpoint while preserving the
  retained normalized geometric value readout?
- **Scope:** two construction-fit documents, attention weights and aggregates
  only; no fitting, decoder, old validation, or D3

## Evidence selecting this seam

Protected score-by-readout Attempt 01 returned
`REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`. Tangent value aggregation was
worse on both frozen documents, while the identity Euclidean score reduced
donor-attention cross-entropy relative to the unit-radius Lorentz score by
`1.1898520130933234` and `0.9698105504859882` nats per row. Pooled normalized
centroid MSE was effectively tied and slightly lower under Euclidean scoring:
`0.011342099168393727` versus `0.0113714841466095`.

The pooled median query/key radial norms were `14.18400239053336` and
`15.675111943027494`, far outside the unit-radius near-flat regime. This parked
future experiment therefore changes only the score radius. It retains the normalized
Lorentz centroid that beat tangent aggregation.

## Exact score decomposition

For transported spatial query and key coordinates `q` and `k`, define

```text
N_R(q,k) = 2 R^2
           - 2 sqrt(R^2 + ||q||^2) sqrt(R^2 + ||k||^2)
           + 2 q dot k
score_R  = N_R / 24
```

`R = 1` is the existing Lorentz compatibility score. The exact `Flat` endpoint
is

```text
N_E(q,k) = -||q-k||^2.
```

The limit `R -> infinity` is `N_E`. More importantly, the only finite-radius
correction is known exactly:

```text
N_R - N_E =
  (sqrt(R^2 + ||q||^2) - sqrt(R^2 + ||k||^2))^2 >= 0.
```

The preflight may evaluate finite radii as `N_E` plus that nonnegative term to
avoid subtractive cancellation. The `R=1` and `Flat` endpoints must still call
the existing frozen score operators so their bytes are exact predecessor
seals. This is a score-radius intervention, not a claim that the complete
value manifold or transport curvature changed.

## Frozen arms and inputs

Evaluate the fixed ordered grid

```text
R = [1, 2, 4, 8, 16, 32, Flat].
```

Reuse only construction-fit documents `8503` and `7754` under predecessor
partition
`blake3:5c5a7dab9d7a0fbc9d176faafd49b42094ef89138cc32699dfc1b4fe937d1bde`.
Use untouched identity Q, K, and V adapters, fixed scale `24`, and uniform bias
`+0.0`. Preserve donor RoPE, complete-prefix causal support, ordinary stable
softmax, and the normalized unit-Lorentz value centroid. Offline metrics remain
in the predecessor's `canonical_model_frame_gauge_equivalent` frame with atlas
transport status `NOT_EXECUTED_COVARIANCE_REDUCED`; they are not counted as
executed transport work. The exhaustive 120-frame covariance census and natural
token-derived atlas liveness separately seal equivalence to coherent exact
R4/Spin transport. Perform no parameter fit.

The implementation must not materialize the remaining localization fit or
audit documents, any V2 validation span or target, or D3. It performs no
next-token scoring and needs no decoder-target commitment.

## Endpoint and geometry seals

The endpoint branches must reproduce the protected localization result exactly:

| Document | Endpoint | Donor-attention CE | Logits CID | Weights CID |
|---|---|---:|---|---|
| `8503` | `R=1` | `5.127463072729906` | `blake3:13119b7ce36dc4de3bf1af94874223af29550177175dad57980d622b0c86254a` | `blake3:409c8ec7e9ee4b6a9e245dda0d291ec38a21b59c098b270cfa7583547300e419` |
| `8503` | `Flat` | `3.9376110596365828` | `blake3:96289e027dc21214ace936b6f881462cfc462bf1a76ed44e284a93da8d2fe11e` | `blake3:c3676e9368cb75a680bfc9a3bb21e043019e0d946e7b8f7b1b939eb1c2df10d7` |
| `7754` | `R=1` | `4.3762953128527595` | `blake3:ba4f0a611399432f2c944820531272c20f13417e812eb83abe288beedd1de21a` | `blake3:b2a90932904703fb35f5adc8b34a0aa9fd86b200c77cb953dc967657e7acac3e` |
| `7754` | `Flat` | `3.4064847623667713` | `blake3:94948d5f52b98ab7e4308ae7230d3eb615ca5a3aaa7350779a0d942871b8486b` | `blake3:6602907fa7afee13b6534394ea004d4f6ab4583d3831cb00430e58619a7210b9` |

Every finite-radius score must pass the same exhaustive 120-frame covariance
tolerance. The natural token-derived `[5, 9, 2]` atlas must keep coherent versus
source-permuted transport live. Report exact causal/work ledgers and require
byte-identical evaluation replay.

## Metrics

Report per document and pooled:

- donor-attention cross-entropy and KL divergence;
- normalized-centroid aggregate MSE;
- attention entropy;
- query/key radial-norm quantiles;
- the finite-radius correction's `p0`, `p50`, `p95`, and `p100` quantiles;
- logits and weight CIDs;
- covariance, causal-read, target-read, work, and replay evidence.

## Frozen decision

Evidence validity has terminal precedence. First require endpoint identity,
all-frame covariance, causal/work isolation, liveness, numerical health, and
replay to pass. Any defect returns
`UNAVAILABLE_SCORE_RADIUS_PREFLIGHT_EVIDENCE` immediately, before forming or
inspecting a metric-eligible radius set.

With evidence valid, form the eligible set of finite radii that satisfy all
three metric conditions below. If it is nonempty, choose `R*` within that set
by minimum pooled donor-attention cross-entropy, breaking an exact tie toward
the smaller radius. Return
`SELECT_FINITE_SCORE_RADIUS_FOR_UNTOUCHED_AUDIT` only when:

1. `R*` cross-entropy is at least `0.01` below `Flat` on each document and
   pooled;
2. `R*` cross-entropy is at least `0.05` below `R=1` on each document;
3. `R*` normalized aggregate MSE is no more than `1.01 * Flat` on either
   document.

That terminal authorizes only one untouched attention-level audit on the eight
previously frozen localization-audit documents using the selected radius
unchanged.

If the eligible set is empty, return
`REJECT_LORENTZ_RADIAL_TIME_CORRECTION_DECOMPOSE_FLAT_SCORE`. This rejects the
tested finite-radius correction and retains `Flat` only as the next control; it
does not establish geometry-native attention. The next separately frozen seam
would compare the flat-distance score with a donor-scaled dot score, because at
fixed query the only nonconstant difference is the Euclidean key-norm penalty:

```text
N_E(q,k) = 2 q dot k - ||k||^2 + constant(q).
```

The precedence rule above means an evidence defect can never be misreported as
an empty eligible set. `UNAVAILABLE_SCORE_RADIUS_PREFLIGHT_EVIDENCE` supports
only evidence repair.

## Run contract

```text
metric to move:        donor-attention cross-entropy on documents 8503 and 7754
reachability ceiling:  each evaluation has 4,320 rows and 54,000 source pairs
                       per arm; seven arms execute 30,240 rows and 378,000
                       pairs in the primary evaluation, and byte-identical
                       replay doubles physical score work to 60,480 rows and
                       756,000 pairs; trace capture remains 64 teacher forwards
cheap instrument:      endpoint byte seals plus 120-frame covariance
if finite positive:    one untouched 8-document attention audit at fixed R*
if finite negative:    separately freeze flat-distance versus dot/key-norm seam
if unavailable:        repair only the failed evidence boundary
cost estimate:         about four minutes trace capture plus seconds for grid
```

Ordinary causal-softmax R4/Spin attention remains the established reference.
This preflight cannot establish curvature advantage, softmax removal,
transformerless serving, generation, correctness, reasoning, scale, or #954.

## Parking decision — 2026-08-30

This contract is preserved for a possible future return to intrinsic-score
research, but no implementation or run is currently authorized. The active
gate is provider-free autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`)
generation using the credited HELM attention seam and UOR's pinned SmolLM2
`HuggingFaceLlamaOracle` decoder path. That reference is explicitly transformer-compatible,
f32/source-weight-backed, and not table-native, multiplication-free, or
transformerless. CLI and evidence come first; web and release work wait for a
positive coherent autonomous-generation result.

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
hosted-page promotion, and static-WASM work. The score-radius preflight remains a
**PARKED future contract**: this supersession neither authorizes its
implementation nor changes its frozen boundaries.
