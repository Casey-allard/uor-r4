# The murmuration read: design from SpiralCore's source and the murmurations paper (2026-10-13)

**Milestone:** M4 #2032 · **Line:** starling read, 3/3, with this as its one decisive run (D21) · **Owner:** approved the
redesign on 13 October. **Design only: no code, no compute.** Training stays held.

**Cards:**
- [design 6151093135](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6151093135);
- [pivot retarget 6151096178](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6151096178);
- [pivot card 6150703214](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6150703214).

## Why a redesign
The original murmuration card ([6115874382](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115874382),
11 October) had four parts:
- an O(1) recurrence summary;
- O(log n) scale-invariant blocks with a starling 600-cell lookup inside them;
- a hard readout;
- a softmax teacher.

D23 banned the teacher. Rung 0b then rebuilt only the neighbour lookup and a hard rank read with five learned maps, one of which (the key address) could never learn. The owner pointed out the drift: the murmuration mechanism, with SpiralCore as the glue, was the intended design.

## What the sources actually contain (read-through, four read-only agents)

**SpiralCore v69:** a local file, 1,229,455 bytes, SHA256
`6c01c2166647133395ab97557c7a44e2aa15dd78fbfb852cddee078ef306dcf7`. Not committed; it is an outside author's work. Line numbers below refer to that file.

| Part | What the code computes |
|---|---|
| Geometry | 120 unit icosians (1430–1444); the 600-cell graph has 12 neighbours per vertex (1630); shells around a point have sizes 1, 12, 20, 12, 30, 12, 20, 12, 1 (`shellAt` 1474; the sizes are claimed in comments and computed at run time); 240 E8 roots (`e8Roots` 2759) |
| Transport | `e8ApplyOperator` (3054): a signed-permutation bivector `B = L_{e_i}L_{e_j}` (octonion left actions, 2931) applied to a root, then snapped to a root ID; routes are composed hop by hop (`e8ApplyOperatorSequence` 3127); the exact inverse is each step cubed in reverse (B² = −I, 3138); round trips are checked for every root and operator (3263). The core graph BFS (`bfsPath` 3707) carries labels only, not state |
| Clifford "6+1" | `clifford67Product` (6049): 6 Cl(0,6) generators plus a closure ω = their product, giving 64 monomials (rank 64, checked at 6199) |
| Coupling | Kuramoto `dθᵢ/dt = ωᵢ + KΣAᵢⱼ sin(θⱼ − θᵢ)`, unnormalised and integrated with RK4 (`syncDerivative` 10212), on 60-edge H2, 15-cell quotient or FBS-chain graphs. SYNC-10 checks only that the 60-edge and 435-edge graphs give different trajectories. In the score lane it drives the timing clock, not data |
| Data → routes | A file is cut into 32-byte cells and written in base 63; each digit picks one of 63 equal-length shortest paths; the choices become FBS ternary trees (parent length = Σ children + 1, `fbsClose` 6109); decoding replays the routes and checks SHA-256 |
| Addresses | `nodeAddr` (4191): four octets giving layer kind, id and two incidence octets; reversible text |
| Learning | **None.** Everything is fixed, deterministic geometry |
| Flock / 7-neighbour rule | **None in the file.** Searches for murmur, flock, starling, swarm and boid all return 0 hits. The 7-neighbour rule comes from starling biology |
| Banned operations | No exp, softmax, sigmoid or tanh. `Math.log` appears in the audio decoder's scoring (10986), which is not used here. `mat8Mul` builds the fixed bivector matrices (fixed geometry, not a learned product) |

**The murmurations paper:** He, Lee, Oliver, Pozdnyakov, "Murmurations of Elliptic Curves", arXiv:2204.10140v2. Local PDF SHA256
`b5fc7057d433cbd4da09ed70ec1f97a3be85434595706acf4b13a32d029308e4`.
- **What is averaged.** The average `f_r(n)` of Frobenius traces `a_{p_n}` over curves of rank r in a conductor window oscillates in n.
- **Rank and sign.** Rank 0 and rank 1 mirror each other: the envelope's sign is opposite, and parity ties ranks 0 and 1 to the root number.
- **The window.** The pattern exists **only in the family average over a bounded window**. Too wide a window smears it, and a single curve is noise of size about √p.
- **The fit.** `A·x^α·sin(B·x^β)` with β ≈ 0.53, and B falls as the conductor grows.
- **Mechanism:** none; the paper calls it "mysterious". Its logistic-regression and PCA tools are not adoptable (sigmoid; dense covariance).

## The design (card 6151093135, condensed)
1. **One address per token,** from the shared SpiralCore/FBS map. It is used both to store and to query, so the key-address gap is gone.
2. **A flock of up to 7 by topological count** in the token's cell plus its 12-neighbour shell (K1). Strictly causal, cleared at BOS.
3. **Exact transport plus alignment.** `Δ_t = K_h·(mean_j (c_t c_j⁻¹) ⊗ v_j − v_t)`, using the exact 120×120 icosian quotient table, then `y = x + P(Δ)`.
4. **An FBS ternary tree.** Parents hold the transported means of their 3 children, giving at most `7·(1+⌊log₃ n⌋)` touched positions per token.
5. **What learns:** `A`, `P`, the values and `K_h`, with the address margin plus exploration slots for the hard cell choice. There is no rank table, null score or rank hinge; that rung-0b source stays inactive as evidence.

**Decisive bar (unchanged):** in-context recall ≥ +5 pp over rung 0 on both seeds. Transfer is scored deterministically. Tokens/s is reported first.

## Not claimed
This is design and hypothesis, not measurement. SpiralCore and the paper are inspirations whose parts are used exactly as their sources compute them. Neither establishes that the read will recall. **Next:** S1/S2 source with the §4 finite controls (FIT requested from Codex), after which the dependency order in the card applies.
