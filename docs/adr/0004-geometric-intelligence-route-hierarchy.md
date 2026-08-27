# ADR-0004: Define a bounded geometric-intelligence route hierarchy

- **Status:** Accepted as an architectural definition and evaluation boundary;
  paragraph, conversation, global, inference, and reasoning capability remain
  unproven and off the product path.
- **Date:** 2026-08-26
- **Builds on:** [ADR-0003](0003-fixed-zeta-prime-route-attention.md)
- **Evaluation:**
  [Geometric Intelligence Evaluation](../geometric_intelligence_evaluation.md)
- **Terminology:** [Glossary](../transformerless/GLOSSARY.md) and
  [Formal Vocabulary](../formal_vocabulary.md)

## Context

The fixed-zeta prime-route substrate establishes factorable storage identities,
ordered route keys, and bounded local lookup. It does not yet define how “last
route,” “whole sentence,” “conversation so far,” and shared background context
coexist without repeatedly scanning all preceding tokens. It also does not
separate exact continuation recall from attention, inference, correctness, or
reasoning strongly enough for future product evaluation.

The hierarchy must preserve the original principle that geometry is the route
and data is addressed by that route, while keeping kappa as exact integrity and
provenance identity. It must permit incremental updates and bounded reads. A
global context mechanism that scans the corpus or serializes the entire prefix
at every token would defeat the intended compute advantage.

The ancestor evidence record
[`prime_router_geometric_context_evidence.md`](../prime_router_geometric_context_evidence.md)
also constrains the design. Its reported ablations locate routed signal in the
full transported trajectory rather than one coordinate: masking the initial
trajectory or retaining only the last state sharply reduced coordinate-
tracking accuracy, while the transported final state became linearly readable
in the reported reduced regime. The ancestor combined a session hypersphere
vector, winding/window classification, projection energy, shared-prime factors,
cosine resonance, and accumulated Hopf phase. Those measurements predate
current reproduction discipline and used an Ollama generation surface, so they
motivate the route state but do not establish current attention, inference, or
language generation.

## Decision

Adopt five ordered, identity-scoped route levels: local, sentence, paragraph,
conversation, and global. Each level is an incremental accumulator over bounded
child records. A parent commits to ordered child kappas, span boundaries,
provenance, and a declared factorable geometric summary. That summary preserves
the signals needed for overlap when an exact kappa misses: the transported
trajectory commitment, session hypersphere vector, winding/window state,
projection energy, shared-prime factors, cosine resonance, accumulated Hopf
phase, and paired-H4/E8 coordinate. Query reads a fixed artifact-declared
number of rows at each enabled level; it never enumerates all descendant tokens
or corpus records.

This ADR defines identities, boundaries, and evaluation. It does not promote a
serving implementation.

## 1. Canonical route state

For scope `q`, define one route-state envelope:

```text
R_t^q = (
  schema, scope=q, identity_scope,
  zero_identity_bridge_mode,
  previous_chain_kappa, ordered_child_kappa,
  span_start, span_end, boundary_kind,
  prime_factors, zeta_phase_signature,
  s3_spin, s2_hopf_observation, torsion, z_phi_radius,
  trig_angle, activation, chirality, cosine_polarity,
  active_chart, quarter_turn_phase_torsion_shift,
  cross_domain_cost_profile,
  session_hypersphere_vector, winding_window_state,
  projection_energy, shared_prime_factors, cosine_resonance,
  accumulated_hopf_phase, paired_h4_e8_coordinate,
  transported_trajectory_commitment,
  payload_or_summary_cid, provenance
)
```

**Definition:** `kappa(R_t^q)` is the canonical content identity of this exact
envelope. The digest supplies equality and integrity, not geometric distance.
Factor, phase, spin/Hopf, torsion, and radial fields supply declared locality.

