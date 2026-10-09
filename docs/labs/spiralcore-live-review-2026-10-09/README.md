# Hosted SpiralCore v69: exact E8 actions and the native learning boundary

## Question, artifact and decision

Owner request: review the freshly hosted SpiralCore implementation and determine whether its geometry-preserving routes can consolidate UOR-R4's geometry, Hamming encodings and learning processes. This follows the [XOR/Hamming document review](../nemesis-xor-review-2026-10-09/README.md); it corrects any interpretation that SpiralCore lacks an actual E8 action graph.

**KEEP** the exact finite-action machinery and typed identity/action/observation separation as consolidation inputs. **NOT YET PROMOTED** as a native model integration or remedy for the protected Prefix/Generate conflict. No runtime, learned artifact, accepted 8/512 complete-development-reply result or milestone status changes in this review.

[Hosted source](https://spiralcore.netlify.app/) fetched 2026-10-09, HTTP Date 20:35:02 GMT, title “SpiralCore v69 — Geometric Instrument and Scores,” 1,025,707 bytes, SHA256 `d976e34d93d3446c030efbb14058cd31f58927a660f5b58811e17d2b9f80a274`, ETag `"aa0f4f5a038aed04d10477c1302a7918-ssl"`. The original HTML and fetch identity are retained as results, not adopted repository instructions. Source line references below refer to that exact downloaded file. Current UOR comparison is main `2636bb2e2a67b65212672f410c692e85b402481f`.

Two expert subagents inspected the source independently by role: mathematics (geometry/action/encoding) and engineering (native callers and credit boundary). They did not execute new model or Rust tests. Parent additionally exercised the hosted browser controls. This is source review plus bounded UI observation, not an independent exhaustive reproduction of the website's fixture suite.

## What the hosted source actually computes

| Boundary | Exact source | Finding |
|---|---|---|
| Intrinsic roots | `e8Roots`, line 2602 | 112 integer roots with two nonzero ±1 coordinates plus 128 even-parity half-integer sign roots; 240 roots in R8, squared norm 2. |
| Operators | `cl06Bivectors`, line 2767 area | Six octonion left-action generators produce 15 Clifford bivectors; signed coordinate-permutation actions. |
| Root action | `e8ApplyOperator`, line 2897 | Applies the actual 8×8 matrix to an actual root and resolves its resulting coordinates to a root ID. |
| Routing | `e8OperatorRoute`, line 2947 | BFS through labelled action edges; rejects distinct operator orbits. |
| Inverse | `e8InverseOperatorSequence`, line 2981 | Reverses operator order and uses each bivector cubed, since B²=−I. |
| Implemented exhaustive checks | `checkE8OperatorFixtures`, line 3076 | Root-set preservation, operator bijections, 15 orbits of 16, stabilizers of 4, within-orbit replay and inverse round trips. |
| Core route freshness | `rebuildAdjacency`, line 2503; `coreRouteFreshness`, line 5750 | Separate core graph has an adjacency epoch and replays stored hops against live edges. |

The source distinguishes 3,600 labelled root×generator records from 2,580 distinct directed endpoint pairs. Their difference, 1,020, is label multiplicity: multiple operators may share endpoints. Retaining the action label matters for composition and inverse replay. Hamming distance is not the BFS route-selection rule.

For a signed permutation S, SᵀS=I, hence (Sx)·(Sy)=x·y and ‖Sx−Sy‖=‖x−y‖. This establishes the relevant geometry-preserving property of the same action applied to both arguments; products retain it. It does **not** imply decreasing distance from a moving root to a fixed target. BFS minimizes hop count within this declared action graph, not Euclidean length or all possible E8 transformations. Cross-orbit rejection describes this subgroup's reachability, not impossibility under every E8 symmetry.

The root construction and E8 route APIs also occur in the preserved [v68 source](../../../research/spiralcore-v68/Spiralcore_Dodecahedron_IP_Schema_v68.html). Do not call all of this newly introduced in v69. v69's change-control primarily adds the shared instrument and reversible file-score interface; audio/file recovery was outside this review.

## Browser observations and their scope

On the live v69 page, the browser displayed its own **256/256 fixture-family** health badge. That is reported application health, not 256 independently rerun tests by this review.

Using Network & Geometry → E8 operator routing:

- Source 0, destination 1: route **0→86→1**, two directed hops, subnet 0; operator labels AXIS2 2 / B45 then AXIS2 1 / B46.
- Source 0, destination 4: **ROUTE REJECTED**, explicitly naming subnet 0 versus subnet 1 and orbit preservation.
- Returning to 0→1, Witness Lab → Measure route: **INTRINSIC E8_8D**, dimension 8, two nondegenerate segments, total length 4, trace(Q)=4, trace error 0, eigenvalues(3,1,0,0,0,0,0,0), directional rank 2, effective dimension 1.6.
- The phase 0 projection display separately reported rank 3, kernel dimension 5, 37 collision groups covering 236 roots, 958 collapsed pairs of 28,680, normalized stress 0.337433304. These are phase-specific browser-reported observables. The route witness explicitly used canonical 8D coordinates, not that projection.

Thus the browser exercises an actual intrinsic action route and an explicit invalid-domain rejection. It does not qualify model learning, audio round-trip recovery, full UI correctness or D11 serving. No browser file upload, paid compute, model inference or fitting occurred.

## Where Hamming is exact, and where the basis matters

For the 128 raw half-integer roots xᵢ∈{−½,+½}, the eight sign bits satisfy

    squared Euclidean distance(x,y) = Hamming(signcode(x),signcode(y)).

Each changed sign contributes exactly 1. On the unit-normalized copy, the squared distance is Hamming/2. A shared signed coordinate permutation induces a bit permutation plus a fixed XOR mask, preserving Hamming distance. This is a concrete reusable encoding contract for that family.

The 112 integer roots also contain zeros and ±1; sign bits alone cannot faithfully encode their support and magnitude. Root IDs are arbitrary enumeration labels and XOR/popcount of those IDs is not this geometric metric. The 128 even-parity half-root sign words are also distinct from the 16-word extended [8,4,4] Hamming code in a Construction-A presentation of E8. The latter additionally requires its integer lifts and normalization; parity code alone is not the complete lattice state.

UOR already has related but different domains:

- [Rust SpiralCore adapter](https://github.com/UOR-Foundation/uor-r4/blob/2636bb2e2a67b65212672f410c692e85b402481f/crates/uor-r4-core/src/spiralcore_operator.rs#L1): oriented octonion/Fano convention, signed matrices, finite 64-state composition/inverse. Its declared chart transport remains unestablished; optional control is not current learned emission.
- [E8 codebook](https://github.com/UOR-Foundation/uor-r4/blob/2636bb2e2a67b65212672f410c692e85b402481f/crates/uor-r4-integer/src/codec.rs#L1280): zero plus 240 scaled-by-two standard roots. IDs, scale and null handling must be matched explicitly before sharing tables.
- [Icosian coefficient map](https://github.com/UOR-Foundation/uor-r4/blob/2636bb2e2a67b65212672f410c692e85b402481f/crates/uor-r4-core/src/canonical_lexical_ingestion.rs#L2788): exact golden/Galois coefficients with inverse witness; explicitly not an orthogonal Euclidean E8 embedding. The native prototype compiles and uses these coefficients, while current M2 Prefix transport uses artifact-bound H4 relations.
- [Directed Prefix transport](https://github.com/UOR-Foundation/uor-r4/blob/2636bb2e2a67b65212672f410c692e85b402481f/crates/uor-r4-integer/src/geometric_prefix_transport.rs#L84): ordered relative H4 state, not an undirected distance or E8 route ID.

If x=T c maps coefficients c into Euclidean coordinates, the coefficient metric is G=TᵀT and the transferred action is A=T⁻¹ST. Check AᵀGA=G and lattice/admitted-state closure. Applying S directly to c merely because both have eight entries is unjustified. The hosted page's E8 roots are constructed separately from its H4 icosians; it does not supply this missing icosian basis bridge. Preserve chirality, orientation, fiber and occurrence/version identity through any proposed map.

## Consolidation that addresses the current blocker

A shared typed action contract should bind state domain, basis/metric, canonical IDs, operator labels, composition/inverse, allowed transitions and replay freshness. Existing Rust finite tables and the E8 codebook are plausible reuse points. This is a specific adapter opportunity; it does not justify adding another geometry backend or erasing the H4/E8/icosian distinctions.

The immediately justified learner consolidation is more direct. In [coupled learning](https://github.com/UOR-Foundation/uor-r4/blob/2636bb2e2a67b65212672f410c692e85b402481f/crates/uor-r4-training/examples/geometric_frozen_map_fit/coupled_episode_learning.rs#L2445), 31 objective backwards happen before preparation of 380 protected rows. The [guard loader](https://github.com/UOR-Foundation/uor-r4/blob/2636bb2e2a67b65212672f410c692e85b402481f/crates/uor-r4-training/examples/geometric_frozen_map_fit/prefix_fragment_learning.rs#L507) assigns zero weight and discards the Prefix trace. The protected guard role vetoes bad finite proposals but adds no gradient credit; overlapping task/reference roles still contribute their declared weights. This is consistent with the four rejected Prefix vectors in #2084; it does not establish global infeasibility.

Reuse one native-anchored Generate/Copy score construction for task CE and protected winner/rival pooled log-mass margins. Every token mass must aggregate all physical Generate/Copy aliases and retain the same clipping, donor, U and integer forward authority. [Existing admitted vocabulary scores and loss](https://github.com/UOR-Foundation/uor-r4/blob/2636bb2e2a67b65212672f410c692e85b402481f/crates/uor-r4-training/src/geometric_generate_learning.rs#L1085) are the appropriate seam. The Generate-only preclip diagnostic is insufficient because it excludes Copy-covered targets and pooled aliases.

A prospective constrained direction can use these protected-margin adjoints while retaining all exact native acceptance checks. Compact authenticated Prefix incidence can preserve occurrence multiplicity and all eight lane keys for both credit and transaction without retaining duplicate full Source traces. This repairs a shared learning boundary rather than adding answer-specific conditions. A zero hinge at already-safe margins yields zero initial corrective credit; averaged guard CE is a distinct trade-off objective, not a constraint guarantee. Quantization and donor changes still require the full native transaction after any local projection.

**Next:** Implement the protected pooled-margin credit boundary through the existing graph, with a separately frozen finite direction policy and cost projection before model work. Preserve all 15 episode positions / 17 reference roles / 380 guards and immediate actual-artifact qualification after construction-positive evidence. A separate SpiralCore/E8 action bridge should proceed only against a named caller/basis gap and a discriminating behavior or cost question; the present review does not establish that replacing H4 transport solves the emission failure.

## Cost, delivery and limitations

Projection: 30 minutes total preparation/expert review/browser observation/documentation/delivery, one owned worktree, less than 20 MiB retained evidence, no model/build/GPU work. The shared cumulative ledger charges elapsed review/delivery separately from model compute. Current accepted model, STATUS, ROADMAP and tracker milestone table remain unchanged. Validation for this documentation piece: source-linked expert review, bounded browser witnesses, claim-wording and diff checks at the PR head. Website fixture claims, static deductions and browser observations remain separately labelled. Results are preserved with source hash and browser-observation receipt in iCloud after use; protected merge and fresh-main verification are required before completion.
