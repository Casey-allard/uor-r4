# NEXT_CRITICAL_EXPERIMENTS

This document lists the experiments most likely to falsify the
architecture quickly.

## 1. Hyperbolic Embedding Benchmark

Test whether embeddings remain stable on large hierarchical datasets.

Failure condition: embedding distortion grows with scale.

------------------------------------------------------------------------

## 2. Shell Routing Preservation

Test whether routing preserves neighborhood structure across shells.

Metrics: - nearest neighbor retention - entropy across shells

Failure condition: routing collapses to central nodes.

------------------------------------------------------------------------

## 3. Phase Ablation Study

Compare:

-   routing without phase
-   routing with phase
-   routing with transported phase

Failure condition: phase adds no measurable signal.

------------------------------------------------------------------------

## 4. Sparse Training Stability

Train sparse event-driven routing networks end-to-end.

Failure condition: training diverges or collapses.

------------------------------------------------------------------------

## 5. Hardware Cost Simulation

Measure compute cost compared to transformer baseline.

Failure condition: routing overhead cancels sparsity gains.

------------------------------------------------------------------------

If the system passes these tests, further large-scale experiments become
justified.
