# The geometry, illustrated

This page is a picture-first tour of the geometric objects the UOR-R4 Geometric Language Model is
built from. "Geometric" here means that the model's state and memory live on exact geometric
objects (unit quaternions, a fixed 120-element icosian table, exact golden-ratio integers, prime
addresses) rather than on a transformer's dense attention and MLP blocks at run time. The target
serving path (decision D11) uses no floating point and no multiplier instruction. Everything below
describes the mechanisms as built; a measured advantage over ordinary controls is not yet
established (see [README, Results](../README.md#results-so-far)).

Two overview figures first: the [architecture](figures/architecture.svg) and the stack of geometric
mechanisms.

<img src="figures/architecture.svg" width="100%" alt="Architecture overview: language base line, grounded memory line, and integer serving">

<img src="figures/geometry-stack.svg" width="100%" alt="The geometry stack: prime addressing, zeta phases, R4/S3 state, H4 and icosian structure with exact Z[phi]">

Two code paths use these objects. The **geometric stack** (`crates/uor-r4-training/src/geometric_stack.rs`)
is the trained language model. The **native learner** (`crates/uor-r4-core/src/native_geometric/`)
is the exact addressed-memory path. They share ideas but not all mechanisms; each section says which.

## How the next token is predicted

<img src="figures/geometry/next-token.svg" width="100%" alt="Pipeline of the geometric stack: token id, embedding, sixteen alternating recurrence and read layers, final norm, logits, softmax, optional copy head, next-token probabilities, and the served integer form">

This is the path of the trained **geometric stack** (`crates/uor-r4-training/src/geometric_stack.rs`)
for one token. The numbers match the figure's badges. Section 2 and the later sections draw the
geometric objects in detail; [Life of one token](#life-of-one-token) lists the same path step by step.

1. **Embed.** The byte-BPE id x_t selects one row of the learned embedding matrix E (vocabulary 4,096,
   width d; d = 1536 at 214M). Formula: h⁰_t = E[x_t].
2. **Carry the state by rotation (`r` layers).** For each 4-channel lane, projections give a unit
   quaternion u_t, a gate λ_t and an input a_t (a width-4 causal convolution). The previous state is
   rotated by u_t and blended with the new input: h_t = λ_t (u_t · h_{t−1}) + √(1−λ_t²) a_t. In
   training, u_t can optionally be snapped to the nearest of the 120 unit icosians (straight-through).
3. **Read the past (`a` layers).** The layer scores earlier positions with a Dot or a Lorentz
   (hyperbolic) score, −β·arcosh(1+e), adds an age term and a NoRead slot, and a softmax mixes the
   earlier states: h_t ← Σ_s softmax(score)_s · h_s.
4. **MLP and residual.** Every layer ends in a SwiGLU MLP with a residual connection. The 16 layers
   follow the pattern `rrarrarrarrarrar` (r = recurrence, a = read).
5. **Logits.** A final norm, then z_t = norm(h_t) · Eᵀ, with the output head tied to the embedding E
   (float form; the served form has its own head). In the float form a softmax over the 4,096 tokens
   gives p_soft = softmax(z_t).
6. **Optional copy head.** A gate g_t = sigmoid(w_g·h_t + b_g) mixes in a copy distribution:
   p(v) = (1−g_t)·softmax(z_t)[v] + g_t·p_copy(v), where p_copy sums attention over earlier positions
   that hold token v. The sources are chosen by a learned score or by the exact prime route: gcd of the
   prime products of the last ≤ 6 tokens against earlier windows, or the longest n-let
   ([section 7](#7-prime-uor-addressing-and-exact-memory)).
7. **Serving.** Steps 2–6 run in fixed point through the D11 integer table kernels (4-bit weights, no
   multiplier instruction, no floating point). The integer path is bit-exact with the float path on a
   3,072-target check.

**What it does not use.** The stack's prediction path uses no Hamming distances, no spin and no
spherical harmonics. Hamming/popcount and H4 codes appear in the native learner (the exact
addressed-memory path) and in the frozen R4G1 runtime. Spherical harmonics appear only in
`native_geometric/learner/geometric_attention.rs`. The "R4/Spin" models in the project history were
transformers. The output softmax is standard: the geometry is in how state is carried (quaternion
rotation), how the past is read (Lorentz score) and how exact copying is routed (primes).

### Softmax at runtime: where it is gone and where it is still emulated

| Place | What the served engine does | Status |
|---|---|---|
| Next-token choice | Greedy argmax over integer i32 scores (`stack_argmax`, `crates/uor-r4-integer/src/stack/chat.rs:296`) | No softmax for token choice. Sampling would need one and is not implemented. |
| Read-layer weights | exp(−d) read from a sealed lookup table (`stack_exp_neg`, `crates/uor-r4-integer/src/stack/kernels.rs:350–375`; table spec `format.rs:228`), then integer normalization. No float, no multiplier instruction. | Table-emulated softmax, not yet designed out. |
| Flock selection | Fixed rank weights w_i ∝ 1/(i+1) from a table (`crates/uor-r4-integer/src/stack/flock.rs:356–436`) | A softmax-free alternative already in the engine. |
| Copy-head mixture | Computed in fixed point by the integer engines | Softmax-shaped, through the same tables. |

## 1. Quaternions and rotation

<img src="figures/geometry/quaternion-rotation.svg" width="100%" alt="A unit quaternion rotating a 3D vector, and the left-multiplication used as a state update">

**What it is.** A quaternion is a four-number value that can encode a rotation in 3D. A *unit*
quaternion (length 1) is a pure rotation, and multiplying two of them composes the rotations. The set
of unit quaternions is the 3-sphere S³.

**What the model does with it.** In the stack, every recurrent layer treats each lane of 4 channels as
one quaternion state. The update is a left-multiplication by an input-dependent unit quaternion
(a rotation of the previous state, not a conjugation q h q*), mixed with new input; see the next
section for the formula. The quaternion algebra is fixed; the rotation the model chooses is learned.
Code: `crates/uor-r4-training/src/geometric_stack.rs` (`quaternion_scan`).

**Status.** Trained and served (the integer engines run the exported stack).

## 2. The state lives on S³ and moves step by step

<img src="figures/geometry/s3-and-transport.svg" width="100%" alt="A state point on the 3-sphere carried along by successive rotations, with decay toward new input">

**What it is.** Think of the state as a point on a sphere. Each new token turns the sphere a little
(the rotation) and pulls the point partly toward the new input. A decay gate decides how much of the
old position survives.

**What the model does with it.** The `r` layer recurrence, per 4-channel lane:

    h_t = λ_t (u_t · h_{t−1}) + √(1−λ_t²) a_t

Here u_t is an input-dependent unit quaternion (near identity at initialisation), λ_t in (0,1) is a
gated decay, and a_t is the input after a width-4 causal convolution. One exact scan runs the
recurrence; the backward pass is the reverse scan. The `a` layer is the other temporal mixer: a
multi-head read over positions up to t, scored by a Lorentz (hyperbolic) or Dot score, with a learned
age term per head and distance, and a NoRead slot whose value is zero. The layer pattern letters
(for example `rrarrarr…`) choose the mixer per layer; every layer ends with a SwiGLU MLP and a
residual connection. Code: `geometric_stack.rs` (header; `layer_kind`, `fused_read`).

**Status.** Trained and served.

## 3. The 120 icosians and the 600-cell codebook

<img src="figures/geometry/600-cell-icosians.svg" width="100%" alt="The 120 unit icosians as vertices of the 600-cell, with a learned rotation snapped to the nearest vertex">

**What it is.** Among all unit quaternions there is a special finite set of 120 (the binary
icosahedral group 2I). They are the vertices of the 600-cell, a 4D analogue of the icosahedron, and
they are closed under multiplication: any product of two is again one of the 120.

**What the model does with it.** `TransportSnap::Icosian` replaces u_t (before the λ scaling) by the
nearest of the 120 elements, so every transport is an exact member of 2I. During training the snap
uses a straight-through gradient: `u + (snap(u) − u).detach()`, so the forward pass sees the snapped
rotation and the backward pass sees the continuous one. It is applied inside the fused recurrence
core and is off unless set (`set_transport_snap`). Separately, the native learner indexes the same
120 elements as a table (product, inverse and rank lookups) in
`native_geometric/addressed_attention/artifact.rs`. `E8 = H4 × H4` is project shorthand for the
concrete golden/Galois-coupled icosian construction `H4 ⊕ φH4`; the typed paired representation is in
`native_geometric/anchors.rs`, and the stack does not use it.

**Status.** Icosian table: built and tested, used by the native learner. Snap: implemented for
training; whether any trained, shipped model uses it has not been verified. Paired-H4 integration is
marked unfinished in the artifact header.

## 4. Exact golden integers

<img src="figures/geometry/golden-integers.svg" width="100%" alt="Numbers of the form a + b phi on a line, with exact sign tests">

**What it is.** The golden ratio φ ≈ 1.618 satisfies φ² = φ + 1. Numbers of the form a + b·φ with
integer a and b add and multiply exactly, with no rounding, and the coordinates of the 120 icosians
live in this set.

**What the model does with it.** `ZPhi { a, b }` is checked exact integer arithmetic
(`crates/uor-r4-core/src/prime_route_attention.rs`; separate copies in `native_geometric/guarantees.rs`
and `canonical_lexical_ingestion.rs`). `compile_anchor_table` (`native_geometric/anchors.rs`) uses it
to compute, for each of the 120 icosian roots, an exact sign of the sine part (chirality) and of the
cosine part (cosine polarity), each in {−1, 0, 1}, with no float comparison. These combine into an
orientation class in 0..8 and a squared projection radius in Z[φ]. The antipode flips chirality. These
are fixed tables.

**Status.** Built and tested. Which `ZPhi` operations a trained forward pass uses has not been
verified.

## 5. Hopf observation and the kept fiber

<img src="figures/geometry/hopf-fibration.svg" width="100%" alt="The Hopf map from the 3-sphere to the 2-sphere, with the circle fiber that the map forgets">

**What it is.** The Hopf map squashes S³ onto an ordinary sphere S². Many points of S³ land on the
same point of S²: they form a circle (the fiber). Rotating a point along its fiber changes the S³
point but not its S² image, so the S² picture alone forgets exactly that phase.

**What the model does with it.** For a unit quaternion (a, b, c, d), `UnitS3Q30::hopf` returns
`(2(ac+bd), 2(bc−ad), a²+b²−c²−d²)` in Q30 fixed point. `rotate_common_fiber(phase)` applies
(z1, z2) → e^{i·phase}(z1, z2), which leaves the Hopf image unchanged. `SpinTorsionState { s3, fiber,
torsion }` keeps the fiber and torsion explicitly, and `GeometricAddress` carries them, so the
observation does not silently drop orientation. Code: `core/src/prime_route_attention.rs`;
`native_geometric/hopf_metric.rs` extends it (its header names a Fubini–Study metric and integer
serving; I did not verify the body). None of this is in the stack recurrence.

**Status.** Built and tested in the core and native-learner code; not part of the trained stack.

## 6. Zeta-zero phases

<img src="figures/geometry/zeta-phases.svg" width="100%" alt="Fixed zeta-zero frequencies turning log-prime differences into phases on a circle">

**What it is.** The imaginary parts γ of the first non-trivial zeros of the Riemann zeta function are
a fixed list of numbers (14.13…, 21.02…, …). Multiplying one by a difference of logarithms gives an
angle, and angles wrap around a circle. The table is a constant; nothing about it is learned, and it
does not require solving any open problem.

**What the model does with it.** In the native learner, the phase of a token with prime p in channel
k is `γ_k · (ln p − ln 2)`, wrapped to a turn and stored as a u16 (fixed table of 258 token rows × 8
channels; byte b is row b+2). At run time each observed byte wrap-adds its first four channel phases
into a running 4-phase state, and the top four bits of each phase (`phase >> 12`) are fed to the
policy as context. So the zeta phases are a fixed additive accumulator that is quantized into policy
inputs; they do not rotate a vector or multiply an activation. They are **not** in the quaternion
stack (`geometric_stack.rs` has no zeta reference). Code: `core/src/zeta_zeros.rs` (512 zeros),
`prime_route_attention.rs` (`zeta_phase_delta`), `native_geometric/training.rs`,
`addressed_attention/engine.rs` (`observe_phases`).

**Status.** Fixed table, used by the native learner's pilot. What the trained policy does with the
zeta bins beyond input encoding has not been verified.

## 7. Prime (UOR) addressing and exact memory

<img src="figures/geometry/prime-addressing.svg" width="100%" alt="Tokens mapped to primes, window products, and a gcd test selecting earlier positions">

**What it is.** Give every token its own prime. A short window of tokens is then the product of its
primes, and two windows share a token exactly when their products share a factor, which a gcd
reveals. Identity is exact; it is an identifier, not a semantic distance.

**What the model does with it.** In the stack's prime-route memory port, `token_prime(id)` is the
(id+1)-th prime and `route_key(ids, end, window)` is the u128 product of the distinct primes of up to
6 tokens ending at `end`. An earlier source is admitted either by `SharedAtom` (default:
`gcd(query_key, key) > 1`, score `ln(gcd) + 0.5·j/(t+1)`) or by `Ngram` (the longest matching n-let,
score the sum of `ln p` over its atoms plus the same recency term). Attention is then
softmax(4.0 · score) over admitted sources only and exactly zero elsewhere. Admission, gcd and
sharpness are fixed; only the optional ranked variant adds a learned score. Code:
`crates/uor-r4-training/src/geometric_stack.rs` (`RouteAdmission`, `route_attention`).

In the native learner the store is a log of `Occurrence` records `{epoch, sequence, turn, byte,
origin, keys: [u16;2], roots: [u16;4]}`, where keys and roots are indices into the 120-icosian table.
After each byte the policy chooses, per lane, an icosian delta and `roots[lane] = product(roots[lane],
delta)`, then chooses two key roots to store on the record. To read, the policy proposes a query
`[u16;2]`; a candidate's score is the sum over the 2 lanes of `ranks[product(query, inverse(key))]`,
scanned over the last 256 records (earliest wins ties; a Null key is allowed). The result is a lease
that typed actions can use. Tables, scoring and arithmetic are fixed integers; the policy that picks
roots is trained offline. Code: `native_geometric/addressed_attention/{objects,artifact,engine}.rs`.

**Status.** Prime-route port: implemented in the stack as an evaluation instrument. Native store:
built and tested, trained policy, with broad language qualification unfinished. The separate core-crate
route (`PrimeAtom`, `SemiprimeExpert`) is canary-scale, and whether a trained model calls it has not
been verified.

## 8. VSA hypervectors and Hamming similarity

<img src="figures/geometry/vsa-hypervectors.svg" width="100%" alt="A token's 4096-bit bipolar code, XOR and popcount to a Hamming distance and bipolar cosine, and where the codes come from">

**What it is.** Each token has a 4096-bit bipolar hypervector (64 × `u64` words, bit set = +1, clear =
−1). Two codes are compared with an XOR and a popcount, which gives a Hamming distance d; the bipolar
cosine is 1 − 2d/4096. A context hypervector bundles the codes of the recent tokens. The similarity is
a statistic of the code bits; it is only as meaningful as the way the codes were assigned.

**What the model does with it.** In the native prose learner (line B) the VSA term adds
`vsa_scale · similarity` to the root and leaf scores. Three code modes exist. Mode 0 is a fixed token-id
hash (historical; it is disconnected from the learned 120-root assignment, which is why it was
described as mis-wired). Mode 1 derives each code from the learned icosian-root assignment (a token's
nearest of the 120 roots, a Voronoi cell over the learned embeddings), and since PR #2077 these codes
are trainable and refresh during training. Mode 2 binds the root code with a readout-hash residual.

**Status.** Built and tested. A pre-registered retraining test (M1, #2029) is running; no measured
advantage is claimed here. VSA is not tied to SpiralCore (next section).

## 9. What carries route history (and what SpiralCore is)

| Question | Answer | Where |
| --- | --- | --- |
| What carries route order? | The stack's quaternion recurrence: quaternion products are order-dependent, so the state encodes the order of rotations. Trained and served. | `geometric_stack.rs` |
| What tracks accumulated phase? | `hopf_metric.rs` tracks a cumulative U(1) holonomy phase and a geodesic distance. Line B only. | `native_geometric/` |
| What are chirality and polarity? | Exact signs on Z[φ]. Line B only. | `native_geometric/` |
| What is SpiralCore? | `spiralcore_operator.rs` reproduces the SpiralCore v63 octonion/Cl(0,6) convention exactly (oriented Fano cycles (124)(235)(346)(457)(561)(672)(713); 15 bivectors ↔ 15 semiprimes; a 64-state composition table). It is an exact finite control in `recursive_geometric_attention.rs` (A10), re-checked in graph-certify, and sits on no training or serving path (0 references in `uor-r4-training`, `uor-r4-integer`, `uor-r4-api`). Mark (N3mesis)'s later supplied substrate sketch uses a different, inconsistent orientation; it must not replace this canonical table (see the review below). | `uor-r4-core/src/spiralcore_operator.rs` |
| What was tried and what is pre-registered? | Recursive geometric attention over earlier tokens (A1) was stopped on 1 October (D18) because a reusable state erased order. A route-holonomy read (rank earlier positions by the angle of h_j⁻¹·h_t, softmax-free) and an octonion-signed binding test that may promote SpiralCore to a state carrier are pre-registered on M1 (#2029). | #2029 |

## 10. How the geometric pieces connect

<img src="figures/geometry/connections.svg" width="100%" alt="Diagram connecting the six-prime registry, semiprimes, K6, the icosahedral axes, the 120 icosians, quaternions, Z[phi], octonions and the Fano plane, Cl(0,6) bivectors, E8, the Hopf map and VSA codes, with each link marked as verified math, used in a model, pre-registered, or a labelling choice">

The project's geometric mechanisms are not separate inventions. They are views of one structure: the
binary icosahedral group 2I (the 120 unit icosians) and the octonions that contain it. Every link below
is checked by exact integer and Z[φ] arithmetic in
[`figures/geometry/verify_connections.py`](figures/geometry/verify_connections.py) (29 of 29 claims pass).
That makes the **structure verified mathematics**. **It does not establish predictive value.** Whether
any of these links helps the model is the subject of the pre-registered experiments on M1
[#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029). The SpiralCore material is credited to
Matthew (SpiralCore v63/v69); the octonion and Fano material to Mark (N3mesis, NEMESIS-Theory).

| From | To | Map | Forced or chosen | Where in code | Used by |
| --- | --- | --- | --- | --- | --- |
| Six-prime registry {5, 7, 11, 13, 17, 19} | 15 semiprimes p·q | products of distinct pairs | forced | `spiralcore_operator.rs` | SpiralCore control |
| 15 semiprimes | 15 edges of K6 | edge (p, q); unique factorisation | forced | `verify_connections.py` | — |
| 15 edges of K6 | 15 half-turn axes of the icosahedral group | each half-turn fixes exactly one pair of the 6 five-fold axes | forced, once the six axes carry the six labels | v69 AXIS2 | pre-registered E1 |
| Six primes | six five-fold axes | a labelling | **chosen**: 720 labellings, 12 inequivalent under rotation | `verify_connections.py` prints its choice | — |
| Semiprimes p_i·p_j | Cl(0,6) bivectors B_ij | B_ij·B_jk = −B_ik, mirroring p_i p_j · p_j p_k → p_i p_k; the B_ij generate a group of order 64 that maps onto the 32 square-free even prime products (kernel ±1) | forced | `spiralcore_operator.rs` | SpiralCore control |
| Cl(0,6) bivectors | octonions | Cl(0,6) acting on the octonions through the 7 Fano lines (124)(235)(346)(457)(561)(672)(713) | forced | `spiralcore_operator.rs`, `cd_space.rs` | control; pre-registered octonion-signed binding |
| Fano plane | XOR on 3-bit indices | after relabelling, every line is {a, b, a⊕b}, so the octonion product is a signed XOR (168 relabellings) | forced (up to relabelling) | `verify_connections.py` | pre-registered binding, Hamming-rank read |
| Octonions | quaternions | each Fano line spans a copy of the quaternions; the octonions are alternative and norm-multiplicative, and 168 of 210 ordered unit triples are non-associative (exactly the non-collinear ones) | forced | `cd_space.rs` | stack transport |
| Quaternions (S³) | 2I, the 120 unit icosians | the finite subgroup of S³; closed, order 120; shells 1, 12, 20, 12, 30, 12, 20, 12, 1 from every element; 720 edges (shell 1) | forced | `geometric_stack.rs` (icosian snap) | stack (snap), pre-registered E1 |
| 2I | icosahedral rotations | 2I/±1 = A5 (60 rotations), acting 2-transitively on the six five-fold axes | forced | `verify_connections.py` | — |
| 2I | exact Z[φ] | icosian coordinates lie in Z[φ]/2 | forced | `ZPhi { a, b }` | native learner |
| 2I ∪ φ·2I | E8 | the 240 E8 roots (the project's E8 = H4 ⊕ φH4); the bivectors B_ij preserve them | forced | E8 notes in `AGENTS.md` | — |
| 2I | Hopf S² | S³ → S² maps the 120 icosians onto one orbit of 30 points, every fibre of size 4 (a two-fold-type orbit, not literally the axes of the chosen frame) | forced | `hopf_metric.rs` | native learner |
| 2I | VSA bit codes | the learned token → icosian-root assignment builds the codes | learned | `vsa_codes.rs` (since #2077) | native learner (VSA test running) |

**Cautions for anyone building on this.**
1. **Half-turns don't compose like bivectors.** Two half-turns whose K6 edges share a vertex multiply to an order-5 rotation, never a half-turn. The prime ↔ half-turn match is a correspondence of sets, not of products.
2. **Labelling primes to axes is a choice.** A design that routes primes through icosian rotations must say which of the 12 inequivalent labellings it uses.

**October 9 substrate review.** Mark (N3mesis)'s supplied *Octonian Substrate Architecture*
and discussion PDFs motivate separating state, algebra, routing and identity. Their literal
positive cycles (123)(145)(167)(246)(257)(347)(356), however, fail alternativity and quadratic
norm multiplicativity: `(e1 + e2)(e4 + e7) = 0` despite factor norms 2 and 2. Fano incidence
is not a complete sign convention. The canonical Rust table above remains unchanged; an
alternative convention needs a signed basis correspondence, not just the same seven lines.
The sample store route also discards a coordinate after right multiplication and projection,
and its packing function returns empty bytes. A pre-route embed/project inverse does not
establish post-route preservation. Modular `ZMod(2^256)` coefficients are neither the real
normed division algebra nor a replacement for exact `Z[φ]` pairs. These findings concern the
supplied draft, not a negative language experiment or retirement of octonion binding.
The [full review and executable counterexamples](labs/octonion-substrate-review-2026-10-09/README.md)
record the exact documents, source, decisions and limits. The 29 connection checks above
continue to concern the repository's specified structures, not certification of external PDFs.

A further fact: the icosahedron's five orthogonal frames of two-fold axes form a *synthematic total* of
K6, five perfect matchings that together cover all 15 edges.

## 11. Pre-registered mechanisms (not yet measured)

These three mechanisms follow from the connections above and are pre-registered with fixed thresholds on M1
[#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) and M4
[#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). **None has a measured result yet.**
Each is integer-only and softmax-free by construction.

<img src="figures/geometry/exact-2i-lanes.svg" width="100%" alt="Exact icosian holonomy lanes: tokens map to icosians, a lane state is a product in the 120-element group updated by a 120 by 120 table, and earlier positions are ranked by the shell index of the relative product">

**Exact icosian holonomy lanes (E1).**
- **How it works:** each token maps to an icosian u_t, and a lane carries the exact product h_t = u_t·h_{t−1}. The state is 7 bits, updated by one read of a 120×120 table. A read ranks earlier positions by the shell index (0–8) of h_j⁻¹·h_t, using rank-table weights.
- **Why it's interesting:** the product keeps route order, because the group is not commutative, and it is exactly invertible.
- **Caveat:** it never decays, so it runs beside the learned r-layer rather than replacing it.
- **Source:** Matthew's SpiralCore v69 peer-shell catalogue.

<img src="figures/geometry/octonion-signed-binding.svg" width="100%" alt="Octonion-signed binding: on the Fano plane every line is a, b, a xor b, and e_a times e_b equals plus or minus e_(a xor b), so the order of the operands changes the sign, unlike plain XOR">

**Octonion-signed binding.** Binding by XOR of the 3-bit Fano indices plus the Fano sign gives
e_a·e_b = ±e_{a⊕b}, after a declared relabelling with a coherent sign table;
for distinct imaginary basis units, e_a·e_b = −e_b·e_a.
- Plain XOR binding loses the order of its operands; the sign keeps it.
- Non-associativity keeps grouping: (a·b)·c ≠ a·(b·c) for 168 of 210 ordered unit triples (verified).
- **Cost:** add, subtract and one sign-table read.
- **Source:** motivation from Mark (N3mesis)'s octonion/Fano material; the implemented convention is Matthew (SpiralCore)'s v63 table. The later substrate draft's all-positive orientations are not adopted.

<img src="figures/geometry/hamming-rank-read.svg" width="100%" alt="Hamming-rank read: binarized query and keys, distance equals popcount of query xor key plus an age term, positions sorted and weighted by a fixed rank table, with no softmax">

**Hamming-rank read.**
- **How it works:** binarize the learned query and keys, score each earlier position by popcount(q ⊕ k_j) plus an age term, sort the positions, and weight them with the flock rank table w_i ∝ 1/(i+1).
- **Why it's interesting:** it would replace today's exp-table-emulated softmax with XOR, popcount and a table.
- **Status:** arm D of the softmax-free read experiment. It will use the shared `BitCode` primitive.
- **Source:** motivated by Mark (N3mesis)'s XOR/Hamming sketch.

## Native bank input access (M2)

<img src="figures/geometry/native-input-access.svg" width="100%" alt="Current M2 input path: all supplied context, query and actual prefix tokens update geometric state, but only Source occurrences enter the explicit read and Copy lists; read-only history access is not yet integrated.">

The native bank generator is a separate caller from the stack's all-position read described below.
It encodes every admitted Source/Context token, the full query and actual response prefix in order.
The query also has its own H4 cue carrier and, when configured, a query-plus-prefix continuation field.
However, its direct occurrence read compares the final query snapshot only against **Source** positions;
those same candidates supply Copy IDs and optional bridge donors. Processing an early query token into
state does not make that position an independently selectable key. The combined 128-token admission cap
is specific to this reader and rejects overflow; it is not a project-wide context limit.

A read-only history role could expose retained input positions without granting Source/Copy authority.
It is **not yet integrated or qualified**, and source inspection does not establish that its absence
causes the current language errors. The [caller trace and causal comparison](labs/input-access-trace-2026-10-09/README.md)
keep input influence, explicit read access, donor selection and emission distinct. Existing H4 operators
can support the comparison; no conversion to E8 is inherently required for this access boundary.
The E1 and Hamming-rank mechanisms in section 11 remain separate pre-registered M1/M4 work.

## Life of one token

Follow one byte-BPE token through the stack (text path):

1. **Embed.** The token id selects a row of `embedding.weight` (width d; tied with the output head in
   float/frozen form).
2. **Project** (section 2). Each recurrent layer applies a norm, then projections give the width-4
   causal-convolution input a_t, a unit quaternion u_t and a decay λ_t for each 4-channel lane.
3. **Snap (optional)** (section 3). In training, u_t is replaced by the nearest of the 120 icosians,
   with a straight-through gradient.
4. **Transport** (sections 1 and 2). h_t = λ_t (u_t · h_{t−1}) + √(1−λ_t²) a_t.
5. **Read** (section 2). An `a` layer reads positions up to t with a Lorentz or Dot score, an age term
   and a NoRead slot; then the SwiGLU MLP and residual.
6. **Logits.** A final norm, then logits z_t = norm(h_t) · embeddingᵀ (tied), then softmax.
7. **Copy (optional)** (section 7). A pointer head computes a query, keys and a gate g_t; attention
   over earlier positions uses a learned score or the exact prime route (gcd of window products, or
   longest n-let), with `ln gcd` plus recency and sharpness 4.
8. **Mix.** p(v|t) = (1−g_t) softmax(z_t)[v] + g_t p_copy(v|t), where p_copy sums attention over
   earlier positions holding token v. The gate is learned (bias initialised at −2).
9. **Serve** (next section). The D11 integer and table engines run steps 2–8 in fixed point; a pointer
   is exported only if it keeps every source.

Zeta phases (section 6) and Hopf observation (section 5) do not appear in this path; they belong to
the native addressed-memory path, which runs separately: byte → add four zeta phase deltas → policy
picks root deltas (icosian product table) → policy picks key roots → record stored → later reads
score `ranks[q · k⁻¹]` over the last 256 records → lease → typed action.

## Serving without multiplication

Under D11 a served kernel has no floating point and no multiplier instruction. A product of two
runtime values is read from a table instead: build the sixteen multiples (0..15) of one operand by
repeated addition, index that table with the radix-16 digits of the other operand, and combine the
results with shifts and adds. For example, to compute 37 × 43: the table holds 0, 37, 74, …, 555.
Write 43 as hex 0x2B, digits 2 and 11. Then 37 × 43 = (T[2] << 4) + T[11] = (74 << 4) + 407 = 1184 +
407 = 1591. A 4-bit weight is a single digit, so one product is one table read. Division is restoring
long division, square root is digit-by-digit, and the kernels are kept opaque to the optimiser so that
an add chain is not folded back into a multiply. Products are taken modulo 2⁶⁴ or 2¹²⁸ to match the
earlier integer engine. Code: `crates/uor-r4-integer/src/stack/kernels.rs` (`stack_mul_u64`,
`stack_div_u64`, `stack_isqrt`); the ARM64 audit is `scripts/audit_zero_matmul_serving.py --stack`.
(The numbers in the example are illustrative, not taken from the code.)

## Reproduce the figures

    python3 docs/figures/geometry/generate.py
