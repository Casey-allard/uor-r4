# SpiralCore

SpiralCore is an exact-geometry source, credited to Matthew (versions v63 and v69). From the multiplication table of the octonions (an eight-part number system) it produces a small set of exact moves on eight numbers: each move shuffles the eight coordinates and flips some signs. Every move can be undone exactly. The model stores earlier words in a tree with three branches per node (the "ternary FBS" tree, the project's name for it; see the [glossary](../start/glossary.md)), and it carries a chosen eight-number block along a short path of these moves. No learned multiplication is involved, only swaps and sign flips. This chapter explains the moves, the tree, and how they connect to the other geometry. The integer form is the core of the current model. Terms are also defined in the [glossary](../start/glossary.md).

![A three-branch tree of stored atoms with one root-to-leaf path highlighted, beside the 16 roots of one E8 orbit drawn as nodes of a 15-edge graph. As a query point moves, the chosen atom's 8 coordinates travel along graph edges; each edge permutes axes and flips signs.](../figures/geometry/spiralcore-ternary-tree.svg)

*A ternary tree picks one stored block. The block then travels along exact edges between the 16 roots of an E8 orbit.*

## The exact moves

**The table.** [`spiralcore_operator.rs`](../../crates/uor-r4-core/src/spiralcore_operator.rs) reproduces the SpiralCore v63 octonion and Cl(0,6) convention exactly. The octonions' seven oriented Fano lines are

$$(124)(235)(346)(457)(561)(672)(713).$$

The Clifford algebra Cl(0,6) has six generators that each square to $-1$. Products of two distinct generators give the 15 **bivectors** $B_{ij}$ (one per pair of the six, $\binom62=15$), with $B_{ij}B_{jk}=-B_{ik}$. They mirror the way the semiprimes $p_ip_j\cdot p_jp_k\to p_ip_k$ combine. The $B_{ij}$ generate a group of order 64 that maps onto the 32 square-free even prime products, with kernel $\pm1$. Together this gives a 64-state composition table. Each $B$ is a signed coordinate permutation of $\mathbb{R}^8$ with $B^2=-I$.

**The orbit.** One anchor orbit holds **16 roots** and **64 signed operator states**, and an output block's carrier is moved by the composed edge history. A breadth-first search (BFS) over the labelled edges gives an ordered history between any two roots, composed by left-composition.

**Exact inverse (proof).** From $B^2=-I$ it follows that $B^4=I$ and so $B^3=B^{-1}$. The inverse of a history is therefore the reversed sequence with each step cubed.

**A worked step.** The projection chooses one input block $x_{\sigma(j)}$ for each output block $j$ and sets $y_j=A_jx_{\sigma(j)}$, where $A_j$ is the composed signed action. The forward pass is a gather and a sign flip. There is no learned coefficient for any input-output pair, and the adjoint is the exact inverse action $A_j^{-1}\delta_j$ ([SpiralCore/FBS projection](../mechanisms/spiralcore-fbs.md), proof by construction).

## How the tree and the moves connect to the rest

| Link | Statement | Forced or chosen |
|---|---|---|
| Six-prime registry {5, 7, 11, 13, 17, 19} → 15 semiprimes $p\cdot q$ | products of distinct pairs | forced |
| 15 semiprimes → 15 edges of K6 | edge $(p,q)$, by unique factorisation | forced |
| Semiprimes → Cl(0,6) bivectors | $B_{ij}B_{jk}=-B_{ik}$ mirrors $p_ip_j\cdot p_jp_k\to p_ip_k$ | forced |
| Cl(0,6) bivectors → octonions | Cl(0,6) acts on the octonions through the 7 Fano lines | forced |
| Fano plane → XOR on 3-bit indices | after relabelling, every line is $\{a,b,a\oplus b\}$, so the octonion product is a signed XOR | forced up to relabelling (168 relabellings) |
| Octonions → quaternions | each Fano line spans a copy of the quaternions; the octonions are alternative and norm-multiplicative, and 168 of 210 ordered unit triples are non-associative (exactly the non-collinear ones) | forced |
| Cl(0,6) bivectors → standard-coordinate E8 | the signed matrices preserve the 240 standard-coordinate roots (check E1), independently of the reduced-metric icosian realisation | forced for the specified coordinates |

The code is in `spiralcore_operator.rs` and [`cd_space.rs`](../../crates/uor-r4-core/src/transformerless/cd_space.rs). The remaining links (icosians, Z[φ], Hopf) are in the [other chapters](./README.md#how-the-pieces-connect). The semiprime-to-axis step needs a labelling choice, covered in [E8, H4 and the 600-cell](./e8-h4-and-the-600-cell.md).

## What carries route history

| Question | Answer |
|---|---|
| What carries route order? | Current M1 uses ordered FBS plans and composed 8D signed transport. The retained float stack uses noncommuting quaternion recurrence. These are distinct implementations (`crates/uor-r4-discrete`, `geometric_stack.rs`) |
| What tracks accumulated phase? | `hopf_metric.rs` tracks a cumulative U(1) phase and a geodesic distance, but only in the held native learner. `SpinTorsionState` uses Q30 storage with float-backed helpers outside M1 ([Hopf chapter](./hopf-observation.md)). Separately, under the October 14 addendum to D32, `prime_zeta.rs` uses offline complex-plane zeta-zero phases to build `Snapshot.table` (ACTIVE; [#2473](https://github.com/UOR-Foundation/uor-r4/pull/2473), [#2477](https://github.com/UOR-Foundation/uor-r4/pull/2477)) |
| What are chirality and polarity? | Exact signs on Z[φ], as fixed integer tables in `anchors.rs`. Used by the held native learner only ([Golden integers](./golden-integers.md)) |
| What is SpiralCore's action used for? | Since #2279 its signed BFS action is the trunk's projection ([`spiralcore_fbs.rs`](../../crates/uor-r4-training/src/spiralcore_fbs.rs)): READY in its float form, ACTIVE in its discrete integer and CUDA form in `crates/uor-r4-discrete` ([#2379](https://github.com/UOR-Foundation/uor-r4/pull/2379), [#2395](https://github.com/UOR-Foundation/uor-r4/pull/2395), [#2400](https://github.com/UOR-Foundation/uor-r4/pull/2400); [discrete record](../../archive/labs/discrete-geometric-learning-2026-10-13/README.md)) |
| What was tried earlier? | An August recursive-attention configuration that kept one reusable summary state lost the order of earlier tokens (the August A1.0 record). The retrieval arms also called A1 were stopped on 1 October (D18) because every arm scored below 0.5 recall at distance 16. Under D22 that closes the configuration, not the mechanism. A route-holonomy read and an octonion-signed binding remain designs that are not on the current path ([#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)) |

## Octonion-signed binding, a design

![The Fano plane has seven points and seven lines, each line a, b, a xor b. Binding two basis units gives plus or minus the third unit on their line; signed binding changes sign on swapping distinct units and is non-associative for 168 of 210 ordered triples.](../figures/geometry/octonion-signed-binding.svg)

*A historical proposal: bind with a signed XOR whose sign depends on operand order, unlike plain XOR.*

On the Fano plane, $e_a\cdot e_b=\pm e_{a\oplus b}$: a signed XOR whose sign depends on operand order, unlike plain XOR (which is commutative and associative, so it loses order and grouping). Incidence alone does not define coherent octonion signs. The design is not on the current path, and it needs its own pre-registration and a `TEST FITNESS: FIT` review before compute. The canonical v63 Fano table in `spiralcore_operator.rs` is the control for it. It has not been tested against plain XOR ([VSA mechanism page](../mechanisms/vsa.md)).

## The 9 October substrate review

Mark (N3mesis)'s supplied *Octonian Substrate Architecture* and discussion PDFs motivate separating state, algebra, routing and identity. The review of that draft found the following.

- Its literal positive cycles $(123)(145)(167)(246)(257)(347)(356)$ fail alternativity and quadratic norm multiplicativity. For example, $(e_1+e_2)(e_4+e_7)=0$ although each factor has squared length 2. Fano incidence is not a complete sign convention.
- For the draft's XOR labelling, an exhaustive search of all $2^7=128$ orientations of those seven lines finds 16 that give a valid octonion algebra (the enumeration is in the [historical geometry tour](../../archive/history/geometry-float-form-2026-10.md); the review record states only the counterexample). One of them reverses three of the draft's lines: $(123)(145)(167)(246)(527)(437)(536)$.
- The sample store route discards a coordinate after right multiplication and projection, and its packing function returns empty bytes. A pre-route embed/project inverse does not establish post-route preservation.
- Modular $\mathbb{Z}/2^{256}$ coefficients are neither the real normed division algebra nor a replacement for exact $\mathbb{Z}[\varphi]$ pairs.

These findings concern the supplied draft. They are not a negative language experiment and not a retirement of octonion binding. The canonical Rust table is unchanged, and any alternative convention needs a signed basis correspondence, not just the same seven lines. The orientation in the draft is inconsistent with the canonical table and must not replace it ([full review](../../archive/labs/octonion-substrate-review-2026-10-09/README.md)).

## Where the model uses this

| Mechanism | Status | Use |
|---|---|---|
| [SpiralCore/FBS projection](../mechanisms/spiralcore-fbs.md) | READY (float); integer form ACTIVE | Each output block picks one stored block and moves it along exact E8 steps |
| [Route read](../mechanisms/route-read.md) | ACTIVE | Walks the ternary tree (context cap 243 positions, $3^5$) and composes a transport action at each level |
| [Flock read](../mechanisms/flock-read.md) | ACTIVE | Carries kept payloads into the query's frame by the precomputed transport between roots |
| [Discrete learning](../mechanisms/discrete-learning.md) | ACTIVE | Learns over the 15 generator moves |
| [Ternary and XOR](../mechanisms/ternary-xor.md) | PARKED | The ternary binder tree and trit gates; its parts live in discrete learning |

## Where it stands

| Claim | Kind | Source |
|---|---|---|
| $B^3=B^{-1}$, so every history reverses exactly | proof | derivation above |
| The seven Fano lines and the 64-state table are reproduced exactly | measurement (finite checks) | [`verify_connections.py`](../figures/geometry/verify_connections.py), 29 checks of differing scope |
| Integer SpiralCore transport runs on the GPU | measurement (controls only) | [discrete record](../../archive/labs/discrete-geometric-learning-2026-10-13/README.md) |
| The transport carries enough information, or improves language | not shown | none of the four K16 runs passed its registered top-1/top-8 bars, and the call was no KEEP ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)) |

## Go deeper

- Code: `spiralcore_operator.rs`, [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu), [`build.rs`](../../crates/uor-r4-discrete/build.rs), [`spiralcore_fbs.rs`](../../crates/uor-r4-training/src/spiralcore_fbs.rs), [`m6_exact_arithmetic.rs`](../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) ([M6 exact arithmetic lab record](../labs/m6-exact-arithmetic-mechanism/README.md)).
- Back to the [overview](./README.md), or on to the [mechanism board](../mechanisms/README.md).
