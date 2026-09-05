# Native geometric mechanism map — #973

This source map connects the native model to reusable mathematical and runtime
components. Its baseline is the implementation delivered through PR #1127 at
`3abf9d7e85f70416c95161863b4413cc42a6912c`; the versioned occurrence-selection
and response-state successors are distinguished below. The [project plan](integration/project-track.md)
owns the goal, and [current-state.md](integration/current-state.md) owns results
and active work. This is not another roadmap or a claim that every architectural
role is already assembled.

## Source to artifact to response

[`src/native_geometric_cli.rs`](../src/native_geometric_cli.rs) prepares
source-bound `Document` records and calls the core's count `Trainer`, separate
readout fitting and optional `MemoryReadTrainer`. The latter replays bounded
batches, restores its fitting state and accepts token-span supervision while
observing the whole causal document. Training uses floating point; exported
scores and gates are quantized.

[`training.rs`](../crates/uor-r4-core/src/native_geometric/training.rs) constructs
the lexical codec, fixed geometry and canonical `Model`, validates loaded
artifacts, then exposes `Model::generate`. That method encodes the prompt,
observes each token through `Session::observe`, repeatedly calls
`Session::predict`, commits each selected token and decodes output bytes.
[`src/native_geometric_service.rs`](../src/native_geometric_service.rs) uses the
same session operations for generation and cancellation; the
[`uor-r4-api` re-export](../crates/uor-r4-api/src/lib.rs) exposes the same core.
[`snapshot.rs`](../crates/uor-r4-core/src/native_geometric/snapshot.rs) binds
session checkpoints to the model and restores retained state by replay. The
`/5` reader's session schema `/2` also restores absolute memory geometry, sparse
posting references and its bounded captured query. That query can outlive ring
eviction; source values cannot. This is explicit response state, without learned
memory consolidation.

## Active representation and operator roles

Paths in the table are source owners under
[`crates/uor-r4-core/src/native_geometric/`](../crates/uor-r4-core/src/native_geometric/)
unless noted; named methods identify their call sites.
Costs are implementation bounds or previously recorded measurements, not new
benchmarks. Learning a score over fixed geometry is distinct from learning the
geometry, write rule or value-producing operator.

