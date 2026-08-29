# #973 corpus-induced document spin-placement contract

- **Issue:** #973
- **Mechanism:** `CorpusInducedDocumentSpinPlacementR4V1`
- **Status:** frozen before implementation and before held-out target attachment
- **Scope:** document only
- **Conversation scope:** `NOT_RUN`
- **Task scope:** `NOT_RUN`
- **Frozen corpus:** `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`
- **D3 partition:** 2,404 construction documents / 596 held-out documents
- **Base table:** `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`
- **Accepted #953 overlay:** `blake3:914126a311c3984d1482258a8f0a7fa2e34896540d502d19f1d9076fbd4a9b76`

This record freezes the next #973 decision before the mechanism is implemented
and before any held-out next-route value is attached to its structural census.
The accepted #953 admission policy, candidate support, and fallback remain
unchanged. This rung tests whether construction-induced placement in an exact
ordered spin state can choose among that fixed support on natural held-out
document prefixes. It does not test conversation or task scope and does not
claim semantics, correctness, reasoning, broad coherence, performance or
energy advantage, or product readiness.

## Frozen representation and identity boundary

**Definition.** The exact product state is

```text
S = 2I x Z_M x Z_M, M = 3,373,259,426
(h,f,t) o (h',f',t') =
    (h h', wrap_M(f + f'), wrap_M(t + t'))
```

where `2I` is the canonical 120-root H4 binary-icosahedral table already used
by the bounded-global V2 mechanism and `wrap_M` returns the canonical interval
`[-1,686,629,713, 1,686,629,713)`.

**Definition.** `SFTBL001` token ID `r` receives immutable identity leaf
`L_r`. `BOS` is the identity/reset. For every other fitted lexical, byte
fallback, special, or EOS token, let `p_r` be the zero-based `r`th prime and
set:

```text
h_r = the V2 exact root row selected by r mod 4
f_r = wrap_M(p_r * 1,000,003 + r * 17,071)
t_r = wrap_M(-p_r * 97,409 + r * 7,919)
L_r = (h_r, f_r, t_r)
```

The four-row H4 map, prime enumeration, constants, modulus, table kappas, and
source-free table CID are artifact-bound. These leaves are route identity
coordinates only. Their prime/rank provenance is recorded explicitly and is
not interpreted as learned meaning.

**Definition.** The learned object is a separate versioned placement overlay.
It never mutates a token ID, payload, base-table row, #953 radius row, or the
immutable leaf. Every stored prototype row contains the source-free token,
exact decoded-payload CID, exact product state, distinct construction-document
support, distinct state support, and binding kappas. Token-to-payload decoding
is its inverse/provenance witness. Support and provenance cannot enter the
score.

## Construction-only induction rule

**Definition.** In each D3 construction document, at a position where the
unchanged #953 path exposes a trigram/bigram maximum-count tie `A` and the
observed construction next route is `c in A`, form the natural document-prefix
state

```text
G(d,i) = L(x_0) o L(x_1) o ... o L(x_(i-1)).
```

Construction prefix-to-observed-next-route pairs are the only label-bearing
input permitted during compilation. Validation/held-out continuations,
teachers, providers, source weights, and runtime future routes are forbidden.

**Definition.** Retain one exemplar per `(candidate token, construction
document)`: the earliest eligible stream position in that document. This
natural cap is at most 2,404 exemplars per candidate and prevents repeated
mentions within one article from dominating. A prototype is usable only with
at least two distinct construction documents and two distinct exact prefix
states.

**Definition.** For each usable candidate `c`, compile exact componentwise
product-space Frechet prototype `C_c = (h_c,f_c,t_c)`:

```text
h_c = argmin over all 120 H4 roots h of
      sum_g shell_rank(h^-1 g_h)
f_c = argmin over observed exemplar phases z of
      sum_g circular_abs_q29(z - g_f)
t_c = argmin over observed exemplar phases z of
      sum_g circular_abs_q29(z - g_t)
```

Ties use canonical H4 table order and then ascending signed Q29. Only integer
table operations and integer phase arithmetic define the artifact. The
compiler may parallelize independent documents and candidates, but reduction
and serialization are in canonical document/token order.