The bridge mode is exact identity data. The project seam is
`exp(i*pi)+pi^0 =_bridge 0^0`. In `ContinuousNull`, the typed transition
preserves the complex cancellation as `0`. In `DiscreteEmptyProduct`, it
phase-shifts/retypes the boundary into the discrete empty-product identity `1`.
Here `=_bridge` is an explicit domain-transition operator, not ordinary
numerical equality. The architecture therefore chooses a route value and
retains its type instead of asserting both interpretations in one untyped
calculation.

**Definition:** the S3/R3 angular transition stores
`theta=atan2(sin(theta),cos(theta))`, activation `sin(theta)^2`, chirality
`sign(sin(theta))`, and cosine polarity where antipodes must remain distinct.
Thus `(sin=+/-1,cos=0)` is active `1`, while `(sin=0,cos=+1)` is the continuous
null `0`.

**Assumption:** tangent is a local chart, never the route identity. At its pole
or a declared null boundary, the route switches to its angle/cotangent chart
and carries an explicit signed quarter-turn phase plus torsion shift. It does
not divide by zero and does not terminate the route. The chart transition is
kappa-bound; its semantic value remains unproven.

**Definition:** `sqrt(2)`, `2i`, and `[0,2]` are typed least-cost adapter
markers: respectively the Euclidean chord between orthogonal unit directions,
the complex/discrete antipodal displacement, and the declared normalized
Riemannian/chord score interval. They do not equate mathematical domains. A
versioned cost profile may select the cheapest faithful operation only after
binding units, direction, chart, quantization, error bounds, canonical
tie-break, and conversion witness.

**Definition:** updating an open scope appends one fixed-shape child step to its
ordered chain and incrementally transports its overlapping harmonic/trajectory
summary. Closing a scope freezes its final route state and makes that state one
child of the next scope. Implementations may use incremental hashes and
compiled tables, but the versioned envelope above is the rebuild meaning. The
full trajectory need not remain as an unbounded vector; its bounded commitment
and declared sufficient summary must remain reproducible from the bounded
coverage/checkpoint witness.

Every observed child address must be an exact member of the bound manifest
address registry. A new ordered combination of valid addresses may miss every
stored higher-scope row and remain a valid causal state; unseen history is not
the same as an unregistered address.

## 2. Lexical codec and route atoms

The lexical codec is upstream of every route level:

```text
input bytes -> pinned normalization/tokenization -> lexical IDs
            -> registered prime atoms + spin/torsion/radial address
            -> hierarchy state
```

Output follows the bound inverse token/byte path where defined. Codec version,
tokenizer CID, normalization, unknown-unit policy, prime registry, and payload
CIDs are part of the manifest identity.

A prime atom is a registered factorable route identity. An adjacent transition
stores the unordered semiprime expert `p*q`; repetition is represented as
`p^2`, not dropped. Direction and repeated multiplicity remain in the ordered
n-let and scope chain. A raw prime or digest is not a substitute for the
lexical codec.

## 3. Scope hierarchy

| Scope | Ordered children | Boundary/update | Permitted query role | Claim boundary |
|---|---|---|---|---|
| **Local** | Current and bounded previous lexical route addresses | Update per observed lexical unit | Last-one, last-two, divisor, adjacent-spin, paired-H4/E8, and local transport rows | Exact hit is local recall until geometry wins a matched anti-recall intervention. |
| **Sentence** | Ordered local route identities within one detected sentence | Increment while open; close at a codec-bound sentence boundary | Sentence continuation plus transported sentence trajectory/harmonic overlap | A sentence-key hit is recall; grammatical output is not correctness. |
| **Paragraph** | Ordered closed sentence-route identities | Close at a codec-bound paragraph boundary | Cross-sentence support using shared factors, projection energy, resonance, winding, and Hopf phase | No paragraph capability exists until held-out cross-sentence effects are measured. |
| **Conversation** | Ordered turns and paragraph-route identities for one session | Update only after an observed turn/paragraph; session identity isolates state | Session hypersphere trajectory, prior commitments, and bounded conversational memory | Conversation state must not leak across identity scopes; continuity is not reasoning. |
| **Global** | Versioned project/knowledge snapshot entries selected and compiled offline | Immutable during one inference session; changed only by a new content-addressed epoch | Bounded shared background candidates through harmonic/trajectory locality and provenance | “Global” never means an unbounded corpus scan, implicit internet access, or universal knowledge. |

