# Integration Paths

## Why This Matters
The project only wins if the geometry can be inserted into real model systems in a way that reduces hardware pressure, not just if it produces a better synthetic benchmark curve.

## Path 1: Token-to-Expert Pre-Routing
- Use the geometry as a coarse router before expert selection.
- Goal: reduce the number of experts or candidate experts each token needs to consider.
- Why plausible: shell/sector addressing is already a discrete routing surface.

## Path 2: KV Retrieval Pruning
- Use geometric addresses to prune attention retrieval neighborhoods before dense similarity work.
- Goal: reduce KV bandwidth and lookup volume.
- Why plausible: a polar/hyperbolic route can act as a locality-preserving pre-index.

## Path 3: Learned Memory Addressing
- Replace or augment memory-key lookup with a geometry-native `(shell, sector, local slot)` address.
- Goal: cheaper retrieval and better locality for long-tail facts.
- Why plausible: current bucket/slot mechanism already resembles an addressable memory layer.

## Path 4: Hybrid Global-Local Router
- Use `phase4d` for coarse routing and a local geometry such as `complex2` inside the chosen neighborhood.
- Goal: keep global routing cheap while preserving local discrimination.
- Why plausible: current results suggest `phase4d` is strong globally while `complex2` may still be useful locally, especially if expressed as discrete complex multiplication through an imaginary-field neighborhood step rather than as the whole global router.
- Current read:
  - first screen failed on unseen-route exposure
  - `INC-0020` showed that a local convergence prior plus `local_min_k=2` rescues the path into a healthy routed-quality branch
  - current limitation is runtime, not viability

## Path 5: Training Data Neighborhood Routing
- Route examples or token windows through geometry-aware neighborhoods during training.
- Goal: reduce wasted updates by concentrating local relevance.
- Why plausible: the chart already appears to make nearby routed samples more label-coherent.

## Current Priority Order
1. Path 1: token-to-expert pre-routing
2. Path 4: hybrid global-local router, but only if the current quality branch can be simplified into a cheaper route
3. Path 2: KV retrieval pruning
4. Path 3: learned memory addressing
5. Path 5: training data neighborhood routing
