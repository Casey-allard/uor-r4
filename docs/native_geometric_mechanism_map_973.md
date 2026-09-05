# Native geometric mechanism map — #973

This source map connects the native model to reusable mathematical and runtime
components. Its baseline is the implementation delivered through PR #1127 at
`3abf9d7e85f70416c95161863b4413cc42a6912c`; the versioned occurrence-selection,
response-state, typed-value, completion, response-entry and retained-word
successors are distinguished below. The [project plan](integration/project-track.md)
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

The optional typed-value component adds `numeral.rs`, `value_runtime.rs`,
`value_training.rs` and `value_snapshot.rs` to this same model. It uses the
stronger `/4` fitted reader and reuses `/5`'s prediction/observation separation
with distinct derived-write identity. A generic signed-integer byte lexer
records numeric payloads, up to four preceding prime cues and cumulative
H4/phase state. These payloads are separate from token-root coefficients.
At a response boundary, the learned integer score table chooses Copy, checked
`ZPhi::checked_add` in the integer subdomain, or no derived write. It examines
up to sixteen records and 256 valid operand/action proposals. Relative H4,
fixed-phase, prime-cue equality and occurrence-rank features influence selection.
This is bounded exhaustive operand scoring, without sparse preselection.

Typed schema `/2` additionally uses `value_lexemes.rs` to preserve complete
ASCII word identity across tokenizer pieces. Sixteen recent words and sixteen
captured query words are bounded to 32 bytes each; each numeric record retains
four preceding words, source endpoints and their geometry. Learned query-position
features consume exact source/query word-match masks. Word geometry is retained
metadata, not a new scoring law. This preserves selected local identities while
discarding most surrounding text; it supplies no entity-role parser, assignment
graph, learned consolidation or multiscale semantic summary. Its 16 extra features
per proposal can add up to 32,768 word comparisons and 1,048,576 byte comparisons
at the sixteen-value limit, in addition to the existing path. Every admitted
addition still executes before selection; sparse compute savings remain open.

Only observing the selected decimal token commits the derived value and its
operand IDs/values. A fixed twenty-byte buffer emits the remaining digits
through the ordinary shortlist. Positive selection emits at the response
boundary; the current gate cannot wait through generated text before inserting
a numeral. Typed schemas `/1` and `/2` alone supply no response suffix or learned
stop classifier; the separate completion component below adds that operation. Raw
prompt/response training labels complete numeral bytes and leaves operand and
action identities latent. Canonical lexical tokens and emitted byte tokens can
differ while decoding to the same text.

The sixteen-record store can retain numbers beyond ordinary token-ring
eviction, including persistent derivation, but deterministically discards old
records when full. Most surrounding source text and general assignment/dependency
structure are not retained. Session schema `/3` validates this explicit state;
older source payloads are user-provided state, not authenticated history.
The [typed-value record](native_geometric_typed_value_973.md) supplies complete
cost bounds, direct behavior, controls and remaining binding/emission limits.

## Committed-value completion — 2026-09-05

The optional `uor-r4.native-value-completion/1` component adds
`completion_types.rs`, `completion_runtime.rs`, `completion_training.rs` and
`completion_snapshot.rs`. It preserves the fitted `/4` reader and typed `/2`
parameters. A completion anchor exists only after the final predicted numeral
byte has actually been observed and the derived write committed. It retains
the write identity/action, endpoint H4 pose and eight phases, frozen query-end
prime, last two actual token identities and at most 32 observed suffix steps.
It does not retain a suffix template or whole-answer key. Prediction is
transient; observed bytes alone advance the state, including mismatches.

Sixteen feature addresses combine prime/token history, action/progress,
relative H4, signed orientation and phase-difference bins. Construction-only
postings admit byte/EOS candidates; signed integer scores select a positive
candidate against zero-score Base. Rust fitting learns these next-byte scores
from actual frozen `/2` numeral rollouts and raw response suffixes plus EOS.
It rejects upstream incorrect, noncanonical or incomplete numerals instead of
teaching suffix repair. NoWrite cases never activate completion. Session schema
`/4` validates the committed anchor and actual suffix history; schemas `/1`–`/3`
and absent-component model serialization retain their earlier laws.

