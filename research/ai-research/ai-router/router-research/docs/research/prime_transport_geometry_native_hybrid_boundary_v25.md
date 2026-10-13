# Prime Transport Geometry-Native Hybrid Boundary V25

## Purpose

Test whether a two-objective / Pareto boundary scorer can improve the existing
two-region hybrid by balancing overall transfer quality against query-specific
quality without changing the hybrid architecture.

## Mechanism

The chosen mechanism is a Pareto boundary selector over a small candidate set.

It works as follows:

1. keep the same two-region hybrid and same local schema induction as v23/v24
2. compute two explicit objectives for each candidate boundary:
   - prototype-fit quality
   - query-sensitive contradiction quality near the boundary
3. keep only the non-dominated candidates
4. choose the Pareto candidate closest to the ideal tradeoff point
5. run the unchanged geometry-native engine under that boundary

This remains geometry-native because the scorer still operates on bounded
regional prototype fit and bridge contradiction structure rather than any
generic learned front-end.

## Comparison

Stronger reduced-alignment line:

- v22 fixed-midpoint hybrid:
  - geometry-native accuracy `0.646577358246`
  - geometry-native query accuracy `0.722751319408`
- v23 induced-boundary hybrid:
  - geometry-native accuracy `0.647507429123`
  - geometry-native query accuracy `0.728548288345`
- v24 contradiction-aware induced-boundary hybrid:
  - geometry-native accuracy `0.649925589561`
  - geometry-native query accuracy `0.719008266926`
- v25 two-objective / Pareto hybrid:
  - geometry-native accuracy `0.634393572807`
  - geometry-native query accuracy `0.697614133358`
  - geometry-native test loss `5.259162425995`
- v25 tiny transformer baseline:
  - accuracy `0.440011173487`
  - query accuracy `0.453838169575`
  - test loss `2.288605690002`

## Interpretation

This is a useful negative result.

What happened:

- v25 does not improve the tradeoff over either v23 or v24
- it lands below both of them on overall accuracy and query accuracy
- it still remains above the transformer baseline, but that is not the bar at
  this stage of the line

Numerically:

- relative to v23:
  - accuracy change: `-0.013113856316`
  - query-accuracy change: `-0.030934154987`
- relative to v24:
  - accuracy change: `-0.015532016754`
  - query-accuracy change: `-0.021394133568`

So the honest reading is that this bounded Pareto-style boundary scorer does
not reveal a better tradeoff frontier. Instead it weakens both objectives,
which suggests the remaining gap is not solved by simple explicit
two-objective balancing over this small candidate set.

## Answers

### Does v25 improve the combined tradeoff over v24?

No.

It is worse than both v23 and v24 on the headline metrics.

### If gains are tiny, does that confirm the current hybrid is near its bounded optimum on this family?

Yes, more strongly than before.

Since a direct Pareto-style tradeoff rule fails to recover a better point,
the current v23/v24 hybrid line now looks close to its bounded optimum on
this task family.

### Does it remain geometry-native in a meaningful sense?

Yes.

The scorer still operates entirely on bounded regional prototype fit and
bridge-contradiction structure over the same geometry-native regime family.

### What is the next smallest stronger test if v25 works or stalls?

Since v25 stalls, the next smallest stronger test is probably not another
hand-authored boundary rule. The next honest step would be either:

- stop this bounded refinement line and treat v23/v24 as the local optimum, or
- move to a more serious learned regional regime-field / boundary field if the
  goal is to keep pushing this family

## Bottom Line

v25 does not improve the hybrid tradeoff. That is strong evidence that the
current bounded two-region hybrid is already near its local optimum on this
family, and that additional gains will likely require a more serious learned
regional field rather than more handcrafted boundary-scoring rules.