**Guarantee sought.** Compile/serialize/reload must be byte-identical, and the
runtime artifact contains only aggregate prototypes and provenance. Runtime
prediction performs no corpus scan and loads no prefix row or source tensor.

## Frozen query rule and controls

**Definition.** #953 first admits its unchanged canonical maximum-count tie
`A`. For observed document-prefix state `G` and every `c in A` with a usable
prototype:

```text
Delta_c = C_c^-1 o G
K_c = (H4S3AngularShell(Delta_c.h),
       circular_abs_q29(Delta_c.f),
       circular_abs_q29(Delta_c.t)).
```

The unique lexicographic minimum wins. A missing prototype or equal minimum
abstains to the exact #953 choice. Candidate token, payload, prime, rank,
digest, support count, and provenance are lookup keys or audit fields, never
numeric score inputs.

**Definition.** Four arms share the same #953 row, support, prototype reads,
exact inverses/products, cost reads, comparisons, decode, and declared-work
ledger:

1. `real`: natural left-ordered prefix fold and induced prototypes;
2. `scope_disabled`: compute the complete real decision, then mask it to #953;
3. `order_shuffled`: use the reverse-prefix product maintained as
   `R_i = L(x_i) o R_(i-1)`;
4. `operator_permuted`: cyclically rotate prototype assignments across
   canonical `A`, with no fixed point when `|A| > 1`.

The controls may change only the named factor. Admission, support, payloads,
base evidence, ceilings, and work remain matched.

## Operative anti-recall slice

**Definition.** After fitting prototypes, construction is replayed once to
build an audit-only set of exact document-prefix state and operative decision
signatures

```text
Omega = (backoff order, A, G, [(c, Delta_c, K_c) for c in A]).
```

Construction prefix/state/signature indexes are qualification evidence and
are not stored in the runtime artifact.

**Definition.** Before any held-out target is read, a held-out position enters
the operative census only when:

- its active #953 structural row occurs in at least two held-out documents;
- every candidate has a usable construction prototype;
- its exact full-prefix CID, exact natural state `G`, and exact `Omega` are all
  absent from construction;
- natural and reverse states differ;
- prototype permutation changes the operative cost vector; and
- all four arms complete with zero support/work mismatch.

Full-prefix disjointness is reported but is not sufficient by itself. The
first canonical non-EOS operative prefix for which real differs from
scope-disabled, order-shuffled, and operator-permuted is frozen for the decoded
witness before its next route is attached.

## Cheap preflight and run contract

**Empirical Criterion.** The target-free preflight must reproduce all frozen
corpus/base identities and the unchanged 76,641 #953 held-out admission
opportunities. It must then report:

- byte-identical operator compile/reload;
- at least 1,024 prototype-complete operative anti-recall positions;
- at least one frozen non-EOS all-control contrast;
- nondegenerate natural/reverse state and real/permuted cost vectors;
- zero support/work mismatches; and
- zero held-out-target, compiler-future, teacher, provider, source-weight, or
  runtime-corpus reads.

An empty/undersized operative population or absent decoded witness stops the
run before target attachment with
`UNAVAILABLE_OPERATIVE_ANTI_RECALL_OR_REACHABILITY`. Any binding, replay,
leakage, support, or work failure stops with
`INVALID_CORPUS_INDUCED_SPIN_PLACEMENT_CONTRACT`.

```text
metric to move:       correct next routes on the frozen operative anti-recall
                      slice; fixed comparator is accepted #953 at
                      103,604/446,342 overall known-target positions
reachability ceiling: 76,641/446,342 = 17.170914 percentage points can be
                      touched because admission is frozen
instrument + verdict: target-free corpus census above; >=1,024 operative
                      positions and one frozen decoded contrast are required
exit rule:            after one target join, real must beat each of #953,
                      order-shuffled, and operator-permuted with directional
                      paired wins greater than losses and a predeclared
                      one-sided exact sign-test p <= 0.05 for each comparator
if positive:          retain document-scope corpus-induced spin attention and
                      run #973's final bounded requalification
if negative:          retain the bounded-global V2 result, reject this
                      placement rule, keep #954 blocked, and redesign placement
cost estimate:        about 20-30 minutes on this M1 after compilation; document
                      scans and prototype fits run in parallel with ordered
                      deterministic reduction
```