The first fit used the existing 192 raw pairs: 160 matched numerals, 32 skipped
NoWrite cases, no upstream failures, and 720/720 correct exported fit positions.
On the same reused open development, Full produces 12/16 exact prose and 12/16
exact Rust responses: all 24 numeric responses complete correctly. The eight
NoWrite responses remain incorrect. Completion-disabled retains the earlier
2/16 prose and 0/16 Rust exact results with 24/24 correct leading numerals.
Suppressing only completion geometry gives 0/16 exact in each family while
preserving the same six completion candidates. This is within-artifact feature
sensitivity, not a separately fitted comparator or general geometric advantage.
The typed-only negative and both `/5` negatives remain preserved evidence.

The current local artifact is
`/Users/casey.allard/uor-r4/.uor-models/native-typed-value-2026-09-05/completion-fit-1/model.json`,
CID `blake3:a1fa0314924fb324f994e449cce6e69793d6c4df6102353a959363cb766009ff`.
Its preserved baseline is `fit-2/model.json` under the same root, CID
`blake3:af5f892e7f10680266911e2f5f6fb0aa96a5f25b1894d2f191f0a6b1179843f5`.
Sibling `fit-report.json` and `report.json` bind the fit and generated outputs;
the [completion record](native_geometric_value_completion_973.md) and
[compact evidence](evidence/native_geometric_value_completion_973.json) retain
their measured scope and controls.
Exact authored numeric completions do not establish general conversation,
dependency execution or Rust coding capability; compiler/execution evidence
is recorded separately in [current state](integration/current-state.md).

Added serving work per active completion prediction is bounded by 16 row
queries, 80 posting offers, 1,280 candidate equality comparisons, 16 candidates
and 256 token-score lookups. At the format maxima of 4,096 rows and 257 byte/EOS
IDs per row, binary searches use at most 13 row comparisons and 10 token
comparisons per lookup. Features add two H4 table reads, one orientation read
and eight phase subtractions. The fitted head has 192 rows/289 associations;
observed completion state occupies 96 bytes on this host. Fixed feature,
candidate and row-index buffers also occupy stack space. Observation/history
copies, ordinary `/4` prediction, numeric scanning and `/2` selection remain
additional work; counters name operations, not every machine instruction.
There are no runtime floats, matrix products or steady-state allocations in
this component. Fitting, serialization and initialization are separate host work.

## Preceding learned response entry — 2026-09-05

The optional `uor-r4.native-response-entry/1` component in
[`response_entry_types.rs`](../crates/uor-r4-core/src/native_geometric/response_entry_types.rs),
[`response_entry_runtime.rs`](../crates/uor-r4-core/src/native_geometric/response_entry_runtime.rs)
and [`response_entry_training.rs`](../crates/uor-r4-core/src/native_geometric/response_entry_training.rs)
addresses a different entry condition from the committed-numeral head. The
preceding numeric artifact has eight incorrect NoWrite development responses,
where no derived write exists and numeric completion cannot anchor. This
implementation's [record](native_geometric_response_entry_973.md) reports all
eight repaired responses, preserved numeric behavior and four content-transfer
failures. Exact development response forms do not establish grounded abstention
or arbitrary identifier binding.

At an explicit response boundary, the state captures the absolute observation
count, cumulative typed H4 pose, eight fixed-zeta phases and final query-cue
prime. It retains the last two actual token IDs, actual observation count,
progress count, active flag and latest action. This origin is a response
boundary, not a numeric derivation or an invented value record. Eligibility
requires active typed state, at least one captured numeric source and a
nonempty query. The typed operator runs first; a positive typed proposal or
committed numeral emission prevents response entry from offering a token.
Consequently the new component does not yet serve arbitrary prompts without
retained numeric sources, or decide when to insert a numeral after prose.

