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

## Phase-II target-free freeze — 2026-08-29

Status: `PHASE_II_INPUT_FREEZE_PASS_PENDING_PROTECTED_MERGE`. Phase I was
protected by PR #1000 at
`b054197acb92e3dd23d88d81bd859379ea8fac67`. The first append-only Phase-II
commit, `c9c3b13ca7c1d346b5c4b3a5d624907b7f765461`, published only the
executable generator checkpoint. An independent review reproduced its NUL
domain tags, little-endian length/token encodings, complete 65,536-counter
order, tie rules, first-12 scan, mate involution, dynamic antichain, and exact
16/8/12 legacy forbidden rows. It imports no V4 model type and cannot reach a
compile or prediction path. The protected seed was executed at that checkpoint;
only policy-declared aggregate invariants were observed, not the selected
population bytes.

The second Phase-II commit freezes the generated target-free manifest. The
selected order is the accepted-pair order and, within each pair, unswapped then
mate. Its accepted counters are:

```text
30149, 53145, 27994, 21913, 21781, 64005,
53150, 46433, 44855, 31599, 62881, 20555
```

All 24 prefixes are unique 13-token rows, end in query token `1`, contain one
earlier query binding, and form an antichain with every other selected row and
all 36 construction/V2/V3 rows. Every matched pair has one unchanged token
multiset under the `5 <-> 6` involution. The aggregate structural balance is
`12/12`; per-case labels and the 32-byte nonce are not published. Prefixes and
case IDs are target-free fields, although the synthetic binding rule makes the
target logically inferable. This is therefore public non-adaptation evidence,
not a claim of blind evaluation.

| Field | Phase-II value |
| --- | --- |
| Generator policy CID | `blake3:73b4233b0b91ba85ffb6cd8c3d86132a954e4fbda5c7ec57510cc30bd9fb5dca` |
| Base forbidden-prefix root | `blake3:877707eff60857b9c790cfb0e8a2a5a12bbcadb51d3448c9bd7119d5b86b6c42` |
| Ordered validation-prefix root | `blake3:a3321b13d808d553d7588997f8fb7951be33e254724d45a1223460dd775a3ad8` |
| Complete validation-input CID | `blake3:5a17c5526d866f2862b042750cb70f5183f6a8fc09ab53a067d79d28d1c989d1` |
| Salted label commitment | `blake3:9773355914ed171f0d14950a4db554f5f543252804c703e8e0bbbbf17fe7b602` |
| Pre-label freeze CID | `blake3:170419cfcf80b2b0e48cc74faff13c9791dd9106045a1ff59a82efe4f6b205aa` |
| Pair / case / aggregate balance | `12 / 24 / 12:12` |
| Validation predictions / scoring-label joins | `0 / 0` |
| Labels / nonce / scores / verdict | `SEALED / SEALED / NOT_RUN / NOT_RUN` |

The commitment steward handled the structurally determined label preimage once
only to calculate the integrity commitment. No scoring/evaluation join occurred.
The nonce is held outside the repository and is not present in source, GitHub,
test output, or CI. The pre-label root binds the Phase-I identities, both legacy
input identities, forbidden/prefix/input roots, the salted commitment, counts,
zero-call declarations, and the literal status
`inputs=FROZEN;labels=SEALED;predictions=NOT_RUN;scores=NOT_RUN`. The protected
Phase-II commit additionally binds the frozen comparator mapping: order,
value, and source-gauge-mismatch controls use H4-compatible attention;
current-only uses its separately trained current-only arm.

Reproduce the only Phase-II decision-bearing test with:

```bash
cargo test -p uor-r4-core \
  --test connection_gauge_covariance_v4_target_free_973 \
  --offline -- --nocapture
```

Observed before commit: `1 passed; 0 failed` in 0.49 seconds. The next and only
authorized action is protected merge of this exact freeze, followed from that
merge by Phase III: reproduce both freezes, emit two byte-identical label-free
prediction streams, open the nonce/preimage once, verify the commitment, and
score once. Any mechanism, population, frame, control, threshold, or encoding
repair after this point is V5 with a new population, never a V4 rerun.

## Phase-III one-time reveal — 2026-08-29

Phase II was protected by PR #1001 at
`a567edd43ec4840c0bce495339c6416777c4c883`. From that exact merge, both
Phase-I and Phase-II suites reproduced their frozen identities. An inert
label-free runner then compiled the unchanged 16-document model twice and
produced two byte-identical 168-record streams over the exact 24 cases and
seven comparator bindings. Every trace passed the frozen score-gap,
causal-work, finiteness, tangency, intervention-liveness, and main-arm
numerical/decision-parity checks. The stream CID was published to #973 before
the label nonce was opened:

```text
blake3:037f800b57c5f3d514629799be4a16709d81b1d79de1a3ff64009ace8b239a56
```

The selected-token masks in frozen case order were:

```text
H4 baseline           566655556655665566555566
alternative tangent   566655556655665566555566
plain fixed           566655556655665566555566
current only          666666666666666666666666
order shuffled        656656665556665566566555
value permuted        556666666666556655666655
source-gauge mismatch 655556555566555555655555
```

