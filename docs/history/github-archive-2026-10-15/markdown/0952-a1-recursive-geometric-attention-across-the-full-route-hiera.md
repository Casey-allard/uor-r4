# #952 — A1: recursive geometric attention across the full route hierarchy

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T23:30:07Z
- **Updated:** 2026-08-27T05:27:02Z
- **Closed:** 2026-08-27T05:26:21Z
- **Labels:** enhancement
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/952
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implement the first complete route-native attention operator. For every current lexical route and admitted next-route candidate, recursively compare the candidate against:

1. the current route;
2. the previous route;
3. the ordered last-two route;
4. the sentence route;
5. the paragraph route;
6. the conversation route; and
7. the global accumulated route.

This stage selects and orders causal support. It does not yet claim coherent generation, correctness, or reasoning.

## Load-bearing route mechanism

- Registered primes are route atoms. Semiprimes are transition experts. Ordered, multiplicity-preserving prime n-lets carry the compositional/Hamiltonian route structure.
- The zeta-ordinate grid is finite, ordered, revisioned, and fixed for an artifact. Route evolution changes winding, local phase, and retained torsion; critical-line use is an architectural assumption, not a proof of the Riemann hypothesis.
- The typed Euler/Hopf bridge uses `e^(iπ) + π^0 =_bridge 0^0`: `continuous-null` preserves the cancellation as `0`, while `discrete-empty-product` phase-shifts/retypes the seam as the discrete identity `1`; `=_bridge` is a domain-transition operator, not ordinary equality. For `S3 → S2 ⊂ R3`, `sin = ±1, cos = 0` is the signed active-magnitude class and `sin = 0, cos = 1` is the null/identity class. When `cos = 0`, tangent is not evaluated by division; a typed quarter-turn phase shift carries the sign and fiber/torsion into the next chart.
- Each hierarchy scope carries exact κ-bound child identities plus overlapping harmonic/trajectory summaries for unseen histories: session hypersphere vector, winding/window state, projection energy, shared-prime factor summary, cosine-resonance profile, and accumulated Hopf phase.
- Candidate evaluation combines exact factor overlap, divisor/prime-gap phase increments, projection energy, cosine resonance, S3/R4 transport, S2 Hopf observation, and retained fiber/torsion. Exact recall and geometric selection are separate trace types.
- Golden radial state uses exact Z[φ] coefficient pairs. Fibonacci recurrence follows from repeated φ shell steps but is not evidence of semantic value.
- The selected least-cost chart adapter is part of the route state: Euclidean `√2`, complex/discrete `2i`, or Riemannian interval `[0,2]`. These are typed coordinate/scale conventions, not literal equalities between domains. Selection requires a deterministic cost rule plus exactness or an explicit error/inverse-fidelity witness.
- The icosian representation is load-bearing: the icosian ring of quaternions over Z[φ] with the declared Turyn norm realizes the E8 lattice as a Z-module. Each 8-dimensional block is a golden-coupled pair of R4 points and the 600-cell `H4 ⊕ φH4` folding is fixed. Basis, scale, shell/orientation, coset/profile, and inverse witness are artifact-bound. Project shorthand **`E8 = H4 × H4`** denotes this paired construction. Its normative executable meaning is the icosian golden-coupled `H4 ⊕ φH4` folding, not a literal Cartesian-product identity or an identification of R4 with E8.

## Recursive attention contract

- Compute a node-local candidate delta at each hierarchy level and reuse unchanged child summaries. Do not rerun a full-prefix or full-corpus calculation for every candidate.
- Admission is causal and bounded before scoring.
- Phase/cosine/trigonometric behavior is implemented through the declared integer/table representation; there is no runtime softmax or dot-product attention.
- The hierarchy reduction order is fixed and emits per-level contributions, admissions, backoffs, and abstention.
- Exact-row hits are recall only. An unseen-history route must use the overlapping harmonic/trajectory summaries rather than masquerading as an exact match.
- Matched current-only, hierarchy-disabled, geometry-disabled, and identity-preserving permuted controls determine which terms are load-bearing.

