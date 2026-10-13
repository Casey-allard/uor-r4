# EVIDENCE_SUMMARY

This document tracks what has been empirically explored so far.

## 1. Routing Experiments

Initial routing experiments explored geometric compatibility routing
versus dense attention baselines.

Observed: - structured routing behavior emerges - compatibility kernels
produce sparse activation

Unknown: - long-range stability - scaling behavior

## 2. Embedding Stability

Experiments evaluated embedding stability within curved spaces.

Observed: - hierarchical datasets embed naturally in negatively curved
space - local neighborhoods remain stable

Risks: - training instability at larger scales

## 3. Sparse Event Routing

Threshold routing experiments tested sparse activation behavior.

Observed: - strong sparsity patterns possible - compute savings in
prototype systems

Unknown: - training convergence - gradient propagation stability

## 4. Phase Transport

Preliminary experiments suggest phase alignment may encode useful
relationships between routed states.

Status: - exploratory - not yet proven necessary

## 5. Hardware Implications

Prototype results suggest possible compute savings if sparse routing
scales.

However: - routing overhead must remain small - memory locality must be
preserved

------------------------------------------------------------------------

Current evidence is promising but insufficient to confirm the central
hypothesis.