The first selected non-EOS entry token commits `Enter` only when that exact
prediction is observed at the captured boundary. Repeated prediction cannot
activate or advance the response. An initial Base or mismatching observation
closes entry until the next explicit response boundary. After entry, actual
observations advance the retained history and progress, including mismatches;
these do not create target-selected provenance. Later learned actions are
`Emit` and `Stop`, with zero-score Base retained. EOS closes the state.
Thirty-two actual response observations deactivate the component without
forcing EOS. Empty continuation preserves an active response; new input
establishes a new boundary through the existing session callers.

Sixteen feature addresses contain bias, last-token prime, ordered last-two
primes, captured query-end prime, query/last-prime pair and progress, plus
`inverse(boundary.pose) * current_typed_pose`, separate signed orientation
and eight upper-four-bit wrapped phase differences. Entry kinds 0–15 and
continuation kinds 16–31 occupy disjoint rows. The exact token identities and
bounded endpoint state survive; arbitrary source text, syntax/dependency
structure, a full response transcript and individual earlier phase terms do
not. Finite H4 state and quantized phase features compress history without
establishing semantic abstraction. Prime and content identities remain
addresses/integrity, never numeric semantic distances. Existing exact
`Z[phi]`, radial and paired-H4/icosian state retains its previous roles.

The new head learns **canonical lexical-token/EOS** associations. It can
therefore preserve learned lexical pieces as single decisions, with ordinary
byte fallback for other text. The earlier committed-value head learns
**byte/EOS** associations after a causally emitted numeral, whose exact digits
are fixed byte tokens. These laws share the model codec and ordinary decoding
but have distinct boundaries and artifacts. In particular, the 49-byte NoWrite
prose target exceeds the existing 32-byte completion horizon; response entry
uses its canonical token length explicitly. Fitting skips a whole example
when its token sequence plus EOS exceeds 32 or would exceed the remaining
position budget. It neither truncates the target nor appends a template.

Training reuses the sparse optimizer extracted from
[`completion_training.rs`](../crates/uor-r4-core/src/native_geometric/completion_training.rs).
First it collects actual eligible boundary frames and learns entry scores,
with Base excluded as a correct state-creating action even when Base already
predicts the target token. It exports quantized first-step rows, executes those
rows through ordinary selection/observation, and collects continuation frames
only after a matching selected Enter actually commits. Subsequent authored
tokens supply declared teacher forcing through actual observation. Continuation
rows learn separately while the entry rows remain fixed. The final merged
global postings are followed by actual first-entry and complete free-running
checks. Reports distinguish numeric upstream verification, NoWrite examples,
whole-example skips, candidate coverage, entry failures, phase-specific fitting
and final responses. The five configuration words bind epoch budget, exact
learning-rate bits, position cap and both selected epochs. The component's
baseline identity binds the unchanged numeric completion model and all its
preceding typed/memory/readout state.

Both heads share bounded candidate gathering and integer score reduction in
[`completion_runtime.rs`](../crates/uor-r4-core/src/native_geometric/completion_runtime.rs).
Per eligible prediction, response entry performs at most sixteen row queries,
80 posting offers, 1,280 duplicate-token comparisons, sixteen retained candidate
writes/evaluations and 256 score lookups. At 4,096 rows, row lookup uses at most
13 comparisons; the 32,768-association artifact cap bounds each token lookup
by sixteen comparisons, hence at most 4,096 score-token comparisons. Features
add two H4 reads, one orientation read, eight wrapped phase subtractions and
three token/row-base metadata reads. These are conservative source bounds,
not latency measurements. The fixed feature/token/row-index arrays occupy
448 logical bytes on a 64-bit host; StateView separately reports the actual
persistent state layout. Boundary capture copies three scalar fields and eight
phases; every observation updates the three history scalars. Pending decisions,
control branches, resets, shortlist insertion and all model tables remain
additional cost beyond those array sizes and named counters.

