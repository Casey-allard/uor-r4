# Prime Transport Geometry-Native Grown Repair

## Purpose

Test whether contradiction-grown regional repair can outperform the current
best stronger-mismatch recovery by letting the repair region expand until
arithmetic contradiction stabilizes.

This step keeps the same stronger v12 family, the same tiny bridge family, and
the same unchanged geometry-native engine. The only change is how the repair
region boundaries are discovered.

## Mechanism

The chosen mechanism is contradiction-grown regional repair.

It works as follows:

1. choose the initial bridge from the same bounded family used in v13-v19
2. detect the first strong query/binding contradiction cluster
3. seed a small local repair window
4. grow that window outward while arithmetic contradiction remains above a
   bounded stabilization threshold
5. inside the final grown region:
   - retain shared prime factors where consistent
   - replace contradicted partner factors
6. outside the grown region:
   - keep the current bridge unchanged

This remains geometry-native because both detection and repair are expressed in
prime-coded bridge factors, semiprime bridge states, and arithmetic conflict
structure. No generic learned front-end is introduced.

## Comparison

Stronger reduced-alignment line:

- v12 fixed bridge:
  - geometry-native accuracy `0.552269339561`
  - geometry-native query accuracy `0.584205031395`
- v13 per-sequence calibration:
  - geometry-native accuracy `0.593377947807`
  - geometry-native query accuracy `0.633928596973`
- v14 midpoint switch:
  - geometry-native accuracy `0.606119811535`
  - geometry-native query accuracy `0.660416662693`
- v16 GCD-style revision:
  - geometry-native accuracy `0.592540919781`
  - geometry-native query accuracy `0.651993811131`
- v17 conflict revision:
  - geometry-native accuracy `0.597052812576`
  - geometry-native query accuracy `0.643229186535`
- v18 microrepair:
  - geometry-native accuracy `0.580117166042`
  - geometry-native query accuracy `0.636718750000`
- v19 mesorepair:
  - geometry-native accuracy `0.587890625000`
  - geometry-native query accuracy `0.645161271095`
- v20 contradiction-grown repair:
  - geometry-native accuracy `0.604437589645`
  - geometry-native query accuracy `0.643454015255`
  - geometry-native test loss `5.770006656647`
- v20 tiny transformer baseline:
  - accuracy `0.457682281733`
  - query accuracy `0.479310333729`
  - test loss `2.233343601227`

## Interpretation

This is a near-miss rather than a win.

What improved:

- v20 beats the fixed v12 bridge clearly
- v20 beats the tiny transformer baseline clearly
- v20 improves on the factor-repair line from v16-v19 in overall accuracy

What did not improve:

- v20 does not beat the current best v14 midpoint switch
- v20 is very close on overall accuracy, but still behind
- v20 is more clearly behind on query accuracy

Numerically relative to v14:

- accuracy gap: `0.001682221889`
- query-accuracy gap: `0.016962647438`

So the honest reading is that contradiction-grown repair gets close enough to
support the boundary-discovery hypothesis in part, but it does not fully solve
it. Discovering a variable repair region helps overall accuracy, yet the
result still does not surpass the simpler midpoint switch.

## Answers

### Does contradiction-grown repair improve on the current best stronger-mismatch recovery?

No.

It does not beat v14, although it comes quite close on overall transfer
accuracy.

### Does it remain geometry-native in a meaningful sense?

Yes.

The mechanism still relies on arithmetic contradiction, prime-coded bridge
factors, semiprime bridge states, and retained-versus-replaced factor repair
inside the unchanged geometry-native engine.

### Does this strengthen the adaptive divisibility-mediated architecture direction?

Moderately.

It strengthens the case that boundary discovery matters more than fixed repair
scale, because v20 improves over the fixed micro/meso repair variants and gets
closest to v14 among the factor-repair family. But it does not yet beat the
best existing stronger-mismatch recovery.

### If it fails, does that imply the next step should be a learned regional schema-induction stage rather than more handcrafted repair rules?

Not conclusively, but it points in that direction.

Since v20 gets close without winning, the next smallest stronger test should
likely move beyond hand-picked growth rules toward a bounded learned or
semi-learned regional schema-induction/calibration stage, rather than more
manual repair-window heuristics.

## Bottom Line

Contradiction-grown repair is the strongest factor-repair variant so far, but
it still does not beat the simpler v14 midpoint switch. That means boundary
discovery matters, yet handcrafted regional repair rules are likely nearing
their limit on this stronger mismatch line.