**Empirical Criterion.** A positive result additionally requires zero contract
failures, byte-reproducible artifact/report bytes, and the frozen prefix to
produce a distinct exact decoded continuation under the real geometry. The
positive terminal is
`RETAIN_CORPUS_INDUCED_DOCUMENT_SPIN_ATTENTION_CONTINUE_FINAL_973_REQUALIFICATION`.
A valid run without the declared uplift/control separation terminates
`RETAIN_BOUNDED_GLOBAL_ONLY_REDESIGN_CORPUS_SPIN_PLACEMENT`.

No threshold, basis row, phase constant, scope rule, prototype rule, candidate
population, control, witness-selection rule, or decision threshold may change
after this freeze. A placement epoch or structural operator revision receives
new kappas and reruns its owning bounded gate before downstream evidence is
read.

## Pre-implementation target-blind accounting correction

Append-only correction, posted before any held-out target attachment: the
`76,641` count above is the accepted #953 count after the next route has been
classified as a construction-fitted known target. Requiring that count in the
target-free preflight would require reading `stream[target_index]` and would
violate the same preflight's zero-target-read rule.

The target-free preflight therefore reproduces **81,177 reachable held-out
prefixes without reading their next routes**. After the one authorized target
join, evaluation must reproduce **76,641 reachable known-target positions**.
The overall known-target reachability ceiling remains
`76,641/446,342 = 17.170914` percentage points. The 1,024-position operative
threshold, mechanism, population, witness rule, controls, positive criterion,
and terminals are unchanged. No held-out next-route identity was inspected to
make this correction.

The same target-blind binding pass freezes the exact D3 sets before target
attachment:

```text
construction-set kappa:
  blake3:af2a2d7d49db55279e7ea40947a3259ac0a100aa56e8d920951e7c27eaf6df5c
held-out-set kappa:
  blake3:7a7558e96aa86aa2d8965972b69ddce02222c6eccc8ca560df2141fc0ac4170e
```

Each kappa hashes canonical document IDs plus complete text CIDs under its
declared construction/held-out domain. This closes alternate-D3-subset
substitution while revealing no next-route value.

## Pre-target document-blocked statistical correction

Append-only correction, posted while held-out next routes remain unattached:
operative token positions within one article are correlated and may repeat the
same structural row. A position-level sign-test p-value alone would therefore
overstate independent natural-document evidence.

For each comparator, the original position counts and direction remain
required and reported. In addition, each of the 596 held-out documents receives
at most one paired vote: document win when the real arm has more correct
operative known-target positions than the comparator in that document,
document loss when it has fewer, and tie otherwise. A positive terminal now
also requires document wins greater than document losses and the one-sided
exact paired sign test over those document votes to satisfy `p <= 0.05` for
scope-disabled, order-shuffled, and operator-permuted separately.

This strengthens rather than relaxes the frozen positive rule. The mechanism,
placement, operative population, target-free threshold, witness, controls, and
failure terminals are unchanged. No held-out next-route identity was inspected
to make this correction.

## Observed document-scope result — 2026-08-28

**Empirical result.** The frozen run completed at
`RETAIN_BOUNDED_GLOBAL_ONLY_REDESIGN_CORPUS_SPIN_PLACEMENT`. The target-free
preflight passed, the exact spin operator was causally active, and its frozen
decoded witness diverged across every control. The held-out target join then
falsified the promotion criterion: componentwise construction Frechet
placement was less accurate than the unchanged #953 fallback and did not beat
either geometry control.

The compiled operator retained these construction-only statistics:

| Quantity | Count |
| --- | ---: |
| construction documents | 2,404 |
| eligible construction positions | 300,483 |
| one-per-document exemplars | 197,717 |
| observed candidate tokens | 43,047 |
| usable prototypes | 15,320 |
| rejected for single-document support | 27,726 |
| rejected for single-state support | 1 |

The target-free qualification traversed 596 held-out documents without reading
their next routes. It indexed 1,934,246 construction full prefixes, 1,934,246
construction natural states, and 287,517 construction operative signatures.
Against those indexes it observed 81,177 admitted held-out prefixes, of which
62,603 had complete prototype support. Overlapping disqualification predicates
counted 83 exact construction-prefix hits, 83 construction-state hits, 29
construction-signature hits, 10,223 natural/reverse equal states, and 18,574
permutation-inert positions. Their conjunction left 36,533 operative positions,
exceeding the frozen 1,024 minimum. Rows contributing to the census occurred in
multiple held-out documents on 53,772 positions. Support mismatches, work
mismatches, invalid score-firewall certificates, and every forbidden-read
counter were zero.