## Identity and serving boundary

κ is canonical identity, integrity, and serialization—not a tokenizer and not a semantic distance. The lexical codec from #961 owns segmentation and reversible text mapping.

Source weights may appear only in an offline teacher that creates labels or comparisons. The serving operator contains no transformer, MoE, dense-matmul, source-attention, source weights, Ollama, hosted provider, or equivalent fallback.

Routine testing is dormant. Only the smallest decision-bearing attention probe below is authorized.

## Blockers and order

- **Future stage; unassigned.**
- Blocked by S0 #961.
- #958's bounded delivery must be reconciled into the S0/A1 schema before A1 can close; #958 itself is not rewritten or closed here.
- A1 blocks #953. #954, #955, #962, #963, #964, and #965 remain downstream in that order.

## Smallest decision-bearing probe

Use one tiny anti-recall fixture whose exact current/last-two/sentence rows are absent but whose intended next route is supported by overlapping hierarchy summaries. Score a fixed candidate set with:

1. full hierarchy geometry;
2. current-route-only;
3. identity-preserving permuted geometry; and
4. hierarchy disabled.

Require deterministic candidate order, a complete per-level trace, no future read, and a predeclared correct candidate selected only by the full recursive mechanism. If the full mechanism does not beat its matched controls or any higher scope is inert, stop and revise A1. Do not launch a corpus compile.

## Definition of done

- All seven requested comparisons are implemented on the deployed operator path.
- Node-local updates and unchanged-summary reuse are demonstrated.
- The typed chart/Euler/Hopf profile, icosian bridge, and every route/hierarchy input are transitively manifest-bound.
- Every geometric term has a matched intervention and a contribution census.
- The probe records coverage, selection/abstention, controls, artifact identities, and PASS/FAIL/NOT_RUN.
- #953 receives a frozen attention state-transition and candidate interface.

## Comments (6)

### comment 5432838748 — 2026-08-27T00:43:07Z — Casey-allard

## Planned-scope preservation note — 2026-08-26

The prior #952 body described G2: progressive all-layer replacement of source causal self-attention, gated on a `PROMOTE_TO_G2` outcome from #958. It explicitly retained the source residual/norm/MLP/LM head and uor-matmul path during transition, required per-layer controls/censuses, and prohibited a full compile before bounded qualification. No completed G2 implementation or product-capability result was claimed.

The rewritten stage preserves those gating lessons but replaces the hybrid destination with full recursive route-native attention.

### comment 5434217921 — 2026-08-27T04:03:42Z — Casey-allard

## Merged S0 consumer contract

S0 #961 is merged through PR #966 at `ae85874a621005ca96f264cece4c24e18ecc225b`. A1 is no longer blocked by S0, but remains unassigned and not started.

### Canonical artifact

Load `uor-r4.canonical-lexical-route-manifest/1` with `CanonicalRouteArtifact::decode_canonical(&[u8])`. The canonical body fields are, in wire order: `schema`, `domain`, `codec`, `vocabulary`, `lexical_route_addresses`, `content_blobs`, `spin_manifest`, `chart_profile`, `icosian_profile`, `route_records`, `paragraph_witnesses`, `global_snapshot`, `hierarchy_nodes`, `hierarchy_roots`, `shared_classes`, `bounds`, `scope_ceilings`, `hierarchy_provenance`, and `provenance`.

The parent embeds the exact frozen `uor-r4.prime-route-spin-manifest/2` canonical bytes. The child address vector remains observed-only and keeps its original validation semantics. The parent `lexical_route_addresses` table is complete for the codec and ordered by contiguous `lexical_unit_id`; parent `address_index` fields refer to this stable table, not to the child vector. Child witness indexes must be resolved independently by exact address. The child candidate cap is fixed at 8.