Each enabled scope declares maximum open children, rows per query, candidates
per row, retained candidates after admission, and patch/epoch depth. Exceeding a
scope ceiling closes, summarizes, backs off, or abstains according to a typed
policy; it never silently expands work.

## 4. Load-bearing paired-H4/E8 bridge

**Assumption:** the project's conceptual identity is
`E8 = H4 × H4`. The target hierarchy realizes that shorthand through the
following concrete icosian serialization statement:

```text
Lambda_E8 ~=_Z I (E8 lattice and icosian ring as Z-modules);
B_ico(I) = (x, x') in R4 ⊕ R4;
declared 600-cell folding: Phi_E8 = H4 ⊕ φH4.
```

Here `x` and `x'` are golden/Galois-coupled R4 coordinates and `+` denotes the
declared direct-sum coordinate construction. This does not reject or rename the
project shorthand; it states how `E8 = H4 × H4` becomes canonical bytes and a
rebuildable operator rather than a bare group-name assertion. The artifact must
bind the icosian basis, glue/parity rule, Galois/golden conjugation, scale, shell
membership, root ordering, orientation, inverse/rebuild witness, and
operator-table kappa.

The paired coordinate is load-bearing in the target route hierarchy: it is part
of each transported geometric summary and therefore must participate in
candidate support or ordering. This architectural requirement does not by
itself establish held-out advantage. Promotion still requires an equal-budget
intervention against factor-only and basis/shell-permuted controls on anti-
recall inputs. If it is not load-bearing, the geometric-intelligence target
fails or requires a superseding architecture decision; the prime/Hopf store may
remain useful only as a separately named storage/recall substrate.

## 5. Retrieval, admission, and attention

Each query forms a bounded union from declared rows such as:

```text
local:last-one
local:last-two
sentence:ordered-route
paragraph:ordered-sentence-route
conversation:ordered-turn-route
global:versioned-summary-route
divisor-overlap
adjacent-spin/torsion sectors
transported-trajectory/harmonic overlap
paired-H4/E8 sectors
```

An implementation may apply a common geometry-independent support-admission
rule, for example source breadth followed by total observed count and canonical
address, when the union exceeds the artifact ceiling. That rule and the number
excluded are part of the coverage witness.

When exact hierarchy kappas miss, bounded fallback uses overlapping
trajectory/harmonic locality rather than digest distance: shared-prime factors,
projection energy, cosine resonance, winding/window compatibility, accumulated
Hopf phase, trigonometric chart/quarter-turn transport, and paired-H4/E8
coordinates.

**Definition:** geometric recall returns a continuation because an exact or
declared backoff identity was stored. **Definition:** geometric attention ranks
the admitted causal support using declared geometric energy or compatibility,
and the complete hierarchy geometry is load-bearing against matched controls.
Consequently,
“least energy” means least energy among admitted candidates unless an artifact
explicitly scores the entire bounded union before admission.

No learned dense full-prefix Q/K projection, future-route input, all-prefix
scan, or corpus scan belongs to this route hierarchy. A different transitional
decoder component may still exist, but its work cannot be attributed to this
geometric-attention mechanism.

## 6. Coverage witness

Every evaluated query emits or can deterministically reconstruct a bounded
coverage witness containing:

- lexical codec, manifest, hierarchy, operator, and control identities;
- observed route membership result and identity scope;
- each row key read, scope/source, hit/miss, and entries examined;
- union size, pre-geometric admission policy, admitted and excluded support;
- per-candidate source counts and declared energy components;
- transported-trajectory fields: session hypersphere vector, winding/window,
  projection energy, shared factors, cosine resonance, accumulated Hopf phase,
  and paired-H4/E8 coordinate;
