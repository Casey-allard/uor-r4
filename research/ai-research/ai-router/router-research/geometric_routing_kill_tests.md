# Geometric Routing Architecture --- Kill Test Protocol

Purpose: Define the minimal set of experiments required to determine
whether the geometric routing architecture is fundamentally viable.

This document is intentionally **minimal and falsification‑focused**.
The goal is to disprove the architecture quickly if it is wrong.

------------------------------------------------------------------------

# Core Hypothesis

Reasoning in large language models can be represented as **navigation in
a geometric routing manifold** rather than dense attention.

Proposed components:

1.  Hyperbolic routing manifold
2.  Hopf‑structured angular coordinates
3.  Phase transport
4.  Sparse event routing

The kill tests evaluate each component independently.

------------------------------------------------------------------------

# Test 1 --- Hyperbolic Embedding Stability

Goal: Determine whether semantic embeddings stabilize in hyperbolic
space.

Method: Train a simple embedding model

    f(x) → H⁴

Diagnostics:

-   geodesic neighbor preservation
-   clustering quality
-   curvature stability

Kill condition: If hyperbolic embeddings do not outperform Euclidean
embeddings for hierarchical structure.

------------------------------------------------------------------------

# Test 2 --- Hyperbolic Shell Routing

Goal: Verify radial routing structure.

Shell law:

    volume ∝ sinh³(r)

Diagnostics:

-   shell occupancy vs expected measure
-   neighbor preservation
-   routing entropy growth

Kill condition: If shell routing destroys local neighborhoods.

------------------------------------------------------------------------

# Test 3 --- Hopf Angular Partition

Goal: Verify angular routing stability.

Coordinates:

    (χ, θ1, θ2)

Diagnostics:

-   sector occupancy balance
-   semantic clustering
-   angular drift

Kill condition: If angular sectors collapse or behave randomly.

------------------------------------------------------------------------

# Test 4 --- Phase Kernel Utility

Compare:

1.  no phase
2.  raw phase kernel
3.  transported phase kernel

Kernel example:

    K_phase(Δα) = exp(κ cos(Δα))

Kill condition: If phase alignment produces no measurable improvement.

------------------------------------------------------------------------

# Test 5 --- Sparse Event Routing

Replace dense mixing with event routing:

    accumulate → threshold → emit

Use differentiable threshold

    s = sigmoid((u − θ)/τ)

Diagnostics:

-   compute cost
-   accuracy
-   sparsity

Kill condition: If routing collapses or activates nearly all cells.

------------------------------------------------------------------------

# Test 6 --- Hardware Efficiency

Compare prototype vs transformer baseline.

Measure:

-   FLOPs per token
-   runtime
-   memory traffic

Kill condition: If routing overhead cancels compute savings.

------------------------------------------------------------------------

# Required Success Conditions

All must hold:

1.  stable hyperbolic embeddings
2.  shell routing preserves neighborhoods
3.  Hopf angular routing stable
4.  phase alignment useful
5.  sparse event routing trainable
6.  compute cost reduced

Failure of any condition should trigger redesign.

------------------------------------------------------------------------

# Execution Order

1.  Hyperbolic embedding test
2.  Shell routing test
3.  Hopf angular test
4.  Phase kernel ablation
5.  Sparse event routing
6.  Hardware profiling

------------------------------------------------------------------------

# Final Principle

Validate:

geometry → routing → sparsity → phase

in that order.
