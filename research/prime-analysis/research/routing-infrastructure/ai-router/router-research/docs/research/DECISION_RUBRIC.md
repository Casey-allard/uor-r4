# Decision Rubric

Use this rubric before promoting a route or opening a new major branch.

## Score Each Candidate On
- `Quality`
  - Does post-growth performance improve on the control route?
- `Runtime`
  - Does wall-clock improve or at least stay competitive?
- `Hardware Proxy`
  - Does the route plausibly reduce active compute, memory traffic, bucket explosion, or routing waste without hiding cost inside route collapse?
- `Transfer`
  - Does the effect survive on LM proxy or another non-synthetic benchmark?
- `Mechanistic Coherence`
  - Can we explain why the route should work in geometric terms?
- `Integration Plausibility`
  - Can the idea be embedded into current model pipelines without unrealistic engineering assumptions?

## Promotion Rule
- Promote only when a route either:
  - beats the control on quality and at least one of runtime or hardware proxy, or
  - materially improves runtime/hardware proxy while staying inside the configured quality tolerance and passing strict transfer-health review.
- For LM proxy transfer, do not promote a branch that concentrates into a near-degenerate routing distribution (`pmax_after` too high or bucket usage too low) even if MSE and runtime improve.
- For multi-seed LM proxy transfer, a mean pass is insufficient if any individual seed breaches the shell-health thresholds.

## Demotion Rule
- Demote when a route wins only on one narrow synthetic setting or cannot explain its own advantage after repeated review.