The work record uses the shared `CompletionWork` shape. Its entry `anchors`
counter measures eligible boundary captures, which can precede numeric
preemption; actual learned entry is identified by the committed `Enter`
action. All ordinary decoder, `/4` occurrence routing, typed ingestion,
operand/action scoring, checked additions and numeric-completion work still
execute at their existing scopes. The final runtime reuses captured NoWrite
after Enter commits while the response stays active. Sources, query tokens/words
and model/control remain fixed; current pose/history and ordinary score cannot
turn that absent proposal positive. EOS, cap and new boundaries restore the
search; restoration rechecks its origin. The record measures this same-artifact
avoidance of repeated failed operand searches separately. It is not a sparse
first-pass arithmetic router, whole-model latency advantage or transformer
comparison. Loading, canonical tokenization, rendering, training and snapshot
validation are allocating host work and belong in complete cost accounting.

[`response_entry_snapshot.rs`](../crates/uor-r4-core/src/native_geometric/response_entry_snapshot.rs)
validates the response-boundary origin, retained actual history, progress and
geometry in session schema `/5`. It rechecks the original typed NoWrite and
learned first-entry selection using captured state, and rejects a fabricated
entry or transient pending prediction in persisted data. The active span fits
the existing 32-entry typed metadata ring before the cap clears its origin.
Older source truth after eviction remains unauthenticated. Absence of this
component preserves the prior model and session serialization laws.

## Retained-word entry and completed-word suffix — 2026-09-05

The response-entry `/2` extension in
[`word_copy_types.rs`](../crates/uor-r4-core/src/native_geometric/word_copy_types.rs),
[`word_copy_runtime.rs`](../crates/uor-r4-core/src/native_geometric/word_copy_runtime.rs)
and [`word_copy_training.rs`](../crates/uor-r4-core/src/native_geometric/word_copy_training.rs)
connects an actual retained spelling to the response. Its
[measurement record](native_geometric_word_copy_973.md) separates the preceding
entry-boundary suffix variant from the optional completed-word suffix repair.
The source laws below do not establish either variant's behavioral result.
The extension binds the exact entry `/1` parent; removing it and restoring the
parent schema must reproduce that parent identity. Existing ordinary readout,
memory `/4`, typed `/2`, numeric-completion and inherited entry parameters are
preserved.

The source payload is the existing sixteen frozen `WordAtom` occurrences from
[`value_lexemes.rs`](../crates/uor-r4-core/src/native_geometric/value_lexemes.rs).
Each carries up to 32 exact ASCII identifier bytes, length, source byte and token
endpoints, H4 pose and eight phases. Equal spellings remain distinct occurrences.
Fixed scanner exclusions and oldest-word eviction remain; punctuation,
intervening text, arbitrary older words and a declaration/dependency graph are
not retained in these atoms. This adds no second context store. A dictionary of
at most 256 construction spellings is selected by frequency with exact-byte
ties, sorted by bytes and assigned canonical prime addresses. An unknown
neighbor receives address zero. These are equality addresses, not semantic
distances; dictionary exclusion does not remove the exact copy payload.

At the first eligible NoWrite boundary, learned scalar rows rank up to sixteen
word occurrences against inherited lexical entry and the same ordinary Base.
Copy must improve strictly on both. Candidate ties retain the first occurrence
visited, in newest-first order. Twenty feature slots contain captured token cue
primes and their ordered pair, occurrence rank, preceding/following word
addresses and pairs, length/missing-context flags, local H4 relation, signed
orientation and eight relative phase bins. The candidate's own dictionary
address is absent; its length and geometric endpoint can still affect scores.
Unknown words are therefore copyable, without a spelling-invariance guarantee.

For preceding/candidate word poses `P,C` and the last two frozen query-word poses
`A,B`, the fixed transport is `S = P^-1 C`, `Q = A^-1 B`, then `S^-1 Q`.
Corresponding phase intervals are subtracted in wrapping u16 arithmetic before
four-bit binning. Missing endpoints omit those features. Shared lexical-token
endpoints can erase a word-level geometric distinction even when byte endpoints
remain distinct. The finite H4 class and quantized phase bins compress history;
they are not a parsed role or a general semantic summary. The learned operation
is finite occurrence selection. Exact byte copying is the fixed operator it
selects. Numeric records and a nonempty captured query remain prerequisites;
copy is not admitted after generated prose or whitespace in this increment.

