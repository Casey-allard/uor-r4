# Research Strategy

## North Star
Design a geometry-driven routing algorithm that reduces the hardware footprint of large-model training and retrieval by making routing itself carry more of the organizational burden currently paid for with brute-force CUDA scale.

## Core Thesis
- Hyperbolic and polar geometry can create natural routing paths that reduce long-tail waste.
- A well-chosen chart can make local neighborhoods more semantically coherent and more cheaply addressable.
- If the routing geometry is structurally right, weighting, training stability, and retrieval locality should improve together rather than requiring separate fixes.

## Success Criteria
- Beat the current control route on post-growth quality.
- Beat or match the control route on runtime.
- Show a plausible hardware-efficiency story, not just a synthetic MSE improvement.
- Preserve a clear integration path into real model components such as expert routing, KV retrieval, memory lookup, or token neighborhood selection.

## Anti-Goals
- Do not chase synthetic wins that do not suggest a path into real model architectures.
- Do not re-run large sweeps without a mechanistic question.
- Do not promote branches on weak seed evidence or single-metric wins.
- Do not treat known-regressive settings as open unless the mechanism has changed.

## Current Best-Known Route
- `R5B`
- `sector_mode=phase4d`
- `phase4_dims=0,2,4,6`
- `time_pressure_lambda=0.0`
- radial scaling on

## What Counts as a Major Breakthrough
- A route that improves quality and runtime while preserving or improving transfer to LM proxy data.
- A hybrid routing mechanism that explains why the improvement occurs, not just that it occurs.
- A formulation that can be embedded into present-day model stacks without needing frontier-scale hardware just to evaluate it.
