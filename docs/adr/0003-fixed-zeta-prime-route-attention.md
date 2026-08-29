# ADR-0003: Use fixed-zeta prime routes for geometric attention

- **Status:** Retained as a source-free storage/recall substrate; geometric
  attention and product behavior are not promoted
  (`RETAIN_STORAGE_RECALL_ONLY`; see the
  [qualification record](../prime_route_attention_qualification_958.md))
- **Date:** 2026-08-26
- **Current attention-stage result:** this ADR's route identities and state
  remain substrate. #969 retained one bounded causal local path mechanism;
  #973 retained bounded synthetic higher-scope witnesses; PR #997 rejected a
  natural componentwise-Frechet placement. Forward predictive semantics now
  live in [ADR-0005](0005-predictive-geometric-connection-memory.md). Exact
  prime/zeta/R4/spin coordinates supply address, frame, state, and transport;
  they are not presumed semantic before the ADR-0005 controls qualify them.
- **Decision owner:** representation redesign
  [#958](https://github.com/UOR-Foundation/uor-r4/issues/958)
- **Programme tracker:**
  [#820](https://github.com/UOR-Foundation/uor-r4/issues/820)
- **Supersedes:** the learned query/key/value replacement selected by
  [ADR-0002](0002-geometric-causal-decoder.md), after the negative #951
  qualification. ADR-0002 remains the historical record for G0 and G1.

## Context

G0 established a reachable layer-29 seam and a coherent source-model control.
G1 then established that the learned four-coordinate mixer could move bounded
support and persistent-memory probability, but its operator-alignment and
sampled-token terms remained effectively flat. The predeclared verdict was
`REDESIGN_REPRESENTATION`, not all-layer promotion.

The failed representation also departed from the original router's intended
mechanism. The original mechanism used a fixed grid of non-trivial zeta-zero
ordinates, prime-mapped data, factor overlap, R4/Hopf state, and torsion/phase
transport. Geometry supplied storage, recall, and steering; bigram/trigram
tables or an external Ollama model still supplied fluent continuation. It did
not establish standalone attention or reasoning.

The surviving Prime Router implementation contains two important analytical
pieces:

1. a windowed Chebyshev residual `psi(x) - x`, where the distributional
   derivative of `psi` is the von Mangoldt signal; and
2. projection onto fixed log-polar phases `exp(i * gamma_j * log(x))`.

Under the Riemann-hypothesis assumption, a critical zero has
`rho_j = 1/2 + i*gamma_j`, so

```text
x^rho_j = sqrt(x) * exp(i * gamma_j * log(x)).
```

This explains both the square-root window scale and the immutable angular
grid. **Assumption:** this architecture may assume RH as a coordinate-design
axiom. It does not claim to prove RH or to make product correctness depend on
RH being proved.

## Decision

Replace the learned layer-29 query/key representation with a factorable,
content-addressed route representation and bounded direct lookup. The source
model remains a teacher and transitional behavioral control; it is not the
definition of the serving geometry.

### 1. Fixed grid and local polar derivative

Let `gamma_j` be an ordinate in the immutable zeta-grid manifest. For prime
atom `p`, define the angular channel

```text
theta_j(p) = wrap(gamma_j * log(p)).
```

For consecutive prime atoms, the local angular derivative is

```text
delta_theta_j(n)
  = wrap(theta_j(p_(n+1)) - theta_j(p_n))
  = wrap(gamma_j * log(p_(n+1) / p_n)).
```

Writing the prime gap as `g_n = p_(n+1) - p_n` gives the small-gap
approximation

```text
delta_theta_j(n) ~= gamma_j * g_n / p_n.
```

The exact log-ratio form is the compiler/reference definition. A compiled
phase-delta table is the intended runtime realization. The runtime updates the
entering and leaving factors; it does not recompute every absolute prefix
coordinate.

The radial signal is separately declared. The first reference candidate is
the square-root-normalized Chebyshev residual

```text
r(x) = (psi(x) - x) / sqrt(x).
```

A divisor-count radial alternative may be tested, but it must not silently
replace `r(x)`: for a square-free `k`-prime n-let, `tau(N) = 2^k` mostly
records context length rather than semantic position.

### 2. Prime atoms, semiprime experts, and n-let context

Each compiled route atom has a registered prime `p_t`. A transition expert is

```text
e_t = p_(t-1) * p_t.
```

The base expert gate requires a square-free semiprime:

```text
rad(e_t) = e_t, omega(e_t) = Omega(e_t) = 2.
```

In the non-degenerate case, consecutive experts expose their handoff without
a global scan:

```text
gcd(e_t, e_(t+1)) = p_t.
```

A length-`k` context is an n-let product

```text
N_t = product(p_(t-r), r=0..k-1).
```

The commutative product is a locality and factor-overlap key, not a complete
ordered identity. Repeated factors and route direction are preserved by the
ordered route record, spin/torsion state, and its canonical kappa.

### 3. S3 compute state and S2 observation

The local non-zero R4 state is normalized onto `S3` and represented as a pair
of complex coordinates:

```text
(z1, z2) = (cos(chi) * exp(i*theta1), sin(chi) * exp(i*theta2)),
|z1|^2 + |z2|^2 = 1.
```

The Hopf observation is the `S2` point

```text
h(z1,z2) = (
  2*Re(z1*conj(z2)),
  2*Im(z1*conj(z2)),
  |z1|^2 - |z2|^2
).
```

Thus `S3` is the compute/spin manifold in R4, while `S2` embedded in R3 is the
observable or heatmap-like projection. The common `S1` fiber phase is not
discarded from storage: it is retained as torsion/transport state so projection
does not destroy route identity.

The 512-channel zeta amplitude vector is a separate spectral heatmap. It must
not be conflated with one four-coordinate `S3` state. A declared compression or
selection operator binds the spectral heatmap to the local R4 state.

### 4. Typed zero/identity bridge

The architecture deliberately distinguishes two uses of `0^0` through the
typed project seam:

```text
exp(i*pi) + pi^0 =_bridge 0^0
continuous_null(0^0) = 0
discrete_empty_product(0^0) = 1.
```

The complex expression cancels to zero. `continuous_null` preserves that value;
`discrete_empty_product` phase-shifts/retypes the boundary as the discrete
identity one. `=_bridge` is a domain-transition operator, not ordinary
numerical equality or an assertion that untyped arithmetic gives both values
simultaneously. The bridge mode is part of the canonical state and therefore
changes its kappa.

The useful ternary landmarks remain

```text
exp(i*pi)          = -1
exp(i*pi) + pi^0   =  0
pi^0               = +1.
```

For a phase `theta`, the proposed collapse gate is

```text
activation(theta) = sin(theta)^2,
chirality(theta)  = sign(sin(theta)).
```

It gives activation `1` at `sin(theta) = +/-1, cos(theta) = 0` and activation
`0` at `sin(theta) = 0`. Chirality retains the `-1/0/+1` orientation that
squaring alone would erase. If the two `sin(theta)=0` antipodes must remain
distinct, `sign(cos(theta))` is retained as an additional polarity bit.

### 5. Direction-preserving golden radial bridge

The discrete complex state and its R4 realization preserve direction. Using
the ordinary identification `E : C^2 -> R4`, define the complex-to-Riemannian
lift

```text
R_phi(z) = phi * E(z),
phi = (1 + sqrt(5)) / 2.
```

For every non-zero state,

```text
normalize(R_phi(z)) = normalize(E(z)),
||R_phi(z)|| / ||E(z)|| = phi.
```

The inverse chart scales by `phi^-1`. Thus angle, prime direction, and Hopf
sector remain invariant across the bridge; only the radial shell changes. This
is a similarity transform chosen by the architecture, not a claim that the
ordinary `C^2 ~= R4` identification itself changes norm.

Golden radial values can be represented exactly in the quadratic integer ring
`Z[phi]`. Store a radius as the coefficient pair `(a,b)` meaning `a + b*phi`.
Then a complex-to-Riemannian radial step needs integer addition only:

```text
(a,b) * phi     -> (b, a+b)
(a,b) * phi^-1  -> (b-a, a).
```

Repeated shell changes therefore follow the Fibonacci recurrence without a
runtime floating multiply. The shell exponent, coefficient pair, direction,
and conversion orientation are bound into the manifest.

The values `sqrt(2)`, `2i`, and `[0,2]` remain related metric landmarks rather
than the radial conversion itself. For unit states `u,v`,

```text
d_chord(u,v) = ||u-v|| = 2*sin(d_geodesic(u,v)/2),
d_chord in [0,2].
```

Orthogonal unit states have chord distance `sqrt(2)`. Antipodal complex states
`i` and `-i` have displacement `2i` and chord magnitude `2`. The raw geodesic
distance is in `[0,pi]`; if a Riemannian score is reported in `[0,2]`, its
normalization or use of chord distance must be explicit.

Compiler code may choose the cheapest faithful chart. A promoted deployed path
must use the exact `Z[phi]` shell update where applicable plus canonical angular
quantization and lookup tables. Its chart, scale direction, and error bounds
must be bound into the artifact rather than switching arithmetic implicitly.

The choice is operation-local and deterministic:

```text
factor, gcd, cardinality       -> discrete prime chart
log-phase delta, conjugation   -> complex chart or compiled phase table
spin, Hopf, torsion transport  -> S3/R4 chart or compiled transition table
distance/ranking               -> cheapest certified integer/table metric
```

For operation `op` and equivalent chart implementations `C(op)`, the normative
compiler will select

```text
chart(op) = argmin(cost(c, target), c in C(op) and fidelity(c) passes),
```

using a versioned target-specific cost profile and canonical tie-break. Every
promoted conversion witness must check preserved direction, the declared `phi`
radial shell change, quantization error, and round-trip identity where the
transform is intended to be reversible. The selected chart, cost-profile CID,
and witness must be kappa-bound. Runtime input must not choose a cheaper but
semantically different arithmetic convention.

### 6. Geometry is the route and the route addresses data

The promoted compiled spin manifest must bind at least:

- zeta-grid CID and revision;
- prime-registry CID and atom-to-prime assignments;
- semiprime expert and n-let overlap tables;
- ordered route keys for last-one, last-two, and sentence contexts;
- R4 spin, Hopf base, fiber phase, torsion, holonomy, and refinement path;
- direction-preserving `Z[phi]` radial shell and complex/Riemannian chart;
- payload CIDs rather than duplicated payload bytes where appropriate;
- tokenizer/corpus/compiler provenance;
- typed bridge conventions and quantization profile; and
- full canonical kappa and rebuild witness.

The factorable geometric address is intended to supply locality. Kappa supplies
exact identity and integrity. Raw kappa bits, hexadecimal spelling, MAC bytes,
or an IPv6-shaped serialization do not define semantic neighborhood.

### SpiralCore v63 compatibility boundary

The local SpiralCore v63 reference, SHA-256
`3f8e6a98186999cca6c55ea42cd8b496935837c2987379f39e8e659b56360215`,
is accepted as a source-free reference candidate, not as semantic-attention
evidence. This decision comes from a static audit. Its embedded JavaScript
fixtures are `NOT_RUN` here because the available browser refuses local
`file://` pages; the page's reported fixture results are therefore claims to
reproduce independently, not evidence imported into this repository.

SpiralCore v63 itself contains no prime-address construction; the following
prime-slot correspondence is this ADR's proposed adapter. For a canonically
selected, ordered, manifest-bound sextet of distinct prime atoms
`(p0,...,p5)`, the fifteen square-free semiprime experts `p_i*p_j` can
share the same unordered-pair index as the fifteen `Cl(0,6)` bivectors
`B_ij = L_i L_j`. Unique factorization then gives the Johnson graph `J(6,2)`:
two experts are adjacent exactly when their GCD is one of the six carrier
primes. The semiprime identifies an unoriented plane; ordered route state and
torsion retain direction. A reverse-step convention may use
`B_ji = -B_ij = B_ij^3`, but its basis, orientation, and causal effect must be
kappa-bound and tested.

A compiled 64-state operator accumulator may be evaluated as bounded holonomy
metadata. It cannot replace ordered route identity because distinct histories
collide into those 64 states. The standard-coordinate E8 root action remains a
separate eight-dimensional test plane, not R4, H4, `S3`, or the Hopf `S2`
observation. Bell MF is an optional pair codec and observability layer. The RFC
3849 IPv6 form is a reversible presentation for constrained symbolic fields;
it is neither a semantic locator nor a carrier for a full 256-bit kappa.

No operator path is promoted until an exact deterministic Rust reproduction
binds the Fano convention, generator order, left/right action, matrices, root
ordering, and transition tables. Sextet selection and chart rollover must be
defined before corpus use, and a causal intervention must beat factor-only and
permuted-slot controls. This bounded port follows the passed current-substrate
worker canary; it does not widen or authorize a corpus-scale run.

As of #952, the exact 64-state finite composition and inverse table is
implemented and deterministically bound in Rust. Its status is
`IMPLEMENTED_CONTROL_ONLY`: it establishes exact noncommutative table mechanics,
not semantic value. The 64 states still cannot replace route identity, and no
causal attention claim follows from the table alone.

### 7. Geometric attention

The promoted compiled artifact must provide three causal indexes:

```text
I1[last_route]                    -> bounded next-route candidates
I2[previous_route, last_route]    -> bounded next-route candidates
IS[ordered_sentence_route_key]    -> bounded continuation candidates
```

Candidate support must be the bounded union of those rows plus declared
divisor-overlap and adjacent-spin fallbacks. Selection must minimize a declared
integer/table-backed energy over factor overlap, prime-gap phase derivative,
S3 transport continuity, S2 observable compatibility, torsion, and compiled
continuation evidence. No learned dense Q/K projection and no scan of all
prefix or corpus positions is permitted in this path.

### 8. Attention is not yet reasoning

This ADR specifies the representation and candidate-selection boundary only.
Once implemented and qualified, attention will identify locally relevant next
routes. Reasoning additionally requires bounded multi-step route composition
toward a goal, preservation of intermediate constraints, branch comparison,
and a closure or contradiction test. That mechanism receives its own measured
stage after one-step and sentence-route attention is causally established.
Fluent output inherited from source weights or an external model is not
evidence that geometric reasoning exists.

## Current implementation status

ADR-0003 remains the normative route substrate. ADR-0005 is the current
predictive mechanism target. The #958 code does not by itself establish
geometric attention or product behavior.

### #952 A1.0 qualification update (2026-08-27)

The A1.0 gate preserved that target while separating mechanics that now work
from the blocking representation defect:

- The concrete scaled 120-root H4 table closes uniquely under exact
  binary-icosahedral multiplication. Its exact identity, inverses,
  associativity, deterministic table identity, and integer-only construction
  are an algebra control, not semantic-attention evidence.
- The exact 64-state SpiralCore composition/inverse table is implemented as an
  order-sensitive control. Its semantic status remains
  `OPTIONAL_CONTROL_PENDING`; no semantic claim is attached to its table kappa.
- The GI-1 envelope exposes its frozen schema-2 child manifest and a complete
  codec-registry value view. A child candidate address resolves exactly to its
  separate child-manifest index, stable codec-registry index, payload CID, and
  payload bytes, and the reachable incremental next state reproduces exactly.
- The fixed construction-only manifest supplies both independently declared
  continuations through the real bounded candidate path. On all three matched
  evaluation contrasts, exact last-one, last-two, and ordered-sentence rows
  miss; the adjacent-spin fallback naturally supplies the same two-candidate
  union without injection or truncation, below the declared ceiling of eight.
- Despite that reachability, every reusable non-digest field in all seven
  attention levels—current, previous, last-two, sentence, paragraph,
  conversation, and global—collides for the fixed same-length, same-multiset,
  same-suffix histories whose earlier order differs. Exact kappas differ, but
  digest inequality is identity evidence and is excluded from semantic
  geometry.

The predeclared terminal verdict is therefore
`REDESIGN_ORDERED_ROUTE_SUMMARY`. The recursive attention scorer and its
equal-budget semantic controls were not implemented. Issue #967 subsequently
delivered the narrow ordered-summary repair; this result does not revoke the storage,
candidate-reachability, value-inversion, H4, or SpiralCore mechanics established
above.

The frozen histories were deliberately matched at current, previous, last-two,
length/multiset/boundary shape, and the immutable `gg` global project snapshot.
For A1R/#967, those current, previous, last-two, and global scopes were required
to remain equal; repaired sentence, paragraph, and conversation ordered state
were required to differ.
Any global-order claim requires a separate construction-independent global
snapshot permutation with all lower scopes and candidate support held fixed.
#967's repaired fold satisfied this scope contract, the independent-global
intervention, exact group/fold laws, incremental reproduction, and support
invariants. The report kappa is
`blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.
Its full arm produced distinct candidate-relative states on 6/6 queries, but
shortest Cayley distance collapsed both candidates to energy 2 and tied on 6/6,
so the terminal verdict is `RETAIN_STATE_ONLY`. #970's corrected independent
preflight retains both exact operands `X=C(H,c)` and `Y=C(P_c,c)`, derives
`D=X*Y^-1`, and keys equality only by the exact signed R4 heatmap
`(sin=q0/2, cos=q1/2, sin^2=q0^2/4, chirality, cosine polarity, chart status)`
in `Z[phi]`. Its frozen endpoints map `sin=+/-1, cos=0` to bit 1 and
`sin=0, cos=+/-1` to bit 0 while retaining chirality/polarity. Across 36
decisions it found 14 heatmap classes, 10/12 validation
coverage, a 10/12 no-splitting ceiling, 0/6 strict construction transfer, and
eight incompatible classes. It therefore stopped before readout or placement
with `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.

The contract, complete 14,400-pair/120-relative-state universe, and corrected
report are bound respectively by
`blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
`blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
and
`blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
The 120 relative rows form 45 heatmap classes; 480 ordered pairs are typed-null.
Preparation is target-free and S4 parity is derived from each history. The
fixed-zeta grid, ordered n-lets, golden `phi` maps, and typed cross-chart adapter
are bound only as structural transport: no zeta/n-let-to-`phi` shell-exponent
rule is established. This negative is only a bounded readout-identifiability
result; it does not negate those structures or establish attention. #970 stays
active and #969 stays blocked until protected merge. Only later A1Q-qualified
semantic terms may reach #953.

| Component | Status | Current evidence boundary |
|---|---|---|
| Typed bridge; prime, square-free semiprime, n-let/GCD, zeta-delta, S3/Hopf/fiber/torsion, and exact `Z[phi]` algebra | `IMPLEMENTED_SUBSTRATE` | Source-free types and focused tests only. |
| Incremental ordered sentence-route identity and exact bounded I1/I2/IS construction/lookup | `IMPLEMENTED_SUBSTRATE` | Exact-key rows remain available; #952 requires them to miss on the anti-recall contrasts. |
| Hard tiny-canary limits and deterministic whole-sentence partitioning | `IMPLEMENTED_SUBSTRATE` | Compiler refuses corpus-shaped inputs through its public compile API. |
| Per-worker partition/completion/elapsed and peak-active instrumentation | `IMPLEMENTED_SUBSTRATE` | Operational metadata only; excluded from semantic bytes and kappa. |
| Exact scaled-H4 binary-icosahedral multiplication/inverse table | `IMPLEMENTED_ALGEBRA_CONTROL` | All 120 roots, 14,400 products, identity, inverses, and associativity close exactly with integer `Z[phi]` arithmetic. This establishes no semantic value. |
| Exact 64-state SpiralCore v63 `Cl(0,6)` composition/inverse table | `IMPLEMENTED_CONTROL_ONLY` | Deterministic noncommutative table mechanics are bound and reproduced; `OPTIONAL_CONTROL_PENDING`, no semantic claim. The proposed six-prime `J(6,2)` semantic adapter remains unqualified. |
| Complete semiprime/n-let manifest tables, chart/quantization binding, and canonical conversion/rebuild witnesses | `PASS_COMPLETE_MANIFEST_V2` | #958 schema 2 binds the prime registry, semiprime experts, ordered n-lets, address order, chart/spin/radial fields, indexes, provenance, and strict rebuild witnesses. This is storage/candidate substrate, not semantic-attention evidence. |
| Frozen schema-2 child, bounded direct/divisor/adjacent-spin candidate union, and exact address-to-payload value view | `IMPLEMENTED_SUBSTRATE` | The fixed #952 contrasts naturally expose both targets below the ceiling of eight, with exact direct rows absent and no target injection or truncation. Candidate reachability is not attention selection. |
| Reusable order-sensitive route summary and recursive attention scorer | `A1P_RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q_PENDING_PROTECTED_MERGE` | #967 repaired ordered state but shortest Cayley distance mapped distinct states to equal energy on 6/6 queries. The corrected local #970 probe exhaustively bound 14,400 paired-H4 witnesses to 120 relative roots and 45 exact R4 heatmaps; its 36 exercised decisions formed 14 classes with 10/12 validation coverage, 10/12 oracle ceiling, 0/6 transfer, and eight incompatible classes. This is a readout-identifiability negative only. #970 remains active and #969 blocked until protected merge; no full-attention or generation claim follows. |
| Layer-29 caller, equal-budget controls, no-Ollama product probe, and teacher comparison | `NOT_YET_IMPLEMENTED` | No deployed or product capability follows from the substrate. |
| Binding one-worker/four-worker release canary with positive work on every worker and measured four-worker compile-stage improvement | `PASS_SUBSTRATE_SCOPE` | The frozen schema-2 report records 32/32 exact artifact/kappa matches, all four workers active, and 1.498x median compile-stage speedup. Any later semantic-input or workload-shape change must re-establish this bounded evidence. |

## Options considered

### Continue the learned four-coordinate mixer

Rejected. #951 showed support reachability but did not establish useful
semantic-value transport. Continuing to tune it would optimize a representation
that omits the factorable address and fixed-grid mechanism.

### Use kappa, MAC, hexadecimal, or IPv6 bits as geometry

Rejected. MAC is optional hardware provenance, hexadecimal is notation, IPv6
is transport-sized serialization, and a cryptographic digest deliberately
destroys locality. They may bind or carry a route but cannot replace it.

### Recompute similarity against every stored position

Rejected. It leaves the system opaque and superlinear, and ignores the direct
factor and route-context indexes that compilation can provide.

### Fixed-zeta factor route plus local spin transport

Selected. It preserves the original data-as-location invariant, supports local
incremental updates, and admits source-free algebraic tests before any costly
teacher or product run.

## Consequences

- The current learned-mixer redesign code is diagnostic history, not the target
  implementation.
- Corpus text supplies ordered route observations. Source weights may supply
  offline teacher labels and a bounded transitional syntax control, but are not
  stored as the serving route geometry.
- The first implementation is allowed to use compiler-side floating point to
  build and verify tables. Runtime lowering is a separate measured step.
- H4 unit quaternions may provide a discrete spin alphabet in R4. This does not
  identify H4 with E8 or claim that a four-dimensional H4 construction is a
  faithful eight-dimensional E8 representation.
- Paired-H4/icosian state remains required for canonical storage, address
  reconstruction, and inverse witnesses. That structural requirement does not
  qualify H4, Hopf, zeta, icosian, SpiralCore, or other geometric fields for
  semantic ranking; A1Q/#969 must qualify each scoring term before #953 can use
  it.
- Every lossy projection declares what is discarded and what the manifest
  retains for reconstruction.
- No hours-long run launches until direct-lookup reachability, deterministic
  rebuild, four-worker scaling, progress/ETA, checkpoint, and distinct decision
  branches pass on a tiny corpus.

## Bounded G1R acceptance order

1. Source-free bridge truth table, semiprime/n-let, GCD handoff, Hopf fiber,
   phase-delta, and chart-conversion tests.
2. Complete canonical manifest and hard worker canary: route order, torsion,
   bridge mode, basis, payload, and provenance changes must change kappa;
   worker count must not. The same tiny corpus must emit byte-identical
   one-worker/four-worker artifacts, all four workers must process non-empty
   partitions, telemetry must report elapsed time and throughput, and four
   workers must improve release-mode wall time. Otherwise stop and profile.
   The current source-free substrate has passed this worker-only portion; the
   complete-manifest portion remains open. Its append-only evidence is
   [`prime_route_worker_canary_958.md`](../prime_route_worker_canary_958.md).
3. Separate source-free operator proof: deterministically reproduce the v63
   Fano convention, six generators, fifteen bivectors, 64-state table, and
   `J(6,2)` correspondence in Rust. Bind every convention and compare against
   factor-only and permuted-slot controls; do not use the E8 display plane as a
   proxy for semantic value. A negative operator result retains the factor-only
   route substrate and does not block testing that mechanism.
4. Causal intervention: last-one, last-two, sentence-route, torsion, and
   phase-delta perturbations must change the declared candidate support or
   ordering without exposing future routes.
5. Bounded no-Ollama product probe comparing the real route indexes with
   equal-budget permuted and count-only controls.
6. Only if those gates pass, one time-bounded source-teacher comparison may be
   run. A negative result preserves the storage/recall substrate without
   promoting geometric attention or reasoning.