After publication, the nonce was opened once. The exact 24-row preimage had no
missing, duplicate, or extra case IDs; every target was in `[5,6]`, every row
matched the declared predecessor-binding rule, and the balance was `12/12`.
Re-encoding it reproduced the Phase-II commitment exactly. Scoring then ran
once by raw case ID, not by implicit row position.

| Arm/control | Correct | Frozen gate | Result |
| --- | ---: | ---: | --- |
| H4-compatible | 13/24 | at least 18 | FAIL |
| Alternative tangent | 13/24 | at least 18 | FAIL |
| Plain fixed | 13/24 | at least 18 | FAIL |
| Current only | 12/24 | at most 12 | PASS |
| Order shuffled | 13/24; drop 0 | drop at least 6 | FAIL |
| Value permuted | 12/24; drop 1 | drop at least 6 | FAIL |
| Source-gauge mismatch | 11/24; drop 2 | drop at least 6 | FAIL |

| Evidence identity | CID |
| --- | --- |
| Revealed label preimage | `blake3:ef0c916d1aadb7c6093139d8dc8e8b653f35bf917b3293bb3085140f261b76d6` |
| Salted label commitment | `blake3:9773355914ed171f0d14950a4db554f5f543252804c703e8e0bbbbf17fe7b602` |
| Canonical Phase-III result | `blake3:35fa3d708c799439c1ef5c5f8aa1fc837f8aafcfca087d7088ecb6d0260ffd1a` |

The terminal is
`FAIL_CONNECTION_GAUGE_COVARIANCE_V4_HELD_OUT_FUNCTIONAL_PARITY_STOP_BEFORE_PAIRED_H4_E8`.
The complete machine-readable result and revealed commitment preimage are
[`connection_gauge_covariance_v4_result_973.json`](connection_gauge_covariance_v4_result_973.json)
and
[`connection_gauge_covariance_v4_label_preimage_973.json`](connection_gauge_covariance_v4_label_preimage_973.json).

### Interpretation and attention-only successor

V4 established that the same learned local coefficients can express the
ordinary softmax operator covariantly in H4-compatible, alternative-tangent,
and fixed gauges. It did not establish that this one-layer token-local operator
learned the held-out relation. The three main arms emitted the same mask at
`13/24`, while destroying order, values, or source-gauge coherence removed at
most two correct cases. The predictive effect is therefore not load-bearing
attention on this population.

The fixture asks for the token following an earlier query. V4 gives a position
a key and value derived from that position's token. It has no explicit causal
transition row whose key is the predecessor and whose value is the current
token. A contextual Transformer can construct that relation in an earlier
layer; this bounded one-head scaffold cannot assume it already exists.

The sole active successor is therefore a fresh causal transition-binding
softmax oracle, not E8, resonance, recurrence, scale, lowering, or a V4 retune.
For each observed transition `x_(i-1) -> x_i`, V5 must store the
predecessor-derived key and current-token value in one transported row; the
final query reads those rows. It must compare separately trained
H4-compatible, alternative-tangent, and plain arms against V4 token-local,
predecessor-disabled/permuted, value-permuted, order-shuffled,
source-gauge-mismatch, and current-only controls on a new independently frozen
population. Ordinary held-out softmax variable binding must qualify before any
other project mechanism advances.

## Successor-direction update — 2026-08-29

The V5 transition-binding paragraph above records the immediate diagnosis made
at the V4 terminal. No V5 mechanism, population, prediction, or result was
implemented or frozen. It is superseded for forward work by `HELM-D-R4`; every
V4 byte, count, CID, terminal, and nonclaim remains unchanged.

`HELM-D-R4` pins the official MIT HELM-D source at
`7501deca8f413848bfef804be64ce874b72a3cd7`, audits the source-faithful
architecture, freezes an ordinary decoder donor, and then preserves learned
Q/K/V, stable causal softmax, value
aggregation, and `W_o` while splitting heads into R4 blocks, encoding exact
cumulative Spin/H4 local frames, transporting K/V into the query frame, and
mapping the aggregate back before `W_o`. Its first gate is numerical and
behavioral parity on separately frozen real causal-language next-token loss,
top-1, and decoded output against the donor plus an equal-work source-frame-
permuted control. Parity is not a
geometry-advantage result. Only after it passes may #973 train an intrinsic R4
distance/centroid operator; only a positive intrinsic result may authorize
multi-resonance replacement and recurrence. Source weights, softmax, and dense
all-prefix work remain offline oracles and establish no transformerless or
serving claim.

## HELM-D-R4 delivery update — 2026-08-29

The successor has now reached the bounded
`PASS_HELM_D_R4_GAUGE_SOFTMAX_FULL_DECODER_PARITY_ADVANCE_TO_INTRINSIC_R4`
terminal. This changes no V4 byte, score, CID, threshold, or interpretation.
See the append-only
[`HELM-D-R4 result`](helm_d_r4_softmax_decoder_973.md). Intrinsic R4 attention,
multi-resonance replacement, and recurrence/lowering remain `NOT_RUN`.