Only observing the selected first byte commits the occurrence. Repeated
prediction does not advance it. Subsequent byte offers use the ordinary winner
plus one while inherited lexical-entry scoring is suspended. Matching
observations advance the cursor; a mismatch marks it Aborted and resumes
inherited actual-history continuation. Immutable first-entry provenance remains
until EOS, a new boundary, ineligibility or the cap clears it. Complete copies
use separate learned suffix rows, with ordinary Base still available. The
32-observation cap rejects a whole copy if its bytes cannot leave room for EOS;
a retained 32-byte word is consequently inadmissible here. Output observations
update ordinary token/geometric memory but do not mutate frozen word or numeric
source inputs. They create no second numeric derivation, preserving the existing
NoWrite reuse invariant.

With artifact-bound `completed_word_suffix: true`, the suffix origin is derived
from the actual final copied byte in the existing 32-entry typed metadata ring.
Its sequence is the original response boundary plus copied length minus one;
checked bounds and the actual endpoint token must agree. A temporary entry frame
uses that observed H4 pose and phases, retains the original query prime, and
counts only subsequent suffix observations. The last two suffix tokens are exact;
missing suffix history uses BOS. The original selected occurrence, boundary and
actual history remain stored. Identifier bytes, their length and the earlier
response path are excluded from suffix progress/history features, rather than
deleted from state. Relative H4 and phase differences cancel the copied-prefix
frame. This is a fixed change of origin for learned suffix scores, not learned
multiscale consolidation. The default false law retains the preceding
entry-boundary features and omits the flag from artifact serialization.

Offline targets mark all complete matching initial-word occurrences as latent
positives; no target-derived index is serialized. A non-identifier initial byte,
including the existing leading-space Unknown form, supplies NoCopy supervision.
An initial identifier absent from retained words is unreachable and skipped,
not an abstention label. Suffix frames require actual quantized selection and a
complete observed copy. Runtime and fitting use the same continuation-feature
method; overlong fitting trajectories are skipped whole. These boundaries keep
learned content selection separate from authored supervision and fixed emission.

Conservative initial bounds are 320 scalar feature queries and 4,160 row-key
comparisons at 4,096 rows; sixteen dictionary lookups add at most 144 dictionary
comparisons and 4,608 byte comparisons. Because the oldest of sixteen words has
no retained predecessor, full support tightens these to 310 features, 4,030
row-key comparisons, 62 H4 reads, fifteen orientation reads and 248 phase
subtractions. Scalar scores require integer additions and comparisons, not a
runtime matrix product. Once copying commits, bytes require no occurrence
rescoring. Suffix selection uses at most sixteen features/candidates in separate
rows. The completed-word frame additionally reads one source-word record, its
final byte and one to three actual-history metadata entries, and constructs a
temporary entry state. It adds no persistent anchor or heap allocation.

Ordinary features, postings, shortlist insertion, enabled `/4` reads, actual
byte observation and the first typed operand search remain additional work.
The typed path can still execute up to 240 checked additions before ranking;
word copying does not provide sparse arithmetic dispatch. `WordCopyWork`
distinguishes logical word-record reads, dictionary/byte comparisons, selected
state assignments and rejected length bounds. Row-base accesses share metadata
counters; successful bound checks and compiler-generated memory traffic are
not a complete separate census. The reported copy-state layout includes its
transient pending-decision reservation, not just serialized provenance. Existing
word storage, enlarged Session/Work members, model tables, fixed local arrays,
host traces, loading, tokenization, fitting and serialization remain separate
storage/work scopes. Counted reductions are not latency or transformer-efficiency
measurements.

