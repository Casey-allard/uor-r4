# Prime Transport Spin-First Reset

## Purpose

This note resets the prime-transport / grouped-recursion line around the
canonical summary in
[prime_research_summary.md](/Users/adminamn/AI-Research/prime_research_summary.md).

The organizing object is **not** the empirical `C^2` quotient. The organizing
object is the exact recursive dynamical system:

- recursive affine lift law
- odometer transport `j -> j + 1`
- phase-fiber-scale factorization `j -> (b, phi, r)`
- finite-horizon spin `spin_H` as compressed predictive state
- delayed visibility of new layers

Low-dimensional quotient attempts should be treated as downstream compression
questions built on top of that exact structure.

## Corrective Framing

The conservative research order is:

1. exact recursive affine lift
2. exact finite-depth state `(b, phi, r)`
3. finite-horizon spin `spin_H`
4. only then low-dimensional quotient attempts such as the empirical `C^2`
   backbone

That ordering matters.

The recent grouped-packet line was useful as a bounded recoverability test, but
it drifted too close to treating packet composition as the primary explanatory
object. The packet idea should now be treated as a subsidiary compression
hypothesis. It is not the canonical state description.

The exact state remains torus-valued at finite depth, and the main compressed
predictive candidate from the recursive summary is finite-horizon spin, not a
direct product of per-layer packet encoders.

## Formal Question

The next bridge question should be stated as:

Can the empirical shared `C^2` backbone be understood more effectively as a
quotient of finite-horizon spin dynamics than as a direct composition of
per-layer phase packets?

More concretely:

- construct the exact finite-depth state `(b, phi, r)` from the current
  admissibility pipeline
- construct `spin_H(j)` as equivalence of the length-`H` future along the
  transport orbit
- test whether `spin_H`, or `(b, spin_H)`, predicts or recovers the current
  `C^2` quotient coordinates `z(j)` more effectively than the currently weak
  per-layer packet families
- if coordinate recovery becomes credible, test whether the shared backbone law
  `z(j+1) ≈ A_* z(j)` is better explained as the image of a simpler transition
  on the spin-state side

This is still an empirical quotient question. It is not a theorem claim.

## Implemented First Pass

The first bounded spin-based comparison is:

- script:
  [tools/prime_transport/run_spin_h_c2_bridge.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_spin_h_c2_bridge.py)
- summary:
  [results/prime_transport_spin_bridge/spin_h_c2_bridge_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_spin_bridge/spin_h_c2_bridge_summary.csv)
- detail:
  [results/prime_transport_spin_bridge/spin_h_c2_bridge_detail.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_spin_bridge/spin_h_c2_bridge_detail.csv)

Current construction:

- source datasets:
  [results/prime_transport_grouped_packets/prime_transport_layer_packet_dataset_W30030.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_grouped_packets/prime_transport_layer_packet_dataset_W30030.csv)
  and
  [results/prime_transport_grouped_packets/prime_transport_layer_packet_dataset_W510510.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_grouped_packets/prime_transport_layer_packet_dataset_W510510.csv)
- exact orbit index `j`, base phase `b`, admissibility bit, and reference
  `C^2` coordinates `z(j)` come from the current checked-in dataset pipeline
- `spin_H(j)` is built exactly as the cyclic length-`H` future word of the
  admissibility bit sequence
- two simple state families are tested:
  - `spin_only`
  - `base_spin`, meaning `(b, spin_H)`
- recovery model:
  deterministic keyed mean predictor from state to `z`

This is intentionally small. It asks whether exact predictive-state classes
already bridge to the empirical quotient better than packet composition does.

## Packet Comparison

The relevant comparison point from the grouped-packet stage is:

- best within-scale packet result:
  `0.9865538737427051`
- best cross-scale packet result:
  `1.0`

Those packet results were already judged weak because absolute recovery stayed
near `1`.

## First Spin-Based Outcome

The best first-pass spin results were:

- best within-scale spin result:
  `spin_only` at `H = 8`
  with test error `0.9986928601567902`
- best cross-scale spin result:
  `spin_only` at `H = 8`
  with test error `1.0029823065163037`

Interpretation:

- the spin-first route is still the better **conceptual** bridge because it
  matches the canonical recursive summary and uses exact predictive-state data
  rather than ad hoc packet composition
- but this first keyed-state recovery is still weak as a quotient explanation
- it does **not** beat the stronger within-scale packet result
- and it transfers slightly worse than the best packet cross-scale number in
  this first form

So the current evidence is:

- packet-first composition remains weak
- spin-first is the correct next bridge to test
- but the presently implemented categorical `spin_H -> z` map is not yet strong
  enough to explain the `C^2` backbone

## Next Experiment Narrowing

The next narrowing step should stay spin-first, but move beyond keyed mean
lookup.

Recommended next experiment:

1. keep exact `spin_H` as the state source
2. test structured spin-side features, such as:
   - one-hot `spin_H`
   - `(b, spin_H)`
   - short transition windows on spin states
   - exact successor state `spin_H(j+1)`
3. fit a small linear or bilinear quotient map from spin-side features into the
   checked-in `C^2` coordinates
4. only after coordinate recovery improves, test whether a simple transition on
   the spin-side induces or approximates the shared `A_*` transport law

That keeps the research aligned with the recursive dynamical system rather than
optimizing a packet family detached from the canonical state description.

## Non-claims

This note does **not** claim:

- a prime oracle
- an exact closed-form theorem
- that `C^2` is the true state
- that `spin_H` already explains the backbone in its current tested form

The empirical `C^2` law remains a downstream compressed quotient candidate.
The exact recursive system and finite-horizon spin remain primary.