| Mechanism | Current source, state and call site | Fixed versus learned | Evidence and missing boundary | Cost and possible integration |
|---|---|---|---|---|
| Prime identities and ordered context | `training.rs::geometry` assigns complete primes; `runtime.rs::features` reads the last prime and ordered pair. The retained ring supplies ordered tokens. | Lexical pieces are corpus selected; prime assignment and the prime-to-H4 leaf map are fixed. Conditional next-token scores learn. | Native inference does not execute the explicit semiprime and general ordered-n-let types in `prime_route_attention.rs` (core source). Identity assignment is not learned meaning. | Two direct lexical feature addresses; the ring is bounded by configured context. Richer factor/ordered routes can supply reusable addressing without using digest distance. |
| Fixed zeta phases | `training.rs::geometry` consumes channels 0–7 from the revisioned zeta grid relative to prime 2 and compiles u16 turns. `Session::observe` adds/removes phases; `MemoryState::collect` forms relative phase features. | Fixed grid, token phases and quantization; learned score rows/gates consume their bins. | Modular addition is commutative. It does not independently retain order, exact analytic zeros or a harmonic field. Native measured contribution varies by artifact; the finite `/3` probe shows no added zeta benefit. | Eight additions per append and eight removals on ring eviction. Relative phase can condition learned transport/selection; new harmonic claims require declared modes and transition law. |
| R4/S3/H4 state and transport | `training.rs::geometry` imports exact H4 products/inverses. `Session::observe` composes the ordered window and removes an evicted left factor. Memory entries retain cumulative H4 poses; `/3` reads value-to-current relative pose. | The token leaf is fixed by prime modulo 120, except BOS identity. Product/inverse tables are fixed; scoring over their states learns. | The ordered fold is active causal state, but it is finite and lossy. Full continuous R4 learned value transport is not supplied by a root index. | Product table: 28,800 bytes; inverse table: 240 bytes, before other metadata. One product on append; inverse/product removal on eviction. Relative local paths can distinguish query/source relations without transporting dense vectors. |
| Hopf observation, fiber and torsion | Typed `SpinTorsionState`, Hopf observation and route machinery live in core `prime_route_attention.rs`; exact spin/icosian support lives in core `canonical_lexical_ingestion.rs`. | Existing foundational updates and chart conventions are fixed. | The active native `Session` has H4 and phase state, but no explicit Hopf/fiber/torsion fields. Earlier paragraph/conversation successes have narrow stored-descriptor scope; Hopf observation alone cannot reconstruct signed S3 state. | No separate native hot-path cost yet. These are reusable types for future state/observation operators; retained fiber and antipodal orientation must survive any projection claiming reconstruction. |
| Exact `Z[phi]` radial accumulation | `anchors.rs::compile_anchor_table` retains exact root coefficients; `Session::observe` maintains their eight-coefficient window sum and exact squared radius numerator. | Exact coefficient arithmetic and square tables are fixed; score rows over the accumulated state learn. | Unit-root norm is constant. The varying native radius belongs to the additive window carrier, not the ordered unit-root product. Additive radial state alone loses ordering. | Eight coefficient additions per append, removals on eviction, and twelve square-table reads per update. Provides exact magnitude state for an explicit learned operator without runtime multiplication. |
| Chirality, cosine polarity and heatmap | `anchors.rs` computes signed sine/cosine orientation, squared activation, projection radius and null status; `runtime.rs::features` consumes exact equality classes. | Fixed, exact class construction; conditional scores/gates learn. | Class numbers are equality addresses, never distances. Squared activation cannot replace signs; the `(q0,q1)` projection cannot replace the complete root. #970's scalar/readout negative retains this distinction. | Finite anchor/orientation lookups. Can condition signed transitions while retaining all coordinates needed for reconstruction. |
| Paired-H4/icosian representation | Core `canonical_lexical_ingestion.rs::canonical_icosian_anchor_table` supplies the witnessed bridge; `anchors.rs` retains integral coefficients, golden/Galois companion and profile identities. | Fixed reversible representation of one H4 root; native window coefficients are deterministically accumulated. | The companion adds no independent state. This is not two independently learned H4 factors or an established orthogonal Euclidean E8 isometry. Project shorthand remains `E8 = H4 x H4`. | 120 compiled anchor rows plus eight running coefficients. Independent paired operator state would need its own explicit representation, learning and inverse boundary. |
| Learned count/readout operators | `training.rs::Trainer::compile` exports conditional log-score rows/postings; `mixture.rs` fits seven score-group gates; `Session::predict` queries 26 features and merges bounded candidate postings. | Counts, row scores and gates learn; support construction, feature definitions and greedy selection are deterministic. | This is a finite-state statistical language prototype. An existing row or geometric gate does not establish abstraction, coherent output or a geometric advantage. | At most 26 feature queries, bounded posting offers and the configured output shortlist; row lookups use binary search. Gates execute through shifts/additions. The full prediction path still needs measured quality, not only low arithmetic cost. |
| Learned memory read and deterministic writes | `memory_runtime.rs::MemoryState::observe` indexes tokens following recent cues, then writes a token/pose/phase entry. `collect` scores retained occurrences; `memory_training.rs` and `memory_training/resumable.rs` learn/calibrate rows. | Read scores learn. Cue equivalence is an explicit option. Admission, posting replacement, ring eviction and writes remain deterministic. | `/3` solves the finite joint retrieval probe but fails broader generation. Retained-token copying cannot generate a computed value absent from those routes; learned result writes are missing. | Write work scales with source offsets × postings. Reads inspect no more than the memory candidate limit, with 18 score features per `/3` route. The recorded joint configuration uses context 512, query 8, source offsets 4, postings 4, candidates 128 and 209,920 bytes of optional memory storage. |
| UOR identity and artifact/session integrity | `training.rs::{refresh_identity,validate}` bind canonical model bytes, source receipts, codec, geometry and learned parameters; `snapshot.rs` rejects another model's session. | Identity and validation are fixed; learned contents change artifact identity. | CIDs and model addresses establish integrity/provenance, not similarity or meaning. A model artifact, session state and evidence report are different identities. | Loading/serialization is allocating host work. Session construction preallocates bounded state; serving operations consume immutable model data. |
| Integer/table inference and product boundary | `runtime.rs::{Session::observe,Session::predict}` and `memory_runtime.rs` are the native kernel. CLI/service tokenization, loading and rendering are host work. | Fixed executable operations act on learned integer score tables. | Focused source/allocation checks and generated outputs have executed, but their scope does not certify every host operation or establish alpha. The historical R4G1 `no_std` runtime is a separate contract. | No neural matrix products in the active kernel; candidate work and storage are configured. Reuse packed runtime components only where representation and operator semantics match. |