Codec identity is `uor-r4.unicode-lexical-runs/1`: UTF-8 bytes are identity-normalized; one unit is a maximal Unicode alphanumeric-or-underscore run or one other scalar; leading whitespace is retained per occurrence and trailing whitespace per declared sentence slice; sentence, paragraph, and turn boundaries are caller-declared; vocabulary is strict surface-byte order; every stored surface must be exactly one boundary-free codec unit; unknown units reject before state mutation. Bounds are 8 turns, 32 paragraphs, 31 sentences, 128 units per sentence, 512 total units, 256 vocabulary units, 64 KiB source bytes, and 64 global-snapshot units.

### Fixed hierarchy order

Consume exactly:

1. `current`
2. `previous`
3. `last-two`
4. `sentence`
5. `paragraph`
6. `conversation`
7. `global`

Terminal roots are `hierarchy_roots.local.current`, `.previous`, `.last_two`, then `.sentence`, `.paragraph`, `.conversation`, and `.global`. Every hierarchy node binds `previous_chain_kappa`, ordered child identity/kind/count, exact boundary kind/identity, chain identity, `exact_chain_kappa`, `summary_kappa`, bounded span, summary payload, and hierarchy provenance.

### APIs A1 must consume

- `canonical_bytes()` for canonical reserialization.
- `attention_consumer_trace()` for the terminal seven-level numeric view.
- `incremental_update_trace()` for causal event/delta identities.
- `incremental_cursor()`, then `IncrementalHierarchyCursor::apply_next()` and `state()` for bounded replay.
- `attention_consumer_trace_for_cursor(&cursor)` for seven fixed slots at every causal state. A slot is `None` until its boundary exists; no prefix artifact rebuild is required.
- `lookup_shared_class_trace(class_kappa)` for every exact matching spin/torsion occurrence without payload duplication.
- `scope_ceilings()` for admission bounds.
- `reconstruct_input()` for the complete lexical inverse.

A lexical event changes only local plus sentence state; sentence close changes paragraph; paragraph close changes conversation; global is published first and remains immutable. Maximum changed hierarchy nodes per event is 2. The final cursor trace must equal the terminal trace exactly.

### Exact `AttentionLevelTrace` fields

`level`, `identity_kind`, `identity_kappa`, `occurrence`, `turn`, `paragraph`, `sentence`, `ordinal_in_sentence`, `lexical_unit_id`, `prime`, `address_index`, `boundary_kind`, `boundary_identity`, `chain_identity`, `exact_chain_kappa`, `geometric_summary_kappa`, `previous_identity_kappa`, `ordered_child_kappa`, `direct_child_count`, `observed_descendant_routes`, `window_start`, `window_end`, `session_hypersphere_q30`, `winding_turns`, `projection_energy_q30`, `shared_prime_factors[{prime,count}]`, `cosine_resonance_q30`, `accumulated_hopf_phase_q29`, `zeta_phase_signature_q29`, `s3_spin_q30`, `s2_hopf_observation_q30`, `fiber_q29`, `torsion_q29`, `radial_zphi`, `bridge_mode`, `active_chart`, `selected_adapter`, `chart_sin_q30`, `chart_cos_q30`, `chart_activation_q30`, `chart_chirality`, `chart_cosine_polarity`, `quarter_turn_orientation`, `phase_shift_q29`, `torsion_shift_q29`, `transported_fiber_q29`, `transported_torsion_q29`, `inverse_fiber_q29`, `inverse_torsion_q29`, `chart_inverse_exact`, `payload_cid`, `address_kappa`, `shared_class_kappa`, `paired_h4_e8_coordinate_sum`, `paired_h4_e8_coordinate_kappa`, and `transported_trajectory_kappa`.

Exact route/chain identity and `geometric_summary_kappa` are deliberately separate. Digest distance is never a geometric score.

### Admission contract and witness identities

All five S0 `scope_ceilings` rows have `maximum_rows_per_query = 0`, `maximum_candidates_per_row = 0`, `maximum_retained_candidates = 0`, and `query_status = NOT_IMPLEMENTED_S0_STATE_ONLY`. A1 must supply and measure candidate admission/scoring rather than treating S0 state as attention.

Authorized S0 witness: `r4 lexical-ingestion-witness` → `PASS_REVERSIBLE_STATE_PLUMBING_ONLY`.

