# Corpus-Induced Harmonic Signed Transport Attention Plan

- **Status:** adopted direction; implementation and qualification are owned by
  [#986](https://github.com/UOR-Foundation/uor-r4/issues/986)
- **Programme authority:**
  [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) and
  [Geometric Intelligence Programme](geometric_intelligence_programme.md)
- **Planning reconciliation:**
  [#985](https://github.com/UOR-Foundation/uor-r4/issues/985)
- **Predecessor evidence:**
  [#983 record](construction_causal_return_attention_983.md)

## Decision

The next experiment will not ask exact geometry to create semantics from an
identity address. It will test a four-part architecture:

1. a source-free observation corpus induces semantic placement, value, and a
   seven-slot harmonic link-state record for every lexical route (self plus at
   most six corpus-derived peers);
2. immutable prime/semiprime, fixed-zeta, R4/S3, H4, `H4 + phi H4`, and
   Cl(0,6)/SpiralCore state supplies addressing, causal order, and transport;
3. one candidate-relative signed zero-sum radial/angular contrast decides
   whether transported context changes a natural candidate choice; and
4. a matched table-native semantic-value predictor establishes whether exact
   geometric transport contributes anything beyond the induced corpus value.

This is `CorpusSignedTransportV1`. It is a falsifiable attention candidate,
not an accepted attention mechanism.

The separation is deliberate. #967 established distinct ordered state without
a discriminating readout. #970 rejected one paired-H4 heatmap readout. #969
established one identity-derived local selector on a tiny matched smoke. #953's
same-object placement lost to its placement permutation. #983 then produced
pure construction classes but zero transfer to six independent held-out
decisions. Those results reject particular readouts and placements; they do not
show that exact algebra supplies semantic locality.

## Roles and claim boundaries

| Layer | Adopted role | What would have to be measured before promotion |
| --- | --- | --- |
| Prime, semiprime, n-let, base-256, CID, and hex serialization | immutable identity, address, compact lookup, and transport | never promoted to semantic proximity merely because encodings align |
| Fixed zeta ordinates | pinned spectral coordinates | reuse the checked-in list; do not recalculate zeros or infer the Riemann hypothesis |
| R4/S3, H4, `H4 + phi H4`, Cl(0,6), SpiralCore, and Cayley-Dickson research algebra | ordered causal state, finite operators, parallel transport, and controls | candidate-relative causal uplift over matched non-geometric value |
| Corpus-induced placement | semantic coordinates and table value compiled from observed past data | held-out CID-disjoint transfer against stratified placement and address controls |
| `HarmonicLinkState7` | static self-plus-six local link-state stencil for bounded candidate-rooted diffusion | uplift over link-disabled and distance-only controls; never attention by itself |
| Signed zero-sum contrast | proposed local attention readout | order-sensitive held-out selection that value-only and disabled/permuted controls do not reproduce |
| Table-native predictor | semantic-value comparator and possible fallback engine | independent transfer under the same natural candidates and work |

The old prime-router manifold caches, word-prime transitions, spin states,
angles, and corpus vectors remain useful import candidates and priors. They
stored per-word vectors and transition tables, but the audit found no qualified
per-vector seven-neighbor attention table. The current graph compiler does
already construct deterministic degree-capped lateral co-activation edges, so
that machinery is an implementation seam rather than evidence that the new
mechanism works. The historical `6k+1` hex-hive and hash-prefix modulo
placement are addressing prototypes, not collision-free semantic geometry. Any
reused artifact must be converted into the current codec, CID partition, and
provenance contract; its historical output is not qualification evidence.

The specific link-state hypothesis is traceable rather than reconstructed from
memory alone. The archived
[Semantic Packet Routing Research Note v3](../research/ai-research/ai-router/router-research/Semantic_Packet_Routing_Research_Note_v3.md)
calls for OSPF-like local route-state convergence; the archived
[QIMC extract](../research/archives/gemini-dev/scratch/qimc_extracted.txt)
describes a seven-node phase-synchronized neighbor cluster; and the supplied
SpiralCore v63 HTML implements Jacobi harmonic interpolation over real
dodecahedron edges. None defines causal attention. In current code,
[`induction.rs`](../crates/uor-r4-graph-compiler/src/induction.rs) already emits
deterministic degree-capped co-activation edges, while the archived
[`ppmi_proxy.py`](../research/ai-research/ai-router/router-research/tasks/ppmi_proxy.py)
constructs distance-weighted corpus co-occurrences. These are reusable seams,
not positive evidence for `HarmonicLinkState7`.

## Candidate computation

Construction data alone induces a deterministic directed causal
co-occurrence/PPMI field, the table-native causal candidate value
`b_t(c) = sum_i phi^(-(t-i)) PPMI(x_i -> c)`, and a canonical rank-8 placement
whose unit direction is `v(x)` and whose separately stored construction-derived
`Z[phi]` radial/frequency factor is `rho(x) > 0`. Basis and sign ambiguity are
fixed by canonical route-address order. Hash bits, prime indices, CIDs, and
hexadecimal spelling remain identities rather than semantic coordinates.

Each route then receives exactly one frozen `HarmonicLinkState7` record: its
self anchor plus up to six non-self peers. Directed peer conductance is the
construction-only value `w_xy = max(PPMI(x -> y), 0)`; retain the six strongest
positive peers by `w_xy` descending and canonical route address ascending, and
encode missing peers as explicit null slots. Each row binds the neighbor CID,
conductance, canonical operator/phase identity, and construction provenance.
Thus "seven" means the center plus six bounded peer slots in a local
stencil, not seven arbitrary nearest vectors and not an H4/E8 theorem.

One complete same-frame lexical-route-to-Cl(0,6)/SpiralCore operator mapping
must supply `O(x)`; absence is unavailable rather than permission to synthesize
a hash-derived mapping. Write `p(x)=rho(x)v(x)` and normalize each nonempty
peer row as `P_xy=w_xy/sum_z w_xz`. A zero-peer artifact row is an explicit
self-loop: define `N(x)=N_6(x)` when a positive non-self peer exists and
`N(x)={x}, P_xx=1` otherwise. Gate 0 nevertheless requires a non-self peer for
every selected history and candidate route.

For each admitted candidate `c`, build one bounded screened-harmonic reachability
field over the frozen link-state rows:

```text
H_c^0(x) = 1[x=c]
H_c^(r+1)(x) = (1/2) 1[x=c]
             + (1/2) sum_(y in N(x)) P_xy H_c^r(y),  r=0,...,5.
```

The exact rational restart, six canonical-float Jacobi steps, canonical row
order, and zero-neighbor behavior freeze before labels. Quantization occurs
only at the `H^1` or `H^6` scoring boundary with the frozen quantizer, never
between sweeps. This is the OSPF-like
part of the proposal: every lexical route carries a bounded local link-state
record and candidate-rooted route influence propagates for a fixed work budget.
OSPF itself is shortest-path routing, not harmonic diffusion; the analogy is
link-state knowledge plus bounded propagation. The stored rows and scalar
diffusion are representation/transport, not attention by themselves.

The Jacobi domain `D` is every construction-supported lexical route in the
frozen induction artifact, ordered by canonical route address. One candidate
evaluation performs exactly six synchronous sweeps over all `|D|` rows and all
six padded peer slots: `36|D|` slot updates, including null reads, followed by
the same eight padded history-slot and operator ledger in every arm. Ordered
reductions make parallel scheduling irrelevant to the result.

Over the fixed eight-event causal window:

```text
G_0 = I
G_i = O(x_i) G_(i-1)
```

For each already-admitted candidate `c` and occupied history slot `i`, transport
the observed semantic value into the candidate frame and measure one frozen
radial/angular alignment:

```text
u_i(c) = O(c) (G_t G_i^-1) p(x_i)
utilde_i(c) = H_c^6(x_i) u_i(c)
a_i(c) = <utilde_i(c), v(c)>
       = H_c^6(x_i) ||u_i(c)||_2 cos(theta_i(c))
z_i(c) = a_i(c) - mean_j a_j(c).
```

For `m` occupied slots, set `k=floor(m/2)`. The pre-label balanced rank rule
assigns `+1` to the strongest `k` history
slots, `-1` to the weakest `k`, and `0` to an unpaired middle slot. Cutoff ties
are zeroed with matched opposite ranks until positive and negative counts
match. Therefore `q_i(c) in {-1,0,+1}` and `sum_i q_i(c)=0` exactly. The signed
candidate-relative receptive state and geometric residual are

```text
r(c) = sum_i q_i(c) utilde_i(c)
g(c) = <r(c), v(c)>.
```

The exact zero-sum coefficients are the ternary `q_i(c)` weights over the
link-aware transported values `utilde_i(c)`; `H_c^6` is part of each value, not
an additional attention coefficient.

Center `b_t(c)` and `g(c)` over each natural candidate set. Without labels,
compute `M_b` and `M_g` once from the canonically pooled candidate-centered
`b_t` and unablated-full `g` values in the induction artifact. Freeze the exact
MAD convention and pooled order, require both scales to be finite and nonzero,
then reuse them unchanged: the full arm and every geometric control use
`(M_b,M_g)`, while the table comparator and its controls use `M_b`. No arm
recenters beyond the common per-decision candidate centering, and no control
re-estimates a scale. The full score is the fixed equal-scale
sum of normalized table value and normalized geometric residual; the comparator
uses normalized table value alone. Only a unique maximum whose lead over second
place exceeds its score-family calibration-derived frozen margin selects. Every
tie or low-margin case abstains.

The signed zero-sum weights act over previous, twice-previous, and every other
retained causal route slot separately for each candidate. They are not merely a
centering of final candidate scores. The one-step candidate append is inferred
from observed state; an actual future route or evaluation label is never an
input.

Compiler-side floating point, allocation, and the existing `cd_space`,
endomorphism, Lie-Jordan, exact H4, and SpiralCore modules are permitted during
this research qualification. The legacy hash-derived Cayley-Dickson toy is a
control, not the operator implementation. A successful behavior is lowered to
base-256/integer/LUT execution only later, and the lowered path must reproduce
the frozen causal decisions before it earns a serving claim.

## Frozen evaluation

[#986](https://github.com/UOR-Foundation/uor-r4/issues/986) is the complete
execution contract. Its decision-bearing sequence is:

1. **Population freeze.** Bind one source-free observation-corpus manifest and
   split by document/CID into induction, 16 matched calibration pairs (32
   decisions), and 32 matched sealed-test pairs (64 decisions). Each pair keeps
   candidates, length, multiset, last-two suffix, support, and work fixed while
   a predeclared earlier-order intervention differs. Within each partition, the
   split builder enumerates every structurally eligible pair-CID tuple, applies
   the separately sealed predicate that the two expected routes are incompatible
   and naturally admitted, then sorts lexicographically by
   `(partition CID, candidate-set CID, length, multiset CID, last-two CID,
   min(decision CID), max(decision CID))`. In that order it greedily accepts a
   tuple only when neither decision CID has already been accepted, then takes
   the first 16 or 32 vertex-disjoint pairs. It commits that ordered pair list,
   intervention map, and sealed expected-route join together. Pair IDs and
   intervention metadata exist only in the audit/statistics harness; placement,
   link/operator construction, diffusion, scoring, margin calibration,
   abstention, and selection receive histories and natural candidates only.
   An insufficient eligible count is `UNAVAILABLE_FRAME_OR_POPULATION`; do not
   change the key or take a different sample. Preserve natural schema-2
   admission and exclude complete-history, suffix, ordered-route, operative-key,
   document, and prior-fixture overlap.
2. **Immutable pre-geometry checkpoint.** Before any placement, link-state,
   harmonic, or score diagnostic, post one CID binding the corpus/splits and
   pair commitment, implementation commit, every formula and quantization,
   operator/codec identities, controls, thresholds, randomization blocks, and
   exact work ledger. A later miss does not authorize a re-freeze or retry in
   #986.
3. **Gate 0: semantic placement reachability.** With labels still sealed,
   require complete selected-history/candidate placement, nondegenerate rank
   and score spread, complete `HarmonicLinkState7` construction, and operative
   directed reachability: every decision/candidate pair has at least one
   occupied history with positive quantized `H_c^6(x_i)` and a nonzero
   quantized `H_c^6` spread across its occupied history. Require at least two
   candidates per decision to have different history-field vectors, at least
   one occupied value per decision to differ between `H^1` and `H^6`, and
   nonzero quantized `g(c)` spread across candidates. Also bind exact frame
   identity, anti-recall, equal work, source
   closure, incremental/full agreement, and two byte-identical multithreaded
   builds. Exact PPMI, factorization/degeneracy, basis/sign, radius, peer
   selection/ties/nulls, row normalization, canonical-float restart/iteration,
   final-only `H^1`/`H^6` quantization, operator, MAD,
   ternarization, and calibration formulas and identities also freeze before
   labels. Any miss stops.
4. **Calibration.** Fit exactly two scalar margins under one predeclared
   deterministic rule and equal budget: `delta_F` from the unablated two-
   component full arm, shared unchanged by every geometric control, and
   `delta_T` from the unablated one-component table comparator, shared unchanged
   by its order/recall controls. For each family, the ordered candidate list has
   exactly 33 positions: zero followed by one top-two gap per calibration
   decision in decision-CID order; duplicate values retain their positions. On
   the same 32 decisions, choose maximum selected precision subject to at least
   24/32 coverage, then break ties by more correct decisions and smaller margin.
   If
   no candidate meets the coverage floor, freeze zero and mark that family
   calibration-ineligible for its positive terminal; the other family may
   still be tested. Both families therefore receive the same grid-construction
   rule, 33 threshold positions, objective, coverage floor, tie-break, and one-
   scalar budget without pretending their final-gap units are identical. No
   control calibrates itself. Freeze both margins and the full calibration
   transcript before the sealed-test join. The induction `M_b` and `M_g`
   scales remain unchanged in every arm. Do not change the placement, formula,
   controls, corpus split, or population.
5. **One sealed test.** Compare the full mechanism with separate placement-only
   and link-destination-only permutations, address-only placement/links,
   harmonic-link disablement, a six-Euclidean-nearest peer table, direct-link
   `H^1`, transport-disabled, order-deranged, last-only, state-disabled,
   candidate/operator-mapping-permuted, absolute-only, positive-only, and exact-
   recall/count controls, plus the table-native value comparator. The distance-
   only row selects peers by canonical rank-8 Euclidean distance ascending and
   address ascending, assigns
   `w_xy=1/(1+||v(x)-v(y)||_2)`, and uses the same normalization, null, and
   self-loop rules. For each candidate, every arm performs six synchronous
   canonical sweeps over every artifact route and all six padded peer slots,
   then the same eight history-slot and operator work. Link-disabled,
   direct-link, transport-disabled, last-only, state-disabled, absolute-only,
   and positive-only arms substitute respectively their frozen boundary value
   only after that work; direct-link scores `H^1` after computing the remaining
   five sweeps. The distance-only arm therefore has the same slots and declared
   work, so a positive cannot be credited to nearest-neighbor lookup. Candidates,
   information, and declared work remain equal.

Let `U=(1/64) sum_j 1/|C_j|`. The positive geometric terminal requires the full
arm to be calibration-eligible, cover at least 48/64, exceed `U` by at least
0.15, and beat every named placement, link, address, distance, direct-link,
transport, order, last, state,
operator-mapping, absolute-only, positive-only, and recall/count control by at
least 7/64. Each exact paired randomization test flips whole 32-pair blocks,
never the 64 conditionally dependent decisions, and must give `p<=0.05`. The
full arm must also get both decisions correct in the sealed direction on at least
24/32 predeclared pairs while last-only and state-disabled do not reproduce
both correct decisions on those same pairs, clear the deterministic gates, and
beat the table-native comparator by at least 4/64 under the same exact 32-pair-blocked
randomization test with `p<=0.05`. The table comparator transfers only if it is
calibration-eligible, independently meets the same coverage and `U+0.15` floors,
and beats its order-deranged and recall/count controls by at least 7/64 with
paired `p<=0.05`.
Thresholds do not move after the sealed labels are visible.

## Decision branches

- `PROCEED_TO_I1_WITH_CORPUS_SIGNED_TRANSPORT_ATTENTION`: placement transfers
  and signed exact transport supplies causal uplift beyond the table-native
  comparator. Apply the frozen mechanism unchanged to #953 only after a new
  label-free preflight.
- `RETAIN_GEOMETRY_AS_TRANSPORT_ADVANCE_TABLE_VALUE_QUALIFIER`: the table arm
  independently transfers but the full signed-transport terminal does not.
  Keep exact
  geometry for address and transport, create one table-value qualifier, and do
  not call its result geometric attention.
- `REDESIGN_CORPUS_OBJECTIVE_OR_PLACEMENT`: a valid sealed run qualifies neither
  the full arm nor the table comparator. Stop before #953 and redesign the
  induction objective or placement.
- `UNAVAILABLE_FRAME_OR_POPULATION` or `INVALID_CONTRACT`: a population,
  frame, work, provenance, or contamination boundary failed. Record and close
  the frozen result; any repair requires a freshly frozen successor rather than
  a re-freeze or another representation in #986.

Precedence is exhaustive: contamination is `INVALID_CONTRACT`; otherwise a
failed pre-sealed gate is `UNAVAILABLE_FRAME_OR_POPULATION`; otherwise a
positive full arm advances; otherwise a transferred table arm takes the table-
value branch; otherwise redesign the corpus objective/placement.

## Now / Next / Later

### Now

- Land #985's programme and issue-graph reconciliation.
- Preserve #983 as a closed bounded negative.
- Execute only #986 on the fresh corpus split. Keep #953 parked, unassigned,
  untouched, and blocked.

### Next

- A positive signed-transport terminal goes unchanged to the bounded #953
  decoder after a fresh label-free preflight.
- A table-preferred terminal creates exactly one table-native value qualifier
  before #953.
- Only accepted #953 generation exposes #973 paragraph, conversation, global,
  and later corpus-scale qualification; accepted #973 then exposes #954
  correctness and abstention.

### Later

- #955 qualifies multi-step reasoning.
- #962 integrates product chat and identity-scoped memory.
- #963 measures optimization and performs any base-256/integer/LUT lowering.
- #964 formalizes the accepted serving contract.
- #965 performs release qualification.

No dates are invented. Only an actively executing issue is assigned. Runs use
all available local hardware through content-addressed partitions and ordered
reductions; a long experiment is never launched as a single-worker job.

## Related work and novelty boundary

The literature review found close ingredients, not the completed UOR design:

- [ZeroS](https://arxiv.org/abs/2602.05230) motivates signed zero-sum contrast.
- [From Self-Attention to Connection Laplacian](https://arxiv.org/abs/2607.10677)
  formalizes attention as weighted connection transport.
- [SPAGAN](https://arxiv.org/abs/2101.03464) demonstrates path-based attention
  over shortest-path graph neighborhoods; it supports the routing analogy but
  remains a learned graph-neural architecture.
- [RiemannFormer](https://arxiv.org/abs/2506.07405) and
  [Riemannian Attention Mechanisms](https://arxiv.org/abs/2608.01283) study
  curved metrics and parallel transport in transformer architectures.
- [LieTransformer](https://arxiv.org/abs/2012.10885) studies equivariant
  attention on Lie groups.
- [RetNet](https://arxiv.org/abs/2307.08621) and
  [Mamba](https://arxiv.org/abs/2312.00752) are evidence that ordered recurrent
  causal state can replace quadratic attention in other architectures.
- [HopfE](https://arxiv.org/abs/2108.05774) and
  [NagE](https://arxiv.org/abs/2005.10956) use quaternionic/noncommutative
  operators for relational representation.
- [IntAttention](https://arxiv.org/abs/2511.21513) is relevant to later
  integer/LUT execution.

Each work retains learned transformer, recurrent, embedding, or knowledge-
graph machinery that differs materially from UOR-R4. No reviewed paper
established the exact combination of immutable prime-route identity,
corpus-induced placement, exact Cl(0,6)/SpiralCore causal transport, ternary
zero-sum candidate contrast, and base-256 lowering. That absence is not proof
of novelty. **Novelty and attention capability remain `NOT_ESTABLISHED`.**

## Nonclaims

This plan does not establish semantic placement, attention, coherent
generation, correctness, reasoning, performance, integer/multiply-free
serving, an E8 identity theorem, a Riemann-hypothesis result, novelty,
patentability, product chat, formal closure, or release readiness. Those claims
remain separately gated in the native programme order.