The `/4` occurrence-selection successor changes the read relation to local
source-cue-to-value and query-cue-to-current H4/phase paths and combines unique
features belonging to the same retained occurrence. Its purpose is to reduce
irrelevant intervening context and combine evidence for selection. It does not
implement arithmetic value construction or learned memory writes. Its executed
results and additional work/storage are in the
[occurrence-selection record](native_geometric_occurrence_selection_973.md);
the `/3` costs and outcomes above remain attached to their original artifact.

The explicit `/5` successor adds `response_runtime.rs` to the integer/table
kernel. `Session::begin_response` captures the bounded query, its H4/phase
endpoint and initial posting visits before any response token. Model scores
choose retained occurrence identity as well as token; only observing that same
predicted token commits the selected position. A mismatched teacher observation
clears the position. One optional prior, still-retained successor competes with
query routes and baseline output, using local H4/phase and learned action
features. EOS remains the baseline's learned token alternative. This keeps
duplicate equal-valued occurrences distinct and prevents generated text from
replacing the initial query. It does not compute a new arithmetic value or learn
write admission.

The first `/5` layout freezes the query endpoint. The explicit, artifact-bound
`advance_response_path` option instead transports those same captured cues to
the current H4/phase endpoint; admission and storage bounds remain identical.
Its fit is a separate retained negative. Neither fitted layout improves the
matched `/4` result, and observed continuation counts are zero and one
respectively. Query persistence does not yet establish useful composition.

The flat route bound becomes `candidate_limit + 1`, with at most 18 features
per route before occurrence union. Captured queries and posting visits are
allocated once, bounded respectively by `query_tokens` and `candidate_limit`.
`ResponseStateDisabled` removes capture and continuation while retaining the
same artifact's ordinary reader. The resumable fitter selects occurrence state
from its own frozen quantized rollout before each observation, including
unsupervised context within a response. The
[response-state record](native_geometric_response_state_973.md) binds the
executed behavior, costs and checkpoint checks; mechanical continuity alone
does not establish useful answer composition.

## Historical pieces and what they can contribute

| Preserved piece | Reusable function | Established limit |
|---|---|---|
| [Ordered-summary #967](associative_ordered_route_summaries_a1r_967.md) and [heatmap #970](candidate_relative_identifiability_a1p_970.md) | Exact noncommutative folding and diagnostics for information lost by a readout. | Ordered states separated while the scalar readout tied; heatmap classes later failed transfer/identifiability. These reject those readouts, not all ordered or geometric state. |
| [Recurrent/sparse/nonlinear checkpoints](integration/project-track.md#historical-mechanical-checkpoints-through-pr-1124-2026-09-04) | Fixed recent records plus age-banked summaries, metadata selection before value gathering, and finite H4-indexed nonlinear R4 action. | Mechanical bounded-state/operation results; no fitted useful assembled model. The old continuous/dense/Python path is reference evidence, not the native product or a required next rung. |
| [Retained-language and write/read experiments](r4_retained_language_path_v1_973.md) | Causal retention, matched state interventions and separation of write, binding and readout causes. | Useful retained state did not ensure prompt coherence; later write/read laws had scoped capacity or attribution negatives. Importing their numeric results does not validate a new native law. |
| [R4G1 route attention](MODEL_LIFECYCLE.md#r4routeattentionv1-604), packed graph format/runtime and compiler | Bounded masked-XOR/popcount selection, fixed-point score aggregation, borrowed immutable data, validation and deterministic serialization. | Dormant operator/synthetic fitting evidence and historical compiled-model contracts; no automatic language, geometric-only intelligence or alpha claim transfers. |

Selective geometric routing could restrict which retained states or operators
participate in a decision. Multiscale state could carry useful information past
the exact window. These are hypotheses resembling some functions of sparse
attention and MoE systems, not evidence of expert specialization or semantic
abstraction. A concrete implementation must name what survives compression,
what its learned selection consumes, the operator applied, information lost and
the complete execution cost. An operator that creates a value also needs a
typed result and causal write/read path; a better copy selector alone cannot
supply it.

The [recovery](native_geometric_recovery_973.md),
[query-context](native_geometric_query_context_973.md) and
[resumable-fitting](native_geometric_resumable_memory_973.md) records bind the
earlier native measurements; the
[occurrence-selection record](native_geometric_occurrence_selection_973.md)
adds the matched corrected-source `/3`–`/4` comparison. The historical `/3`
reader reached 96/96 prose and 96/96
Rust cases derived from 12 development worlds; H4/zeta-disabled controls also
passed. Broader streaming variants reached 6/32 or 7/32 prose and 0/32 Rust
exact outputs, with failed compilation/repair examples retained. These results
separate useful bounded retrieval, reachable-but-misselected answers and
missing value computation; they do not qualify either alpha capability group.
