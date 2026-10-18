# Starling read: topological neighbours on the 600-cell

**Milestone:** M4 #2032 (no softmax at runtime; flat cost at long context). **Lab:** claude. **Owner idea:** a starling tracks about seven neighbours, and the flock's coherence emerges from that local rule.

## The idea

Real starlings respond to about seven neighbours chosen topologically, not by distance ([Princeton 2013](https://www.princeton.edu/news/2013/02/07/birds-feather-track-seven-neighbors-flock-together)). The read applies the same rule:
- Each token sits at an icosian vertex of the 600-cell.
- It reads only the ~K nearest recent tokens found in its own cell and the 12 adjacent cells.
- It never scans the whole context and never normalises with a softmax.
- Coherence across the sequence comes from layers and the quaternion state passing information one hop further each time.

SpiralCore v69 runs an unnormalised Kuramoto phase coupling, `dθᵢ/dt = ωᵢ + K Σⱼ Aᵢⱼ sin(θⱼ − θᵢ)`, over fixed geometric graphs: 60 edges on the H2 icosidodecahedron, a 15-cell signed quotient, or an FBS chain. Its test SYNC-10 shows only that the 60-edge and 435-edge graphs give *different* trajectories, not equal synchrony; an earlier wording here overstated it. SpiralCore contains no flock or 7-neighbour rule. That rule comes from the starling biology ([read-through](../labs/murmuration-design-2026-10-13/README.md)).

## In code

- **Active, discrete (D32): `ReadSpec::Flock { kept }` is the integer murmuration read.**
  - **Source:** `crates/uor-r4-discrete/src/lib.rs:100-103`; kernel `flock_vector` at `crates/uor-r4-discrete/src/discrete.cu:125`, dispatched at `:271`.
  - **How it works, per payload block b:**
    - Each kept candidate's payload is carried into the query's frame by the actual SpiralCore BFS action between the two frame roots.
    - The consensus is `snap_K` of the carried sum.
    - A Gate trit adds (+1) or subtracts (−1) each level's consensus to the query payload, and the output is `snap_n` of the exact residual.
  - **The rule it realises:** the murmuration rule `Δ = K_h·(mean − v_t)` with `K_h ∈ {−1, 0, +1}`. There is no float, no mean division and no learned projection.
  - **Selection:** rank order is frame L1, then coordinate Hamming, then recency (`lib.rs:94-96`). Kept is limited to 1..8.
  - **Status:**
    - Six finite CUDA construction controls PASS (`crates/uor-r4-discrete/src/flock_qualification.rs`; #2395, #2400).
    - Never trained on the corpus; training is held (`lib.rs:212-214`).
    - Design record: [discrete murmuration read](../labs/discrete-murmuration-read-2026-10-13/README.md).
  - **Relation to Dose 1:**
    - `ReadSpec::Relation` (`lib.rs:103`) is Flock plus an exact-relation rerank with 5 edge-type Gate trits per level. "Relation-off ≡ Flock" is its rollback identity.
    - Dose 1 (`docs/harness/prep/m1-learned-attention-link-2026-10-17.md`) compares learned against frozen relation trits **on top of Flock**. No arm tests the transported-consensus read itself against `Route`.
    - Under zero gates the read is causally inert, which is why Dose 1 initialises with +1 leaf gates (`ActiveLeafV1`, `lib.rs:76-86`).
- **Historical:** `read_route=starling:M:K`: `crates/uor-r4-training/src/geometric_stack.rs` (`StarlingRoute`, `starling_select`, `starling_read`). Also in `mqar-bench`. It selects on the host and gates with a sigmoid or tanh, so under D23–D25 it is inactive evidence.
- **Native, rung 0b:** `native_read=starling_native:M:K:E` on the native trunk. Source: `crates/uor-r4-training/src/starling_native.{rs,cu}`; wiring: `StackModel::set_native_starling_read`.
  - Kernel K1 selects neighbours on the GPU. It is built ahead of time by nvcc into its own module (`build.rs`; no NVRTC).
  - The read is a hard rank read over the kept slots, with a learned null score.
  - Its declared credit, injected in backward (`CreditCapture`): a pairwise hinge over utility-ordered kept, exploration and null candidates, and an address margin toward an off-home helpful candidate.

## Tried

- **Rung 1: recall at chance when trained from scratch** ([record](../labs/starling-read-2026-10-11/README.md)). Hard selection gives no gradient to keys outside the kept set. The cost saving is confirmed: 7× fewer positions scored.
- **Rung 2: the addresses learn, the readout does not** ([record](../labs/starling-anneal-2026-10-11/README.md)). A training-only dense helper is annealed to zero. With it, the true source enters the kept 7 neighbours 73% of the time at peak, against 10% at initialisation. On the pure hard read it stays at 43–56%. Recall is still at chance: the rank table and gate do not pick the right value among the 7.
- **Rung 3: a content gate does not help** ([record](../labs/starling-content-2026-10-11/README.md)). Gating the kept 7 by the read's q·k leaves recall at chance (0.010), with the source kept 47–56% of the time. The gate was not the bottleneck; the value path, or per-head availability of the source, is suspect. Note: content logits need a tanh-form sigmoid (the exp form overflowed).
- **Pivot diagnostics** ([record](../labs/starling-pivot-2026-10-11/README.md)):
  - Velocity routing adds nothing beyond token identity.
  - With the source at a fixed slot, recall is 1.000, so the value path works.
  - A shared kept set raises per-head availability to 61–72%, but recall stays at chance.
  - Content ranking does not learn to rank the source first (0.03–0.29). The likely cause is the missing competitive gradient.
  - Recommended: a training-only softmax surrogate (owner decision).
- **Softmax surrogate (training only)** ([record](../labs/starling-surrogate-2026-10-12/README.md)):
  - It teaches content selection: the source is ranked first 0.55–0.78 of the time when kept, against 0.03–0.12 without it.
  - With a shared kept set, recall lifts off chance unevenly (0.044, 0.288, 0.091).
  - Below the pre-registered reading-2 bar, so the line goes to the owner.

Rungs 1–3, the pivot and the surrogate were measured beside softmax helpers, surrogates or teachers. Under D23 they are inactive evidence: they stay recorded and are not to be rerun.

- **Rung 0b, the lawful native read: design and source.** Card [#2032 6146209361](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6146209361), amendment [A1](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6147797508), §4 finite controls [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6148141029); [record](../labs/starling-rung0b-source-2026-10-13/README.md).
  - It replaces the softmax surrogate's competitive gradient with a rank-consistent hinge whose order comes from each slot's stopped-gradient utility `u = -<g, v>`.
  - The null is a utility-0 candidate, so abstention is learned through truncation.
  - Exploration slots (zero forward weight) give near-miss keys credit, fixing rung 1's "keys outside the kept set get no gradient".
  - The decisive §5 run (in-context recall top-1 on copyable dev targets against rung 0) needs its own sheet after rung 0 reports.
- **D21 pivot card, 13 October** ([#2032 6150703214](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6150703214)). The line is at 3/3, and the card continues it with one decisive run instead of archiving it: §5, plus three amendments.
  - P1: the native `native_read=starling_native` read.
  - P2: the key-address gap closed first. The recommended fix is a symmetric key-address margin, with its own finite controls and FIT.
  - P3: no RoPE arm and no transformer comparison.

  The run is gated on rung 0, P2 and the owner's hold release.

- **Murmuration read, 13 October (owner-approved). This is now the line's mechanism.** [Card #2032 6151093135](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6151093135), [pivot retarget 6151096178](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6151096178), [record](../labs/murmuration-design-2026-10-13/README.md).
  - **The read:** alignment, not selection. Each token reads its 7-token topological flock (K1), moves every neighbour's value into its own frame by the exact icosian quotient `c_t·c_j⁻¹`, and takes `Δ = K_h·(mean − v_t)`, the Vicsek/Kuramoto rule.
  - **One shared address per token,** used both to store and to query, which closes the key-address gap.
  - **An FBS ternary tree for O(log n):** parents hold the transported means of their children.
  - **No rank table, null score or rank hinge.** That rung-0b source stays on main, inactive, as evidence. K1 carries over.
  - **Gating:** its finite controls (§4) need a FIT review. The decisive run is gated on rung 0, the S1/S2 source and controls, and the owner's hold release.

## Next configuration — candidates after the Dose 1 decision (hypotheses; not registered, no compute)

[D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised) (18 October) makes float forward passes, gradients and optimizers legal in offline training. Softmax/squashing and matmul stay banned, and the served artifact must be D11 integer with a reported export gap. D37 does not help every recorded failure:
- **Value/readout failure:** D37 helps most here. A coupling magnitude can now be learned.
- **Missing competitive selection signal** (rungs 1–3, the pivot): this stays structural. Softmax is still banned, so any credit must be rank- or hinge-shaped.
- **Key-address gap:** already closed by the shared per-token address.

Candidates, ranked:
- **C1 — float-trained coupling over the fixed Flock topology.** Selection stays the integer Flock rank.
  - Offline, train a float `K_h` per (head, level), optionally per edge type: `y = v_t + K_h·(Σ carried_j / n − v_t)`. This is an elementwise scalar update, not a GEMM.
  - Export `K_h` to a Gate trit (or a small integer applied by repeated addition, as `snap` already builds `n·r`). The Gate/snap tables are the integer boundary.
  - **Decisive measurement:** conditional top-1 on the Dose 1 slices against the discrete-trained same-spec artifact, plus the export gap.
  - **Controls:** a zero-coupling ablation and a random-coupling refit; ≥ 2 seeds or ≥ 200 fresh rows (D22); sealed fresh transfer.
  - **Dose 1 outcome:** a Relation KEEP extends C1 to the 25 relation trits. A negative closes only that configuration (D22) and makes C1 the test of whether a learned coupling magnitude rescues the read.
- **C2 — lawful soft-to-hard selection.** A rank-consistent utility hinge (rung 0b's `u = −⟨g, v⟩` ordering) plus exploration slots, annealed on a linear or piecewise schedule (no temperature exponential) down to the exact integer Flock rank.
  - **Decisive measurement:** source-in-flock rate and rank-1-when-kept, by distance bucket, against a hard-only arm.
  - **When it leads:** most relevant if Dose 1 shows no relation lift.
- **C3 — FBS ternary-tree murmuration (O(log n)).** Parents hold the transported sums of their three children. Worth it only after C1 shows that the coupling learns, and gated on a long-context bar with tokens/s reported first.

**Open decisions for the C1 sheet:**
- May an exported coupling exceed a trit (a small integer by repeated addition, which is multiplier-free under D11)?
- Is C1 the line's decisive run under D21, or a new line?

Every candidate enters only through a pre-registration, then `TEST FITNESS: FIT`, then owner release.

## A fair test needs

- A training signal for the addresses: soft-to-hard annealing works (rung 2).
- A readout that keeps the right source's information: under the murmuration read this is the transported flock mean, scored by flock coherence and the source-in-flock rate.
- Recall measured by distance bucket (so eviction is distinguishable).
- More than one read layer before any multi-hop claim.

## Open questions

- Soft-to-hard schedule against an auxiliary loss.
- Settled 13 October: one shared address per token (it serves both storing and reading), produced by the SpiralCore/FBS map of the state.
- K and M at long context.
- Collapse under the tied address: cell-occupancy entropy is a decisive-run diagnostic.
- A later arm: a chirality sign on the coupling (in the murmurations paper, parity sets the oscillation's sign).