The one authorized target-join traversal reproduced all 81,177 structural
admissions and 76,641 construction-fitted known-target admissions. It observed
617,710 next-route values: this counter is one read for each evaluated document
stream position in that single post-gate traversal, not 617,710 separate joins.
None of those values entered candidate scoring, prototype construction, the
target-free census, or witness selection. Of the 36,533 operative positions,
35,028 had a construction-fitted target:

| Arm | Correct | Accuracy on 35,028 | Difference from real |
| --- | ---: | ---: | ---: |
| real componentwise Frechet placement | 2,931 | 8.367592% | — |
| scope-disabled / unchanged #953 | 4,281 | 12.221651% | real -3.854060 pp |
| order-shuffled query state | 2,934 | 8.376156% | real -0.008565 pp |
| operator-permuted prototypes | 2,966 | 8.467512% | real -0.099920 pp |

The predeclared exact paired tests all failed in the required direction:

| Comparator | Position wins/losses/ties | Document wins/losses/ties | Pass |
| --- | ---: | ---: | :---: |
| scope-disabled | 1,118 / 2,468 / 31,442 | 71 / 364 / 161 | no |
| order-shuffled | 1,734 / 1,737 / 31,557 | 199 / 212 / 185 | no |
| operator-permuted | 2,931 / 2,966 / 29,131 | 221 / 228 / 147 | no |

Every table entry is an exact integer count. Each one-sided exact sign-test
predicate `20 * sum <= 2^n` evaluated false; no floating-point p-value or
post-run threshold decided the terminal.

The target-free witness was document `4546`, target index `9560`. The real arm
selected token `106399` (`I`), while scope-disabled, order-shuffled, and
operator-permuted each selected token `109525` (`The`). Their complete bounded
64-unit continuations were pairwise distinct from the real continuation, so
the frozen continuation-contrast predicate passed. This is evidence that the
exact geometric operator can causally change decoded behavior. It is not
evidence that the change is useful attention: the held-out accuracy and every
matched paired comparison reject that claim for this placement.

Canonical evidence identities are:

```text
operator:                  blake3:6aa7edc027e6d26c2d6f924edbe55b835720bf0fa3e0a110a367a79b73b3d344
anti-recall index:         blake3:4d662e9c2e8f63228cd6d95bf04a25e0cd357350534e2efb339e68f4cddf258c
target-free census:        blake3:e511415747c7d8ddec2723ee97ea8b32cd38dad6fd90511184ae80e2d0d79d10
evaluation report:         blake3:aebd4edb5ca2d5469c62615cb7f712c71953fa0d09d207686a247bcac460ec51
witness continuation:      blake3:93cc3273990739639fa1fa699777868e396d3401342b20e175f528d48ac6de54
target-free report sha256: cfa6c82cf0ad8918f27530a32c974259f10bd230d555a915cf1b737398ef6375
evaluation report sha256:  e070550fb55b8c2ad727c365b8637aa3277a28fa4f521b375ec01452b4a40879
canonical run sha256:      b681935a1dcabc863ab4b2b4857a6b6ce944e203eb06127121dbdcfcc4823844
```

The first profile-only attempt was stopped before a target-free report was
written and before any held-out target was attached after it exposed serial
document scans. Frozen table/operator inputs were retained. Construction,
anti-recall, and target-free document work were then partitioned over at most
eight native workers with canonical ordered reduction; WASM remains serial.
Forced one-worker and eight-worker tests reproduced byte-identical anti-recall
indexes, census reports, and CIDs. The completed decision run took 281.741
seconds from bound input load through durable terminal. The target-free census
milestone was reached at 56.653 seconds, its report was durable at 56.756
seconds, and the serial label-bearing traversal plus final serialization
occupied the remaining 224.985 seconds.