- artifact κ: `blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd`
- canonical byte CID: `blake3:55ee35e2301e7c262c6acab2de0c8d007f5ba7d4ad41086a15185e63d9471969` (137222 bytes)
- codec κ: `blake3:0a58577e461c70392992a6975a8988175c7b08940bf6dcf99022e66e74c13810`
- vocabulary κ: `blake3:429dffcd27639c385f24b5ec5e1456e0e1f9c0922042a46ba7eae2393e0b9f89`
- child schema-2 κ: `blake3:e874ceedebb05846976dc06ce039bc930f03371760a71ec741d71940685c6b37`
- chart profile κ: `blake3:38182fc770ce5ee5c717cd7e2ab4c54a81252dee7ce4995ab521340a8f34a0de`
- icosian profile κ: `blake3:58c5595206b9a28cdc8eb2af15053c81ae03907fc45dd1d9fdf24a14c92d42a2`
- H4 root table κ: `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76`
- icosian operator table κ: `blake3:e6c46c43f3db079e30e4d27d922502d55d13d93b9930692c453830fe677df874`
- global snapshot κ: `blake3:84698581c8a978768cbb7ab375afd24f295802275d051e62c7588a5b40e68820`

S0 establishes reversible state plumbing only. Source-free attention, coherent generation, correctness, and reasoning remain unestablished and belong to A1 and later stages.

### comment 5434437267 — 2026-08-27T04:36:50Z — Casey-allard

## A1.0 predeclared ordered-state and value-reachability contract

This contract is frozen before collecting any new A1 evidence. Baseline is live `origin/main` at `ae85874a621005ca96f264cece4c24e18ecc225b`, consuming the merged S0 artifact `blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd` through the exact interface recorded in #issuecomment-5434217921.

### Decision

Before implementing the recursive scorer, determine whether the deployed S0 consumer state can distinguish the ordered histories attention needs and whether the independently specified continuation is reachable and decodable through the real bounded candidate-index path.

### Frozen micro-population and construction rules

- One worker; smallest deterministic population that answers the decision; no corpus compile, source model, generation loop, broad QA, or future-event access.
- Register vocabulary independently of evaluation targets. Freeze construction/evaluation partition before compiling route rows, summaries, placement, coefficients, or candidate statistics.
- Use several matched contrast pairs, including same-length, same-multiset, same-current/previous/last-two-suffix histories whose earlier order differs.
- Exclude exact local, last-two, sentence, paragraph, conversation, and global continuation leakage. Exact rows must miss while overlapping summaries remain available.
- Both competing continuations must arise naturally in the same bounded pre-geometric union. Do not inject the target, hide rows after compilation, tune on evaluation cases, or hand the scorer a future event. Candidate ceiling remains at most eight.

### A1.0 measurements

1. Census every non-digest field exposed by `AttentionLevelTrace` for each matched ordered contrast. Exact/chain/summary κ inequality is identity evidence only and does not count as semantic geometry.
2. Report candidate origin, exclusions, lexical coverage, pre-admission support, post-admission support, truncation, row and candidate ceilings, backoff/abstention, and work budget.
3. Require the intended continuation to enter through the real I1/I2/IS/divisor/adjacent-spin candidate-index path rather than a fixture-supplied list.
4. Require every admitted selected address to invert through the exact S0 address-to-payload/value interface and reproduce the incremental next state.
5. If ordered state and value reachability pass, compare equal-budget full ordered geometry against current/last-only, existing additive summary, factor/count-only, deterministic operator permutation, hierarchy-disabled, exact-recall-only, and the smallest relevant paired-H4 or SpiralCore intervention. Expand level-by-level only while each added scope is causally active.
6. Any A1 artifact must have an independent κ transitively binding frozen S0, construction/evaluation partition, operator tables, candidate indexes, ceilings, controls, and payload interface. κ is used only for identity, provenance, integrity, and final deterministic tie-breaking.

### Algebra gate

