# Routed read (address-routed context access)

**Status: ACTIVE.** It is the base read of the pilot, rung-0 (the first full-length run, 16,404 updates) and four K16 runs (the Flock and Relation arms build on its tree; see [flock-read](./flock-read.md) and [relation-read](./relation-read.md)), but its own contribution has never been isolated: it received only 20 learning proposals in 16,404 updates ([attention-state record](../labs/attention-state-2026-10-18/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

A transformer decides what to read by comparing the current word with every earlier word. This model does something cheaper. It keeps the earlier words at the ends of a small tree of geometric addresses. To read, it starts at the top of the tree and, at each level, steps toward the branch whose address is nearest to where the current state points. After a few steps it arrives at one earlier word and picks up that word's information. By construction it never scores the whole history, so the work should not grow with the context (a design property; speed was never measured). Whether this single pick is useful is still unknown, because nobody has switched it off to see.

## What it replaces in a transformer

Attention: the step that decides which earlier positions to read. There is no query-key score table and no softmax. The read makes one hard choice per step and returns one earlier position's carrier vector.

![A ternary tree drawn over a row of context tokens. The query enters at the top, three child ports are lit at each level with their distances to the 16 orbit roots, the nearest is highlighted, and a rotation arrow accumulates down the path until one leaf token is picked.](../figures/mechanisms/route-read.svg)

*The routed read walks a ternary tree over the context: one nearest port per level, one leaf at the end.*

## How it works

The model width is split into 8-dimensional blocks. Per block, the query is an integer vector $q\in\mathbb{Z}^8$ taken at the last position. A ternary tree (a "BinderPlan" built by SpiralCore, the project's exact-arithmetic geometry library) is built over the context. Its leaves are earlier positions, and the context cap is 243 positions, which is $3^5$.

**Definition.** Let $G$ be the group of 64 signed-axis permutations of $\mathbb{Z}^8$. Coordinates are permuted and sign-flipped by table lookup, so no multiplication is involved. Let $R=\{r_0,\dots,r_{15}\}$ be the 16 roots of one E8 orbit, each a signed permutation of $(2,2,0,0,0,0,0,0)$. At level $\ell$ the model holds one learned element $a_\ell\in G$ and chooses a port $p$ by hard minimum, where $r_p$ is the orbit root tied to that port (schematic):

$$c_\ell=\arg\min_{p}\ \big(\lVert a_\ell q-r_p\rVert_1,\ \operatorname{Ham}(a_\ell q,r_p),\ p\big)$$

At level 0 the candidates are up to 16 frontier roots; at later levels they are the 3 children. Each choice composes a precomputed transport action between orbit roots ($\mathrm{BFS}$ is a table built by breadth-first search over exact moves), so $T_\ell=\mathrm{BFS}[\rho_{\ell-1}\to\rho_\ell]\circ T_{\ell-1}$. The output is $T_L$ applied to the carrier vector of the leaf reached. This is a schematic of the kernel `route_vector`; the exact comparator is in the code.

**Cost (proof, by construction).** The work per block is about levels $\times$ 16 distance comparisons. It does not depend on how many earlier positions exist. The cost of building and updating the tree, and tokens per second, were never measured in the audited records.

**Content-blindness (proof).** Port selection never inspects which token a leaf holds. Content enters only through the geometry of the query. That is why the read cannot express "copy the word after the earlier match"; see [relation-read](./relation-read.md).

**Learning.** Training is discrete. Each route parameter is tried against its 15 generator moves in a counterfactual forward pass. The best move that strictly lowers the integer loss is applied. Nothing is differentiated ([discrete-learning](./discrete-learning.md)). The output vector goes to the shared writer ([admission](./admission.md)).

**Rule check.** No softmax, sigmoid or tanh (D23); no matrix multiplication, because the 8x8 "rotation" is a table lookup (D30). The kernel uses gather, sign flip, compare and integer add (D11), but no D11 served export or export-gap measurement of a trained Route model exists in the records.

## What has been tried

Verdicts follow the audit: positive, negative, inconclusive, diagnostic-stop. Many older rows tested other forms (prime routing, float readers) that share the idea, not the kernel.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 2026-07-28 | Ancestor prime-router claims re-examined under this repo's gates | Reported angular routing, not token generation; not reproduced | inconclusive | [evidence note](../../archive/prime_router_geometric_context_evidence.md) |
| 2026-07-31 | Held-out Hopf-sector routing | 7 of 512 sectors used, against a bound of 64; a later projection raised this to 43 of 512 | negative | [D3 occupancy](../../archive/hopf_sector_occupancy_d3.md) |
| 2026-08-26 | Fixed prime-route attention, source-free qualification | Address and selection mechanics pass (32/32 canary); no lexical or semantic value shown | inconclusive (product probe NOT_RUN) | [#958 record](../../archive/prime_route_attention_qualification_958.md) |
| 2026-09-13 | 64-update pilot of learned addressed attention | Ran; no useful generated answers | negative | [#973 pilot](../../archive/native_geometric_addressed_attention_pilot_973.md) |
| 2026-09-24 | Integer coarse-address learning in the integrated prototype | Admission 1/24 to 17/24; selection 0/24; complete answers 0/12 per arm | negative | [A3 result](../../archive/integration/integrated-attention-a3-result-2026-09-24.md) |
| 2026-10-01 | Signed-2I address reader (float), 2 seeds x 320 updates, authored gap grammar | Correct source on 128/128 rows in both seeds; float only, not served. An integer paired q/k variant kept 128/128 answers | positive | [address reader](../../archive/integration/geometric-address-reader-2026-10-01.md) |
| 2026-10-04 | Three integer cell edits; matched radial sharing | Frontier loss falls but construction replies 0/8; 0 of 32 transfer answers | inconclusive / negative | [consumed cells](../../archive/integration/geometric-consumed-cells-2026-10-04.md), [radial sharing](../../archive/integration/geometric-radial-sharing-2026-10-04.md) |
| 2026-10-12 | Lorentz vs l2 score for a dense (softmax) read, 3 seeds | Ahead on every seed, but the pre-registered per-seed margin was met on 1 of 3. A scoring change, not addressing; the record calls it positive but below its own bar | inconclusive | [geometric read](../../archive/labs/geometric-read-2026-10-11/README.md) |
| 2026-10-13 | First integer-only GPU path: tree routing, rotation, shared writer | 11 finite CUDA controls pass; no training | positive | [discrete learning](../../archive/labs/discrete-geometric-learning-2026-10-13/README.md) |
| 2026-10-14 | Seed-1 pilot, 1,640 batches | Top-8 173 to 1,169; loss rose slightly; Route parameters never attempted | inconclusive | [A2 pilot](../../archive/labs/a2-pilot-2026-10-14/README.md) |
| 2026-10-15 | Seed-1 full rung-0 dose, 16,404 batches, 14,287 development rows | Top-1 149 (bar 1,941); top-8 747 (bar 4,775). The bars were unreachable because only 1,920 targets were admitted | negative | [rung-0 prep](../../archive/labs/rung0-admission-prep-2026-10-15/README.md) |
| 2026-10-17 | Host-only re-score of saved checkpoints | A context-free successor table scores 6,300 of 14,287 top-1; the trained read 149 | inconclusive | [host diagnostics](../../archive/labs/host-diagnostics-2026-10-17/README.md) |
| 2026-10-19 | Four K16 runs: two writer objectives x two seeds, Route held fixed | No read comparison (the writer objective varied). Occupancy collapse, no KEEP; top-1 65 and 463 (A), 101 and 230 (R) | negative (configuration) | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) |

**Reading these honestly.** On rung 0 the runs were first limited by [admission](./admission.md): most targets never reached the read. After K16 the cause of the weak ranking is open (ranking capacity, the read and the schedule all remain candidates), so these rows do not show whether routing helps or hurts. Rung 0 and the K16 runs used a codebook with only 16 distinct vectors at the start, so the read could not be separated from codebook drift. None of the failures closes the mechanism (D22: a negative closes a configuration, never a mechanism). Trajectory routing was found inert in its product probe, and the product probe of prime-route attention (#958) was NOT_RUN, so it is neither a pass nor a negative. That says nothing about routing inside a trained integer read.

## What has never been tested

- Route against no read (a read-off ablation), and Route against [Flock](./flock-read.md) on a codebook whose addresses are distinct.
- Route trained with a schedule that gives it more than 20 proposals in 16,404 updates.
- The tangent-null and normal-null controls (move the address along the manifold versus across a branch), which the addressed-read brief requires to show the geometry is load-bearing.
- Spanning-tree addressing as addressing. Earlier variants were measured only as scoring changes.
- Served tokens per second of a routed read against a scored read at long context.
- Content-aware port selection.

## What comes next

1. Run a read-off versus read-on ablation with enough Route proposals (an interleaved schedule or a dedicated Route stage). This is the next step named on the mechanism board.
2. Fix the codebook first: a fair test needs distinct addresses ([address-cells](./address-cells.md), [prime-zeta-codebook](./prime-zeta-codebook.md)).
3. The float-era forms (spanning-tree router, address-as-read) may be re-registered as an offline float sheet under D37, with a fresh fitness review, an integer export and a reported export gap (hypothesis).

Open questions: how much of the 149 top-1 comes from Route; whether a content-blind read can reach the 3,847 copy-successor targets (the relational-head design argues from the kernel code that it cannot, which is why [relation-read](./relation-read.md) exists); which invariant should serve as address; and whether one 16-root orbit is enough atlas.

## Also known as

`ReadSpec::Route`; invariant-addressed read; spanning-tree router; route-native or prime-route attention; read lineage (phase bind, rotation, carrier); the legacy `uor-r4-router` prime router.

## Code

- [`crates/uor-r4-discrete/src/discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu): kernel `route_vector` (lines 41-79) and the shared counterfactual kernels.
- [`crates/uor-r4-discrete/src/lib.rs`](../../crates/uor-r4-discrete/src/lib.rs): `ReadSpec::{Route,Flock,Relation}` and `Parameter::Route`.
- [`qualification.rs`](../../crates/uor-r4-discrete/src/qualification.rs): the 11 finite CUDA controls. [`rows_device.rs`](../../crates/uor-r4-discrete/src/rows_device.rs): row-parallel scoring, about 56 times faster with identical losses.
- Float-era, dormant: `crates/uor-r4-training/src/stack_address.rs`, `geometric_address.rs`. The legacy `crates/uor-r4-router/` does not build against the current core (#2345) and is kept as evidence.

## Related mechanisms

[flock-read](./flock-read.md) replaces the single leaf with a consensus of several neighbours. [relation-read](./relation-read.md) adds links Route cannot express. [admission](./admission.md) consumes the routed vector and is where rung 0 first hit a measured limit. [address-cells](./address-cells.md) and [prime-zeta-codebook](./prime-zeta-codebook.md) give positions their addresses. [spiralcore-fbs](./spiralcore-fbs.md) supplies the transport table and the tree. [discrete-learning](./discrete-learning.md) trains the route parameters. [exact-memory](./exact-memory.md) is the older idea of routing to an exact prime address.
