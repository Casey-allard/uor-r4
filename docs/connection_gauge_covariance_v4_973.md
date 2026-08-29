# Connection-gauge covariance V4 (#973)

Status: `PHASE_I_PASS_PENDING_PROTECTED_MERGE`; V4 validation inputs, labels,
predictions, and scores are `NOT_RUN`.

This record is append-only. It defines the pre-score contract for
`ConnectionGaugeCovarianceV4`, the next bounded rung after the frozen V3
negative result. V4 asks whether ordinary one-head causal attention is
representation-covariant when its Q/K/V/O coefficients are stored directly in
three-coordinate tangent gauges. It does not test a geometry-specific
advantage, paired E8/fiber input, natural-language transfer, a resonance
replacement for softmax, bounded recurrence, or the deployed integer runtime.

## Three-stage chronology

V4 uses two public pre-label checkpoints followed by a one-way reveal.

1. **Phase I — mechanism and geometry.** Fit and inspect only the 16 already
   public construction documents. Freeze the implementation, trained artifacts,
   complete 120-frame manifest, generator rule, controls, tolerances, and
   terminal thresholds. No V4 validation row exists in this phase.
2. **Phase II — target-free input population.** Starting from the protected
   Phase-I merge without changing the mechanism or thresholds, derive 12
   matched input pairs from that merge identity. Commit their target-free
   prefixes, disjointness report, input CID, salted label commitment, and final
   pre-label manifest. Produce no prediction.
3. **Phase III — one-time reveal.** Reproduce both freezes, produce two
   byte-identical label-free prediction streams, reveal and verify the label
   preimage once, score once, and preserve the result. A repair after reveal is
   V5 with a new population, never a V4 retune.

The correct label is structurally inferable from each prefix because it is the
value following the sole earlier query token. The chronology therefore proves
non-adaptation, not secret or cryptographically blind labels. The salted label
commitment provides integrity only.

## Frozen Phase-I mechanism contract

Each token and role owns an unconstrained local coefficient
`theta in R^3`. H4-compatible, alternative-tangent, fixed-frame plain, and
fixed-frame current-only arms own separate Q/K/V/O tables. The H4,
alternative, and plain arms begin from identical coefficient bytes and train
separately. Coefficients receive additive analytical-gradient updates and are
not normalized; normalizing them would reduce the effective parameter count
from three to two.

For unit H4 root `g`, define the oriented H4-compatible tangent basis

```text
B_H(g) = [g*i, g*j, g*k].
```

The alternative basis is deterministic lexicographic Gram-Schmidt against
`g`, with the final tangent column flipped when required to make the full
frame `[g | B_A(g)]` positively oriented. The fixed comparator uses base
`e0` and basis `[e1,e2,e3]`.

Two connection objects are kept distinct:

```text
P(d <- s) = B(d) B(s)^T
C(d <- s) = d s^T + P(d <- s).
```

`P` is a rank-three tangent transport: `P B(s) = B(d)`,
`P^T P = I - s s^T`, and `P P^T = I - d d^T`. It is not a full orthogonal
base-mapping matrix. `C` is the full orthogonal extension: `C s = d` and
`C^T C = I`. For the H4 basis, `[g | B_H(g)]` is the left-quaternion matrix,
so `C_H(d <- s) = L(d s^-1)` and `P_H` agrees with that action on source
tangent vectors.

With `beta = 1 / (sqrt(3) * temperature)`, the local operator is

```text
z_i     = beta * dot(q, k_i)
alpha_i = stable_softmax(z)_i
r       = sum_i alpha_i * v_i
score(c)= dot(o_c, r).
```

For a fixed true candidate `y`, fixed negative `n`,
`delta = o_y - o_n`, and margin `m = dot(delta,r)`, define

```text
h_i       = alpha_i * dot(delta, v_i - r)
grad(q)   = beta * sum_i h_i * k_i
grad(k_i) = beta * h_i * q
grad(v_i) = alpha_i * delta
grad(o_y) = r
grad(o_n) = -r.
```

Repeated-token K/V contributions are summed before their shared token
parameter is updated. Training uses a unit-margin hinge: an event updates only
while its fixed-target contrastive margin is strictly below `1.0`. This stops
unbounded margin growth without normalizing, clipping, or reducing the three
local degrees of freedom. The hard negative is held fixed during the
analytical and finite-difference comparison.

The causal row is inclusive (`i <= t`), support remains `[5,6]`, query token
remains `1`, and the fit remains 80 epochs, learning rate `0.04`, and
temperature `0.30`. Construction documents are sorted by document ID; within
each document, updates remain in causal prefix order. A numerical decision
whose winning score gap is below `1e-8` fails as a tie.

## Frozen controls and thresholds

The three main arms are separately trained H4, alternative-tangent, and
fixed-frame plain attention. Current-only is also separately trained. The
order-shuffled, value-permuted, and deliberate gauge-mismatch controls are
inference interventions over their named frozen main arm. The gauge mismatch
encodes H4 coefficients through an alternative source leg while retaining the
H4 destination leg; it preserves shape and tangent work but breaks coherent
gauge cancellation.

Phase I must satisfy all of:

- H4, alternative, and plain construction decisions are each `16/16`.
- Their logits, weights, scores, selected tokens, and local update deltas have
  numerical and decision parity.
- Every trace reads zero future token values and byte replay is deterministic.
- The frame manifest contains all 120 canonical H4 roots and the exhaustive
  `120 x 120` ordered-pair audit passes. Composition is checked over all
  `120^3` triples if its focused runtime remains practical.
- Every active Q/K/V/O coordinate in a representative construction event,
  including repeated-token accumulation, passes a central finite-difference
  audit.

Phase III is positive only if all of:

- every main arm selects at least `18/24` frozen targets;
- current-only selects at most `12/24`;
- order-shuffled, value-permuted, and gauge-mismatch each select at least six
  fewer targets than their bound main arm;
- the construction, covariance, causal-input, commitment, and deterministic
  replay gates still pass unchanged.

The structural maximum-absolute tolerance is `1e-12`. Forward covariance uses
absolute `1e-11` and relative `1e-10`; analytical gradient/update covariance
uses absolute `2e-10` and relative `2e-9`. Central differences use
`h = 2^-16 * max(1, abs(theta_j))`, absolute `2e-8`, and relative `2e-6`.

If Phase I fails, later phases remain `NOT_RUN`. If Phase III passes, the next
rung binds the real paired H4/E8 hierarchy/fiber inputs with a matched disabled
arm. If it fails, the negative result is preserved and paired E8, resonance,
recurrence, and lowering remain blocked.

## Frozen Phase-II generator rule

The seed source is exactly the 40 lowercase ASCII hexadecimal bytes of the
protected Phase-I merge commit. No model output enters generation or selection.
For each unsigned-16-bit candidate counter in `0..=65535`, the frozen
`uor-r4.cgcv-v4.pair-order/1\0` hash orders candidate pairs. The
`uor-r4.cgcv-v4.unit-order/1\0` domain independently orders the eleven indexed
units
`[1,5]`, `[6]`, `[2]`, `[3]`, `[4]`, `[7]`, `[8]`, `[9]`, `[10]`, `[11]`,
and `[12]`. Variable bytes are length-prefixed with little-endian `u64`; tokens
are little-endian `u32`; counters and unit indexes are little-endian `u16`.
Unit-hash ties resolve by unit index and pair-hash ties by numeric counter.
Concatenate the ordered units and append the final query token `[1]`.
The matched mate swaps every `5` and `6`. This involution preserves length and
token multiset and changes the sole earlier query binding. Select the first 12
pairs in pair-hash order that satisfy all frozen eligibility rules.

Each selected prefix must be unique, end in query token `1`, contain exactly
one earlier query token, and be an antichain member against its mate, every
prior selected V4 prefix, and every complete construction, V2, and V3 prefix.
The exact forbidden-root, case-ID, and validation-prefix-root BLAKE3 domains
and byte encodings are frozen by
`CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY` in the implementation. Phase
II declares exactly 12 targets of each support token, but does not publish
per-case labels or the commitment nonce.

## Phase-I evidence

The construction-only preflight passed all eight focused gates. The public Git
commit is not yet bound; these identities become the immutable Phase-I freeze
only after protected merge.

| Field | Phase-I value |
| --- | --- |
| Preflight evidence root | `blake3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e` |
| Trained artifact CID | `blake3:0ed7bf62074857df80045ac3b8bee13ee5f367be4b2b971748631b606ab5985a` |
| Mechanism/core-freeze CID | `blake3:4c7c33d8de40dd6bd7424c9e6360183f672d55453c257a83fa554e045b6b1d1a` |
| Common initialization CID | `blake3:8f91f8d05cbde422593860cffdc3153007fb5b3b2946217ef0015668d3ac34d0` |
| Construction-population kappa | `blake3:446e4f16c9aff5b5dee4c342bf45847e6e8332d6bed8d4a9a21bfc99f82dbe39` |
| Canonical 120-frame manifest CID | `blake3:205ee0d1b9aebbee2475d97de3b95d359ff2ee8220334995cfe4c7a71ead5920` |
| V4 validation inputs / labels / predictions | `NOT_RUN / NOT_RUN / NOT_RUN` |

H4-compatible, alternative-tangent, and fixed-frame plain attention each fit
`16/16`; their Q/K/V/O update counts were identically
`[105,105,105,105]`. Current-only fit `8/16` with
`[1280,1280,1280,1280]` updates. The three main arms had exact decision parity;
their maximum scalar tolerance ratio was `9.53441e-5` and their maximum
gradient/proposed-update tolerance ratio was `6.23339e-6`, both far below the
frozen limit of one.

The frame manifest contains 120 distinct exact scaled-`Z[phi]` roots. All
14,400 ordered pairs passed the tangent-basis mapping, source/destination
projector, transpose-reciprocity, base mapping, full-connection orthogonality,
composition, and H4-left-action checks. The maximum H4-left-action residual was
`1.11022e-16`; the largest reported structural residual was
`1.08247e-15`, below `1e-12`. Because every frame proves `B^T B = I`, the tested
identity-pivot equation algebraically implies composition through every one of
the 120 possible intermediate frames; a redundant 1,728,000-triple loop was
not used as additional evidence.

Central differences passed all 39 active coordinates of the representative
Q/K/V/O event, including accumulated repeated-token contributions. Maximum
absolute gradient residual was `9.02327e-11`, or `3.65739e-4` of its frozen
absolute-plus-relative allowance. Order, value, and gauge-mismatch controls all
executed with the declared work/tangent shape; order changed the causal trace,
value permutation changed value sources while leaving logits byte-identical,
and gauge mismatch changed logits. Opaque suffix mutation left the causal
result unchanged with zero future reads. Two complete compiles reproduced
artifact, core, frame-manifest, construction, and prediction bytes.

Reproduce the only decision-bearing Phase-I suite with:

```bash
cargo test -p uor-r4-core \
  --test connection_gauge_covariance_v4_973 \
  --offline --no-fail-fast -- --nocapture
```

Observed locally on the frozen branch: `8 passed; 0 failed` in 1.74 seconds.
This positive establishes construction-scale representation covariance for the
ordinary softmax attention oracle. It is not the 24-case V4 held-out result and
does not authorize paired E8, resonance replacement, recurrence, lowering, or
generation yet.