[`word_copy_snapshot.rs`](../crates/uor-r4-core/src/native_geometric/word_copy_snapshot.rs)
uses session schema `/6` to verify captured provenance, observed byte prefixes,
original NoWrite and the combined first selection. Transient predictions are
rejected. The optional suffix frame is reconstructed from that existing state;
older source truth remains unauthenticated. Copy-disabled bypasses the copy
selector and leaves inherited entry active. Copy-geometry-disabled removes
geometry from both the new selector and its suffix head while retaining
inherited geometry and word admission.
It measures combined within-artifact sensitivity, not a separately fitted
geometry-free comparator or a selector-only advantage. Different trajectories
can incur different complete work. No general declaration-role, multiscale
semantic-memory or alpha claim follows from this implementation.

## Historical multiscale source inspection for response entry

The historical pieces below remain unchanged evidence. Source inspection found
reusable update/admission arrangements, with different payload and serving
contracts from the active response-entry problem:

- [`fixed_recurrent_kv_binding.py::_merge_local_summaries`](../tools/r4-softmax-trainer/src/r4_softmax_trainer/fixed_recurrent_kv_binding.py)
  transports older K/V into the newer H4 frame and takes a count-weighted mean.
  `_fold_evicted` uses four binary-carry summary banks beside eight exact live
  records; its final bank absorbs overflow. Means, counts, latest frames and
  latest positions survive, while individual evicted K/V values/identities and
  detailed within-bank order cannot be reconstructed. This fixed compression
  does not learn semantic consolidation. Original dense Q/K/V/O, softmax and
  continuous state remain part of the computation.
- [`sparse_geometric_kv_binding.py::_rank_candidates`](../tools/r4-softmax-trainer/src/r4_softmax_trainer/sparse_geometric_kv_binding.py)
  examines the twelve-slot metadata directory, orders exact signed-S3 shells,
  applies greedy maximin separation and age/slot ties, and admits at most eight
  persistent slots. Only afterward does `_selected_r4_attention` gather K/V
  and execute unchanged learned Q/K softmax. The gate itself is unfitted. This
  supplies an admission-before-expensive-gather pattern, but no learned query
  semantics, numeric result path or response-entry/termination operator.
- [`quaternion_cube_nonlinear.py::_post_attention_nonlinear`](../tools/r4-softmax-trainer/src/r4_softmax_trainer/quaternion_cube_nonlinear.py)
  applies a framed quaternion-cube residual to twelve R4 cells. It retains
  floating-point frame products, reciprocal and RMS normalization. Its
  mechanical dense-MLP replacement and preserved block norms did not preserve
  useful generated text; its attempted fit stopped on resources before a new
  quality result. It does not solve missing learned response entry.

These sources informed integration choices, rather than being copied into the
new Rust serving path. Reusing the existing integer candidate scorer, actual
query boundary and causal observation interfaces directly addresses the
observed NoWrite entry/continuation gap. A future multiscale retention change
still needs a stated payload, loss/retention law, learned query selector,
operator and complete cost; no new multiscale capability is asserted here.

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

The inspected sparse-expert reference makes the ordering concrete:
Mesh TensorFlow's [`_switch_gating`](https://github.com/tensorflow/mesh/blob/master/mesh_tensorflow/transformer/moe.py#L1200-L1323)
learns input-to-expert scores, selects one expert and applies capacity filtering
before [`transformer_moe_layer_v1`](https://github.com/tensorflow/mesh/blob/master/mesh_tensorflow/transformer/moe.py#L405-L534)
dispatches inputs and executes dense experts. Overflow skips that expert branch;
extra capacity adds padded computation, memory and communication. The router,
dispatch, combine and padding costs remain part of total work. See the
[Switch paper, sections 2.1–2.2](https://jmlr.org/papers/volume23/21-0998/21-0998.pdf).
Our `/4` and completion postings restrict later candidate scoring; `/2` executes
up to 240 checked additions before ranking, so its gate currently avoids no
operator arithmetic. These native heads are not MoE expert networks. A future
geometric gate could select operands/operators before expensive execution, but
must preserve reachability and count its own cost. Sparse dispatch also does
not by itself specify what a multiscale context summary retains or loses.

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