- active trigonometric chart, activation/chirality/polarity, any signed
  quarter-turn phase/torsion shift, and cross-domain cost-profile identity;
- deterministic tie-break stages and geometry source under controls;
- selected route or typed abstention; and
- fixed work ceilings and observed work.

The witness proves reachability and replay of that query only. It cannot by
itself establish generalization, correctness, reasoning, or product readiness.

## 7. Inference, correctness, and reasoning boundaries

**Definition:** inference consumes observed hierarchy state, selects next-token
scores or a token, updates state, and decodes output through the pinned lexical
codec. Attention supplies bounded context; it is not the complete inference
mechanism.

**Empirical Criterion:** correctness is measured against a predeclared
independent oracle or constraint, with answered, incorrect, and abstained
denominators. Status is **Empirical** until a specific report establishes it on
its declared distribution.

**Empirical Criterion:** reasoning requires anti-recall multi-step tasks whose
typed intermediate route states preserve constraints, compare alternatives or
counterfactuals, and reach an independently checkable conclusion. Status is
**Unproven** until such a report exists. Fluent text, a recalled answer, or a
non-zero attention trace does not satisfy this criterion.

**Definition:** provider-free serving makes no runtime call to Ollama, a cloud
model, teacher endpoint, or another generative provider. This property is
orthogonal to transformerless, geometry-only, multiplication-free, correctness,
and reasoning claims.

## 8. Delivery sequence

Lexical ingestion, canonical serialization, registered-address membership, and
rebuild witnesses are prerequisite plumbing. They are not inference.

Delivery proceeds without skipping stages:

1. complete recursive attention through local, sentence, paragraph,
   conversation, and global scopes, including exact-key misses served by
   transported harmonic/trajectory locality and matched controls;
2. implement provider-free inference and coherent generation over that complete
   hierarchy;
3. measure correctness and typed abstention against independent oracles; and
4. measure bounded reasoning through novel multi-step state transitions.

No generation score can substitute for incomplete attention scopes, and no
reasoning claim can precede correctness evidence.

## 9. Consequences

- Last-route and whole-context information have distinct typed scopes instead
  of competing in one unbounded key.
- Higher context can update incrementally and be reused by kappa rather than
  recomputed at every node.
- Exact recall remains useful and measurable without being mislabeled
  attention.
- Unknown addresses fail closed; unseen ordered combinations may still be
  evaluated through bounded geometric backoff.
- The target realizes the conceptual `E8 = H4 × H4` identity through the
  required, basis/glue-bound icosian `H4 ⊕ φH4` serialization and inverse
  witness.
- Product, correctness, and reasoning claims require separate anti-recall
  evaluations and real serving evidence.

The cost is additional manifest sections, boundary-sensitive lexical rules,
scope-specific ceilings, and more explicit abstention states. Those costs are
accepted because they make global context bounded, replayable, and falsifiable.

## 10. Alternatives rejected

### One flat full-prefix route

Rejected because it either rescans an expanding prefix or collapses distinct
scope boundaries into one opaque identity.

### Kappa-only similarity

Rejected because digest proximity is deliberately non-semantic. Kappa remains
identity; factorable coordinates remain locality.

### Treat stored continuation hits as attention

Rejected because it cannot distinguish memorized recall from geometric
selection on novel combinations.

### Serialize only the words `E8 = H4 × H4`

Rejected as an implementation contract, while retaining it as the project
shorthand. The load-bearing serialized bridge is the basis/glue-bound icosian
`Z`-module construction with golden-coupled R4 points and the declared
`H4 ⊕ φH4` folding. Omitting that data would leave the conceptual identity
without canonical bytes, a rebuild path, or an inverse witness.

### Run exhaustive certification during representation discovery

Rejected. Evaluation follows the dormant, minimum-decision-bearing policy in
`docs/geometric_intelligence_evaluation.md`; release QA begins only after the
provider-free product path is ready for a release decision.