**Decision and next action.** Retain the bounded-global V2 result, reject
`CorpusInducedDocumentSpinPlacementR4V1` as the corpus placement rule, keep
#973 open, and keep #954 blocked. More documents, table density, or execution
time are not authorized as the next action: the real arm already acts on a
large operative population, yet did not establish superiority over the
order/permutation controls and was materially worse than scope-disabled
fallback.
The next #973 construction must therefore freeze a new candidate-relative,
construction-only discriminative placement objective in exact R4 state—one
that is trained to separate competing admitted candidates rather than estimate
each candidate's componentwise marginal center. It must preserve immutable
route/payload identity, the #953 admission/support boundary, exact least-cost
routing, target-free anti-recall admission, matched controls, and a single
held-out join. No final #973 requalification or #954 work is authorized until
such a replacement independently passes its own frozen gate.

Conversation scope and task scope remain `NOT_RUN` for this corpus-induced
operator. General semantic attention, coherence, correctness, reasoning,
performance or energy advantage, and product readiness remain unestablished.

## Forward disposition after architecture review (2026-08-28)

The result and terminal above are unchanged. The phrase "discriminative
placement objective" is now made concrete by
[ADR-0005](adr/0005-predictive-geometric-connection-memory.md), which
supersedes this record only for forward work.

The then-immediate action was `PredictiveConnectionRetentionGate0V1`, not
another componentwise center and not the full recurrent cell. It tested whether current,
previous, ordered last-two, and complete-prefix exact-route relations support a
construction-transferred candidate-discriminative integer readout under frozen
#953 support/work. A deterministic fit/construction-validation split runs
before the protected D3 held-out targets are opened. Only a Gate-0 positive may
authorize full key/value/query placement, connection-transported multiscale
retention, gated delta writes, exact lowering, or final #973 requalification.

This disposition preserves every #997 byte, count, metric, and nonclaim. It
narrows the next experiment because the failed marginal-center objective was
already operative at sufficient population scale.

## Current sequencing update (2026-08-29)

The gated-delta smoke and direct-attention V3 experiment are now complete. The
gated-delta cell did not beat its matched plain recurrent control. Direct V3
then established the dense fixed-tangent Q/K/V/O path (`12/12`) but rejected
the tested mixed-gauge H4 projection/connection/optimizer combination (`3/12`).
`ConnectionGaugeCovarianceV4` Phase I subsequently passed with explicit local
coordinates and separately trained H4, alternative tangent, and fixed-tangent
arms. Its target-free held-out freeze and salted commitment are sealed in PR
#1001; protected merge/reveal is the active #973 action.
Paired-E8, corpus, resonance-sieve, recurrent-factorization, and
exact-lowering work remain conditional on that result. This update changes no
#997 evidence or terminal above.

## Successor direction (2026-08-29)

V4 subsequently completed terminal-negative at `13/24`, without adequate
separation from its destructive controls. V4 will not be rerun or retuned. The
HELM-D-R4 became the full-decoder, gauge-equivalent ordinary-causal-softmax
reference with R4/Spin frame transport. Its parity gate now passes; the verdict
and scope are authoritative only in the
[HELM-D-R4 result](helm_d_r4_softmax_decoder_result_973.json). The active #973
successor is intrinsic R4 distance and normalized-centroid attention, followed
conditionally by multi-resonance replacement and recurrent lowering.

## Attempt 02 successor update — 2026-08-29

This record's bounded evidence and the `HELM-D-R4` gauge-equivalent ordinary-
softmax PASS remain unchanged. The separately trained intrinsic Lorentz-R4
successor stopped before D3 at
`UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` (result CID
`blake3:da2a63323d6211b8d581e5a4ed75d788eb919ff0f210d2e3beb8a749ee1bc64f`):
normalized-barycenter covariance was `9.1214e-8` against the frozen `1e-8`
limit, and construction-validation NLL was diagnostically worse than the donor
by `1.2531` and the matched flat control by `0.20893` nats/token. No reveal
marker or held-out result exists. No Attempt 03 is authorized under this freeze;
any further intrinsic work must be a newly frozen, source-faithful
learned-manifold successor. Multi-resonance, recurrence, lowering, scale, and
#954 remain blocked. See the
[owning intrinsic record](intrinsic_lorentz_r4_attention_973.md) and the
[compact result summary](intrinsic_lorentz_r4_attention_attempt_02_summary_973.json).