- Reuse the exact signed SpiralCore 8x8 left/right action and 64-state finite group as `OPTIONAL_CONTROL_PENDING`; compile runtime composition to a finite table and do not treat 64 states as route identity or semantic proof.
- Exhaustively test whether the concrete frozen 120-root H4/Z[phi] table closes uniquely under exact binary-icosahedral multiplication, with identity, inverses, associativity, determinism, and no rounding. If this exact table does not close, stop that lane without approximation.
- The primary hierarchy fold must be associative and noncommutative with `S(A || B) = S(A) * S(B)`. A nonassociative associator may appear only as an explicit boundary/tree control.

### Terminal actions

- Ordered non-digest states collide on the required contrasts: `REDESIGN_ORDERED_ROUTE_SUMMARY`; stop before scorer implementation.
- Correct candidate absent from the natural union: redesign candidate indexes.
- Candidate present but truncated before geometry: redesign admission or the declared ceiling; do not tune energy.
- Candidate cannot invert to exact payload/next state: repair the value interface.
- Order observable but matched anti-recall semantic selection fails: `REDESIGN_ROUTE_PLACEMENT`; investigate deterministic construction-only co-occurrence placement or a separately frozen offline-teacher placement before adding geometric terms.
- Geometry ties count/identity controls or works only on exact hits: `RETAIN_STATE_ONLY`; preserve #958/#961 as storage/recall and keep #953 blocked.
- Exactly one localized reachable defect: `REVISE_A1`, naming the defect and next falsifier.
- Full recursive attention selects and decodes the independently specified anti-recall continuation, beats equal-budget controls, and every claimed hierarchy scope is causally active: `PROMOTE_TO_I1`; freeze the attention interface and only then unblock #953.
- Missing fixture, empty denominator, leakage, target injection, or unexercised branch: `NOT_RUN`, `UNAVAILABLE`, or invalid probe, never PASS.

### Budget and stop rule

The A1.0 probe is a focused in-memory one-worker run expected to complete in minutes with a finite matched-pair denominator. Static audit and the smallest focused crate checks only. Stop immediately at the first structural terminal branch; no corpus-scale expansion is authorized.


### comment 5434478600 — 2026-08-27T04:43:32Z — Casey-allard

## A1.0 frozen micro-population and partition

Before running the probe, the exact fixture is now frozen:

- Codec registration only: global singleton `gg`; independent singleton sentences registering `aa`, `bb`, `cc`, `dd`, `qq`, `uu`, `ll`, `vv`, and `rr`. Registration creates vocabulary/address identity only and is not used as construction continuation evidence.
- Construction partition only: sentences `uu ll` and `vv rr`, followed by singleton sentences `aa`, `bb`, `cc`, `dd`, and `qq`. Thus the only construction continuations are `uu -> ll` and `vv -> rr`; `qq` is registered and observed but never has a construction successor.
- Evaluation contrast 1: `[aa, bb, dd, cc, qq] -> ll` versus `[bb, aa, dd, cc, qq] -> rr`.
- Evaluation contrast 2: `[aa, dd, bb, cc, qq] -> ll` versus `[dd, aa, bb, cc, qq] -> rr`.
- Evaluation contrast 3: `[bb, dd, aa, cc, qq] -> ll` versus `[dd, bb, aa, cc, qq] -> rr`.

Every pair has equal length, the same multiset, the same current/previous/last-two suffix `cc, qq`, and identical declared boundaries. The target labels are frozen above and never enter construction rows, summaries, admission coefficients, or scorer input.

The probe must reject itself unless I1(`qq`), I2(`cc`,`qq`), and the full ordered-sentence row all miss; the exact construction union naturally contains both `ll` and `rr` through the existing bounded fallback path; the pre-admission union and retained support are exactly reported and at most eight; and both addresses invert to exact payload bytes plus a reproduced one-step incremental state. No fixture row or fixed candidate list may be substituted for the real child-manifest indexes.

The non-digest census compares all seven levels after removing κ/CID/commitment identity fields. A difference in exact chain or transported-trajectory commitments does not satisfy order observability.


### comment 5434743370 — 2026-08-27T05:23:18Z — Casey-allard

## A1.0 result — terminal representation stop

The publicly frozen A1.0 gate reached its predeclared negative branch:

`REDESIGN_ORDERED_ROUTE_SUMMARY`

- 3/3 matched contrasts collided across all 7 hierarchy levels and all 46 non-digest fields, including the plain boundary and chain identities: 966/966 field comparisons were equal.
- Exact identities differed for 3/3 contrasts, but digest distance was not used as semantic geometry.
- The real schema-2 candidate path remained reachable: all I1/I2/IS/divisor rows missed; the natural adjacent-spin row supplied exactly both `ll` and `rr`; union size was 2 under ceiling 8; no candidate was injected or truncated.
- 12/12 emitted admitted-candidate value views inverted exact address to payload; 6/6 incremental next states reproduced exactly. Attention-selected output is `NOT_RUN` because no scorer exists.
- Exact controls closed: H4 120 roots / 14,400 products / 1,728,000 associativity triples; SpiralCore 64 states / 4,096 products / 262,144 associativity triples. These are algebra controls only, with no semantic-value claim.
- The recursive attention scorer and its semantic controls remain `NOT_IMPLEMENTED_PREDECLARED_ORDERED_STATE_STOP`. Source-model, transformer, MoE, learned-router, dense-matrix, Ollama, and hosted-provider call counts were all zero.

Independent report kappa:

`blake3:23e07a17e897abb701c09354d35a3113a1b075ba1d60ee634067fa3ed3fd1904`

Frozen partition kappa:

`blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b`

Focused verification:

- `cargo test -p uor-r4-core --test recursive_geometric_attention_952 --offline -- --nocapture`: 2 passed, 0 failed.
- `cargo check -p uor-r4-core --lib --offline`: passed.
- `cargo check --bin r4 --offline`: passed.
- Exact local source-closure test: 1 passed, 0 failed.
- Workspace rustfmt, claim-wording, and diff checks: passed. The two pinned-formatter reflows needed for the live-main baseline are mechanical only.

The narrow successor is #967: preserve the natural candidate/value path, replace only the order-erasing reusable hierarchy summary with an associative noncommutative fold, and rerun this same frozen gate before any scorer work. #953 is natively blocked by both #952 and #967, so closing this issue cannot make generation eligible.


### comment 5434771091 — 2026-08-27T05:27:02Z — Casey-allard

## Protected closure

PR #968 merged through the protected queue as `9a04fe9705a16ba1de87566da9f17181e346f4d7`.

Terminal verdict: `REDESIGN_ORDERED_ROUTE_SUMMARY`.

- Independent report kappa: `blake3:23e07a17e897abb701c09354d35a3113a1b075ba1d60ee634067fa3ed3fd1904`.
- Frozen partition kappa: `blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b`.
- Ordered-state denominator: 3/3 contrasts; 7/7 levels each; 46/46 non-digest fields each; 966/966 comparisons equal. Exact identities differed 3/3 but were not used as geometry.
- Candidate/value denominator: 6/6 queries; natural union exactly `{ll, rr}`; size 2 under ceiling 8; no exact-row leakage, injection, truncation, or future events; 12/12 admitted-candidate values inverted; 6/6 incremental next states reproduced.
- Exact controls: H4 120 roots / 14,400 products / 120 inverses / 1,728,000 associativity checks; SpiralCore 64 states / 4,096 products / 64 inverses / 262,144 associativity checks. Both remain algebra controls with no semantic-value claim.
- Attention-selected output: `NOT_RUN`. Scorer status: `NOT_IMPLEMENTED_PREDECLARED_ORDERED_STATE_STOP`.
- Focused probe 2/2 and exact source-closure 1/1 passed; core/CLI checks, workspace formatting, claim wording, and diff checks passed. Queue acknowledgements were transport only and are not reported as scientific tests.

Exact successor state after closure: #967 is OPEN, unassigned, parented by #820, and now eligible because its #952 blocker is closed. #953 remains OPEN and natively blocked by open #967. No attention promotion occurred.



---
_Comment/review ids in this file: 5432838748, 5434217921, 5434437267, 5434478600, 5434743370, 5434771091_

