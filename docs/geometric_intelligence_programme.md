# Geometric Intelligence Programme

- **Status:** Authoritative for post-#958 intelligence architecture, sequencing,
  and claim boundaries
- **Adopted:** 2026-08-26
- **Foundation evidence:**
  [#958 prime-route qualification](prime_route_attention_qualification_958.md)
- **Supersedes for forward sequencing:**
  [Geometric Causal Decoder Roadmap](geometric_causal_decoder_plan.md)
- **Historical measurements:** [Research ledger](RESEARCH.md)

## Purpose

UOR-R4 is building geometric intelligence for ordinary local machines. The
serving engine is intended to generate, infer, and reason by following
content-addressed geometric routes, not by executing a transformer, mixture of
experts, sparse learned router, or dense matrix stack.

The research goal is frontier-like useful capability on a local CPU without
the power, heat, and accelerator dependence of contemporary dense models.
**Frontier-like is a target, not a current result.** The repository has not yet
established source-free coherent chat, correctness, or reasoning; every such
capability must be earned in the sequence below. Spherical harmonics are the
programme's primary physical/mathematical picture for storing and transporting
overlapping spin states: a routed state is a multichannel harmonic field, while
R4/S3 and Hopf/S2 are bounded compute and observation charts of that field.

The central invariant is:

> **The geometry is the route, and the route is the data location.**

Text is not made intelligent by its cryptographic digest or by putting token
IDs in a table. A canonical lexical payload is assigned a factorable location;
the ordered route through those locations carries causal context; local
spin/torsion changes move the state; and a bounded least-cost lookup chooses the
next route. The route must remain reconstructable and causally falsifiable.

The programme is sequenced by capability: lexically reversible geometry first,
then recursive attention, then source-free inference/generation, then measured
correctness, and only then multi-step reasoning. Attention must exist before an
output can be credited to geometric inference.

Optimization, broad QA, formalization, and release certification follow a
working decision-bearing product slice. They do not replace it.

## Current truth after #958

#958 is foundation evidence, not abandonment of geometric intelligence. It
retained the following source-free mechanisms:

- typed bridge, prime-route, semiprime, ordered n-let, fixed-zeta, S3/Hopf,
  torsion, and exact `Z[phi]` algebra fixtures;
- a canonical schema-2 spin manifest with complete route/rebuild witnesses;
- deterministic bounded I1/I2/ordered-sentence lookup;
- a seven-row integer/table-backed candidate query with matched controls;
- a source-free SpiralCore operator reproduction with a distinct optional
  six-prime chart; and
- byte-identical one-worker/four-worker artifacts with useful four-worker
  occupancy and a 1.484x measured compile-stage speedup.

The programme also carries forward the ancestor's scoped
[geometric-context evidence](prime_router_geometric_context_evidence.md): its
context signal lived in the transported trajectory, session hypersphere state,
winding/window classification, window projection energy, shared-prime-factor
retrieval, cosine resonance, and accumulated Hopf phase—not only in an exact
route key or final coordinate. That ancestor used Ollama for language and is not
generation evidence, but its overlapping context mechanism is a required input
to the new source-free design.

Its terminal outcome was `RETAIN_STORAGE_RECALL_ONLY`. That wording is a claim
boundary: the core experiment is not connected to a lexical generation loop,
has no independent attention-artifact kappa, and has no API or chat caller.
Incremental state maintenance is not yet certified as an integer-only,
allocation-free serving path. Nothing in that result refutes prime-route
geometry; it identifies the missing product boundary.

The old #952 all-layer replacement step is retired. The live GitHub graph now
assigns #961 to lexical geometry/state plumbing and blocks the rewritten #952
recursive-attention stage behind it.

## Architecture invariants

### 1. Data is location

Each lexical or semantic atom receives a registered prime. The geometric
address binds the prime, payload CID, harmonic/spin state, radial shell, route
conventions, and provenance. Payload bytes may live in a canonical payload
store, but their address is the factorable geometry rather than an arrival
order or opaque embedding.

Cryptographic identity and geometric locality are different jobs:

- the prime/n-let/spin address supplies neighborhood and route operations;
- the payload CID supplies content integrity; and
- kappa supplies canonical object identity and serialization integrity.

Raw kappa bits, hexadecimal spelling, MAC bytes, IPv4, or IPv6 may identify or
transport an object. They do not define its semantic neighborhood.

### 2. Fixed grid; local state change

The ordered non-trivial zeta-zero ordinates form the immutable spectral grid.
This is an **Assumption** used as a coordinate design; it is not a claim to
settle the Riemann hypothesis.

For prime `p` and fixed ordinate `gamma_j`, the compiler may bind the phase

```text
theta_j(p) = wrap(gamma_j * log(p)).
```

For a local step `p -> q`, the relevant update is the phase delta

```text
delta_theta_j = wrap(gamma_j * log(q / p)).
```

The grid identity and payload identity do not change during inference. Torsion,
spin, phase, radial shell, and accumulated route state change locally. A node
updates the current state from the last state and proposed next route; it does
not recompute the absolute geometry of the entire history.

Spherical-harmonic or fixed-zeta coefficients may provide the multichannel
spectral field. A four-coordinate state is a selected local chart of that
field, not a claim that the full harmonic population is only four numbers.

### 3. Prime experts and ordered Hamiltonian context

A semantic atom maps to a prime `p`. An adjacent transition maps to a
semiprime expert:

```text
e_t = p_(t-1) * p_t.
```

When the factors differ, this is a square-free distinct-edge expert. When the
route repeats, `p^2` is a valid semiprime self-loop. The self-loop must not be
discarded merely because an optional six-prime operator chart uses only
distinct pairs.

Higher context is carried by ordered prime n-lets. The commutative factor
product exposes divisor overlap and inexpensive neighborhood lookup; the
ordered record, repeated factors, spin, torsion, and route kappa retain
direction and history. These combinatorics carry the Hamiltonian context used
by least-cost continuation.

For a six-prime chart, the 15 unordered distinct pairs may index the 15
square-free `J(6,2)` edges and the 15 `Cl(0,6)` bivectors. Those fixtures remain
separate from general `p^2` route self-loops.

### 4. Typed zero/identity bridge

The architecture names the domain seam with

```text
e^(i*pi) + pi^0 =_bridge 0^0.
```

The equality sign at this seam is typed as a domain-transition operator. The
complex side cancels to zero; the selected bridge either preserves that null or
phase-shifts/retypes it as the discrete empty-product identity:

```text
e^(i*pi) + pi^0 =_bridge 0^0
ContinuousNull:        0 -> 0
DiscreteEmptyProduct: 0 -> 1.
```

This is not ordinary untyped numerical equality; it is the explicit conversion
between continuous-null and discrete-identity calculations that the route has
selected. The bridge mode is part of canonical state and changes kappa.
Together with `e^(i*pi) = -1` and `pi^0 = +1`, it supplies the
`-1 / 0 / +1` phase landmarks. Any collapse rule must retain chirality or an
explicit polarity bit when squaring would lose direction.

### 5. Trigonometric collapse and chart continuation

For a phase state with `s = sin(theta)` and `c = cos(theta)`, the canonical
collapse fields are

```text
activation(theta) = s^2
chirality(theta)  = sign(s)
polarity(theta)   = sign(c)
theta             = atan2(s,c).
```

The required landmarks are

```text
(sin = +/-1, cos = 0) -> activation 1
(sin = 0,    cos = +1) -> activation 0.
```

Chirality retains the `-1 / 0 / +1` orientation that `s^2` would erase.
Cosine polarity distinguishes the two zero-activation antipodes when that
distinction is required. Angle recovery uses `atan2(s,c)`, never an unchecked
`atan(s/c)`.

A tangent pole or typed-null boundary does not end the route. If the active
tangent chart would divide by zero, the state switches to its complementary
chart and records a canonical quarter-turn:

```text
chart        -> complementary(chart)
phase        -> wrap(phase + q*pi/2)
torsion      -> wrap(torsion + q*pi/2)
q            -> declared orientation in {-1,+1}.
```

The orientation `q`, source/destination chart, bridge mode, and phase/torsion
shift are kappa-bound. At an encoded `(s,c)=(0,0)` typed-null sentinel, the
typed bridge and previous declared orientation choose the continuation; this
sentinel is not an ordinary trigonometric point. The implementation must not
invent an angle, divide by zero, or silently terminate the route.

### 6. R4/S3 compute and Hopf observation

The local non-zero compute state is a normalized point on `S3` in R4. The Hopf
map supplies an `S2` observation embedded in R3. These spaces have different
roles:

- R4/S3 carries local compute, spin, direction, and fiber state;
- S2/R3 supplies the observable heatmap or sector view; and
- the common S1 fiber phase is retained as torsion/transport state.

Projection is not deletion. A manifest must retain enough fiber, route, and
radial information to rebuild the state it claims to preserve.

### 7. Golden radial shells

Radial conversion uses golden shells represented in `Z[phi]`, with

```text
phi = (1 + sqrt(5)) / 2
(a,b) means a + b*phi
(a,b) * phi     -> (b, a+b)
(a,b) * phi^-1  -> (b-a, a).
```

The exact pair update uses integer add/subtract and produces the Fibonacci
recurrence across shells. Direction remains unchanged while radial scale moves.
Shell exponent, coefficient pair, orientation, quantization, and round-trip
witness are manifest-bound.

### 8. Paired H4 to E8 implementation bridge

The broader UOR action plane keeps the conceptual identity

```text
project shorthand: E8 = H4 x H4.
```

The concrete implementation contract realizes that shorthand through the
golden/Galois-coupled icosian/600-cell folding

```text
H4 direct-sum phi*H4
```

written mathematically as `H4 ⊕ phi H4`: an E8 lattice point is represented,
as a `Z`-module construction, by a golden-coupled pair of R4 points under one
fixed basis and glue convention.

This is a load-bearing **Architecture Assumption**. `E8 = H4 x H4` is the
project's conceptual name; `H4 ⊕ phi H4` is the exact construction that code
and canonical serialization must bind. The distinction specifies the
representation—it does not reject the project identity. It also avoids
silently treating R4, an S3/Hopf state, or an unbound abstract product as the
serialized eight-dimensional state. The bridge must declare:

- the two H4-derived charts and their basis/order;
- the `H4 ⊕ phi H4` golden coupling and glue into the eight-dimensional
  lattice/action plane;
- the fixed forward map and inverse/reconstruction witness;
- collision and quantization behavior; and
- a kappa-bound witness for every preserved invariant.

The paired construction participates in route transport and candidate energy.
A matched factor-only and permuted-pair intervention must show its causal
contribution before source-free generation is promoted. The factor-only route
remains the required diagnostic control, not a substitute final architecture.
The optional six-prime SpiralCore chart retained by #958 is a separate operator
fixture; it does not satisfy this paired-H4/E8 requirement by itself.

### 9. Least-cost certified chart selection

Equivalent operations may be cheaper in different mathematical substrates:

```text
factor, gcd, cardinality       -> discrete prime chart
phase delta, conjugation       -> complex chart or compiled phase table
spin, Hopf, torsion transport  -> R4/S3 chart or compiled transition table
distance and ranking           -> certified integer/table metric
paired-H4 action               -> witnessed H4 ⊕ phi H4 / E8 bridge
```

For operation `op`, target `t`, and candidate chart set `C(op)`, the compiler
selects

```text
chart(op,t) = argmin cost(c,t)
              where c in C(op) and fidelity(c,t) passes.
```

The cost profile, target, fidelity bounds, canonical tie-break, selected chart,
and conversion witnesses are all kappa-bound. Runtime input cannot silently
select a cheaper but semantically different convention. “Least cost” is an
empirical, target-specific result—not a universal statement that one geometry
is always cheaper.

The least-cost adapter also binds three project-defined chart markers:

```text
sqrt(2)  -> Euclidean orthogonal-unit chord marker
2i       -> complex/discrete antipodal displacement marker
[0,2]    -> normalized Riemannian/chord-distance interval marker.
```

These markers select conversion conventions and fidelity checks. They are not
a literal equality between Euclidean, complex/discrete, and Riemannian
mathematical domains. The complex marker retains its orientation and has
magnitude `2`; the Riemannian marker must state whether it uses chord distance
or another explicit normalization.

### 10. Kappa is identity, not language

Kappa is the canonical identity of serialized state. It binds the grid,
registry, payload identities, route order, semiprime/n-let tables, bridge mode,
charts, cost profile, spin/torsion state, radial shells, indexes, witnesses,
and provenance.

Kappa is not the tokenizer, an embedding, a semantic distance, or a next-token
policy. Changing worker count must not change semantic kappa. Changing a
semantic input must.

The first product uses a pinned lexical codec to map between text and stable
token payloads. The codec may be the existing SmolLM2 byte-BPE implementation,
but loading that vocabulary does not authorize loading model weights. Its CID,
normalization, special-token policy, and encode/decode behavior are manifest
inputs.

## Ingestion and storage

The initial source-free ingestion path is:

```text
canonical text/corpus
    -> pinned lexical codec
    -> ordered token payloads and payload CIDs
    -> prime registry
    -> semiprime transitions + ordered hierarchical n-lets
    -> fixed-zeta harmonic state + R4/S3 spin/torsion + Z[phi] shell
    -> exact recursive indexes + overlapping trajectory/harmonic summaries
    -> rebuild/coverage witnesses
    -> canonical manifest and kappa
```

Corpus observations populate routes. They do not become source weights. The
manifest stores or references the spin/harmonic state, full transported
trajectory summary, session hypersphere vector, winding/window state, window
projection energy, accumulated Hopf phase, route topology, payload identity,
lexical provenance, and reconstruction evidence needed to recover the same
object. Like spin states may share a canonical operator/chart entry, so a
single bound state transition can update all references to that state without
duplicating the calculation.

An offline source model may later label examples, propose candidate rankings,
or act as a quality comparator. Those observations must be converted into
explicit compiled evidence with provenance. Source tensors are never serving
geometry and are never required to answer a request.

## Recursive causal geometric attention

Attention is a recursive route lookup, not a Q/K dot-product approximation.
At step `t`, only observed state and declared persistent memory may enter the
query. The actual future route is never an input.

The state hierarchy is:

| Level | Bound causal context |
|---|---|
| `R0` | current route and proposed next candidate |
| `R1` | previous route -> current/next candidate |
| `R2` | last two ordered routes -> next candidate |
| `RS` | accumulated sentence route/holonomy -> next candidate |
| `RP` | accumulated paragraph route -> next candidate |
| `RC` | accumulated conversation route, including prior turns and identity-scoped memory |
| `RG` | global accumulated route over the admitted knowledge/memory scope |

Each level carries both an exact ordered route identity and overlapping
trajectory/harmonic summaries. The summaries include the transported path,
session hypersphere vector, accumulated Hopf phase, winding/window state, and
window projection energy. Each is updated incrementally from its prior state
and the newly observed route. Higher levels summarize and constrain lower-level
candidates without rescanning every token or corpus position.

The candidate set is admitted from bounded exact rows plus shared prime factors,
n-let overlap, adjacent spin sectors, window/trajectory overlap, and bounded
cosine resonance against content-derived stored state. A least-cost certified
integer/table form replaces floating cosine where the fidelity witness passes.
Admission bounds are explicit; least-energy ranking runs only over the admitted
set.

An exact kappa miss is not permission to collapse paragraph, conversation, or
global state to a suffix-only default. When an exact route is unseen, the
overlapping summaries must still distinguish histories whose transported
trajectory, hypersphere state, winding window, projection energy, factor
overlap, resonance, or Hopf phase differs.

The energy is an ordered, witnessed comparison over applicable terms such as:

- exact context and source breadth;
- divisor and n-let overlap;
- fixed-zeta phase continuity;
- S3 spin and Hopf-sector compatibility;
- torsion and accumulated holonomy;
- golden radial-shell cost;
- transported-trajectory, hypersphere, winding/window, projection-energy,
  shared-factor, cosine-resonance, and accumulated-Hopf compatibility;
- paragraph, conversation, and global constraint agreement; and
- compiled lexical/grammar continuation evidence.

No learned dense Q/K projection, softmax over a full prefix, all-corpus scan,
MoE gate, or sparse learned router is permitted in the promoted serving path.

### Global context coverage witness

Every selected route must carry a coverage witness that makes “global context”
auditable. At minimum it records:

- the kappa and route key consulted at every hierarchy level;
- whether each level hit, missed, abstained, or used a declared fallback;
- which exact-key and overlapping trajectory/harmonic channels contributed;
- the number of candidate entries examined and pruned at each bound;
- the constraints contributed by sentence, paragraph, conversation, and global
  state;
- conflicts, unresolved constraints, and the deterministic resolution rule;
- the selected route, payload CID, energy trace, and rebuild witness; and
- the portion of admitted global memory that was reachable under the declared
  scope.

A witness can establish coverage of a declared scope. It cannot by itself
guarantee that the stored knowledge is complete or that an answer is correct.
Correctness receives its own stage.

## Source-free grammar and generation

The lexical codec only segments and reconstructs text. Grammar, syntax, and
continuation must come from the route engine. The source-free generator must
compile and use explicit route structures for token/word class, agreement,
punctuation, clause/sentence closure, ordered n-let continuation, and higher
context constraints. It must update the recursive state after every emitted
token and commit completed turns to the identity-scoped manifold.

Exact corpus replay is a recall control, not inference. A readable completion
copied from an exact sentence route cannot pass the source-free generation gate
by itself. Held-out prompts, exact-recall labels, geometry-permuted controls,
count-only controls, and equal candidate/token budgets must separate stored
grammar from load-bearing geometry.

## Programme sequence

### GI-0 — retained foundation

Status: **established within #958's declared source-free scope**.

Keep the schema-2 prime-route manifest, worker compiler, bounded query closure,
controls, and optional SpiralCore fixtures. Add no product claim to them. Any
semantic manifest change rebinds the artifact and reruns only the canary needed
for the next product decision.

### GI-1 / #961 S0 — lexical geometry and state plumbing

Build the reversible, API-neutral substrate required by attention:

1. pin the lexical codec without opening weights;
2. compile arbitrary prompt tokens into manifest-bound prime/spin addresses;
3. implement the address-to-token-payload inverse without corpus-text lookup;
4. serialize the entire local/sentence/paragraph/conversation/global hierarchy
   canonically and give the attention artifact its own kappa;
5. bind the fixed `H4 ⊕ phi H4` basis, glue, forward map, and inverse witness;
6. maintain all hierarchy levels incrementally; and
7. expose API-neutral state/query types only.

GI-1 does not generate text, add a CLI/chat caller, or claim attention. Its exit
is deterministic lexical/address round-trip, complete hierarchy identity, and
rebuildable paired-H4 state.

### GI-2 / #952 — recursive geometric attention

Implement attention across current/next candidate, previous, last-two,
sentence, paragraph, conversation, identity-scoped memory, and global
accumulated route state. Every selection carries the global context coverage
witness. Interventions must show that each hierarchy level and the paired-H4
state changes admitted support or ordering on a fixture designed for that
level, without future-route leakage or a population scan.

Anti-recall evidence is mandatory: exact-row hits are labeled separately;
held-out histories cannot be exact stored continuations; real geometry shares
payload and budgets with exact-recall, factor-only, count-only,
permuted-geometry, and permuted-pair controls. Unseen paragraph, conversation,
and global histories must remain distinguishable through their overlapping
trajectory/harmonic summaries even when exact route kappas miss.

GI-2 closes only when full recursive attention is causally load-bearing. It
does not emit or score grammatical product text.

### GI-3 / #953 — source-free grammatical inference and generation

After GI-2 attention passes, compile explicit lexical class, agreement,
punctuation, clause/sentence closure, and ordered continuation geometry. Connect
one route-native implementation to the library API, CLI, HTTP, and chat
surfaces. Emit bounded autoregressive text using no Ollama, source weights,
dense matrix model, transformer, MoE, or sparse learned router.

The gate asks whether the attention-qualified geometry produces distinct,
decodable, non-cycling, prompt-responsive text beyond exact recall. It records
real, factor-only, permuted-pair/permuted-geometry, count-only, and exact-recall
arms separately. This is source-free grammatical inference/generation, not yet
a claim that answers are correct.

### GI-4 / #954 — correctness and abstention

After source-free generation exists, test whether the engine understands the
input well enough to choose a correct answer. Use held-out questions with
answerable, unanswerable, contradictory, and context-dependent cases. Score
correctness, relevance, calibration/abstention, constraint coverage, and causal
dependence on the required route levels.

An offline teacher may provide labels or a comparator only after the
source-free arm has a frozen report. Teacher output is never substituted for
the product response.

### GI-5 / #955 — reasoning

Reasoning begins only after correct one-step inference. Add bounded multi-step
route composition with:

- explicit goal and intermediate constraints;
- branch creation and comparison;
- reversible state updates;
- closure, contradiction, or abstention tests;
- a trace showing which route operation produced each intermediate; and
- controls that break one required premise or route.

A fluent rationale is not reasoning evidence. The selected conclusion and its
intermediate constraints must change under the matched causal control.

### GI-6 / #962–#965 — product integration, serving purity, measured cost, and bounded release

Freeze one accepted path and remove any remaining transitional serving
dependency. The final serving census must show:

- zero source-model weight loads and teacher forwards;
- zero transformer/self-attention calls;
- zero dense matrix intelligence kernels;
- zero MoE or sparse learned-router calls;
- bounded route, table, integer, and witnessed chart operations only; and
- one canonical API/chat implementation with deterministic identity and
  declared resource ceilings.

Profile before changing arithmetic. The least-cost chart selector and worker
plan may be optimized only against the accepted product behavior.

Activate product/release checks for the frozen capability, publish exact limits,
and bind the release artifact, lexical codec, manifests, coverage witnesses,
and serving census. Formal proofs, exhaustive conformance, cross-target work,
and broad performance campaigns are separately authorized only when they can
change this release decision.

## Testing and QA activation policy

Testing and QA are **dormant by default**. Existing suites and historical
certificates remain in the repository, but routine implementation does not run
them automatically and does not create new QA infrastructure.

A check is activated only when a product or release decision names:

1. the exact decision the check can change;
2. the product path or release claim it exercises;
3. the fixture and evidence identity;
4. the positive, negative, and stop actions; and
5. the time, disk, memory, and worker budget.

Source-free product probes, causal controls, and serving-path censuses are
product evidence and may be activated by their stage contract. Compilation,
cross-target, deterministic rebuild, kappa reproduction, BDD, fuzz,
conformance, formal, corpus-scale, and broad workspace suites stay dormant
unless the active product/release decision explicitly requires them. Automatic
QA is disabled. Because legacy ruleset `19597522` cannot currently be edited,
pull-request and merge-group events publish only five instantaneous no-QA
transport acknowledgements. They carry no evidence and must never be reported
as PASS. #940 tracks eventual removal of the obsolete rule and queue.

No run expected to exceed 15 minutes starts without a finite denominator,
progress, ETA, checkpoint, hard wall, and decision-bearing run contract. Eight
hours remains a hard kill ceiling, never an estimate.

## Claim boundaries

- #958 established a storage/recall foundation and bounded causal mechanics,
  not product attention.
- A tokenizer is a lexical codec, not intelligence.
- Kappa is canonical identity, not locality or semantics.
- A fixed zeta grid is a coordinate assumption, not a result about RH.
- Project shorthand is `E8 = H4 x H4`; its concrete implementation and
  serialization contract is the golden/Galois-coupled icosian pair
  `H4 ⊕ phi H4` with fixed basis, glue, and inverse witness.
- S3/R4 compute, Hopf S2/R3 observation, and an E8 action plane are distinct
  objects.
- Exact recall, grammatical text, correct inference, and reasoning are separate
  gates.
- Offline teacher agreement is not a source-free product result.
- No frontier, general-intelligence, correctness, or energy-superiority claim
  follows until its own declared product evidence exists.

## Historical preservation

The TLA/R4G1 compiler/runtime, graph programme, learned layer-29 mixer,
historical geometric causal decoder plan, formal lanes, measurement records,
and their negative or positive outcomes remain intact at their recorded scope.
They may serve as comparators or components only when a new stage explicitly
names them. They no longer decide forward intelligence sequencing.

The ancestor
[prime-router geometric-context record](prime_router_geometric_context_evidence.md)
remains `some-true` evidence for structural context routing, not source-free
token generation. Its overlapping trajectory mechanism is carried forward here
without importing its Ollama language output as product evidence.
