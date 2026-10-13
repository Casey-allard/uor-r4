# Prime Transport Geometry-Native Conflict Revision

## Purpose

Test whether a query/binding-conflict-triggered factor-level bridge revision
can beat the current best stronger-mismatch recovery result on the v12
reduced-alignment setting.

This step keeps the same stronger family, the same tiny bridge family, and the
same unchanged geometry-native engine. The only change is the trigger for
factor-level revision.

## Mechanism

The chosen mechanism is one conflict-triggered local revision window.

It works as follows:

1. choose the initial bridge from the same bounded family used in v13-v16
2. scan later local windows
3. open a revision window only when:
   - query tokens and binding tokens both appear locally
   - the current bridge and an alternative bridge disagree on both query-side
     and binding-side factor assignments at a sufficiently high rate
   - the alternative bridge also scores better on the local support score
4. revise the suffix by retaining shared prime factors and replacing only
   contradicted partner factors

This stays geometry-native because the trigger and repair are both expressed in
terms of prime-coded bridge factors, semiprime bridge states, and arithmetic
conflict structure. There is no generic learned controller.

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
- v15 coherence-triggered switch:
  - geometry-native accuracy `0.591052830219`
  - geometry-native query accuracy `0.637893617153`
- v16 GCD-style factor revision:
  - geometry-native accuracy `0.592540919781`
  - geometry-native query accuracy `0.651993811131`
- v17 conflict-triggered factor revision:
  - geometry-native accuracy `0.597052812576`
  - geometry-native query accuracy `0.643229186535`
  - geometry-native test loss `5.901119232178`
- v17 tiny transformer baseline:
  - accuracy `0.459914058447`
  - query accuracy `0.489458352327`
  - test loss `2.239838600159`

## Interpretation

The result is positive but not decisive.

What improved:

- v17 beats the fixed v12 bridge clearly
- v17 beats the tiny transformer baseline clearly
- v17 improves on v13 and v15

What did not improve:

- v17 does not beat v14
- v17 also remains below v16 on query accuracy and below v14 on both overall
  and query accuracy

So the honest reading is:

- explicit structural conflict is a better trigger than the generic coherence
  drop in v15
- but it still does not outperform the simpler midpoint switch from v14

That means the architecture direction remains plausible, but the exact
event-local repair rule is still not strong enough to become the best current
stronger-mismatch recovery mechanism.

## Answers

### Does conflict-triggered factor revision improve on the current best stronger-mismatch recovery?

No.

It does not beat v14:

- v14 accuracy `0.606119811535` vs v17 accuracy `0.597052812576`
- v14 query accuracy `0.660416662693` vs v17 query accuracy `0.643229186535`

### Does it remain geometry-native in a meaningful sense?

Yes.

The trigger is based on explicit query/binding conflict in prime-coded bridge
factors, and the repair retains shared factors while replacing only
contradicted partners. No generic dense adapter or transformer block is used.

### Does this strengthen the adaptive divisibility-mediated architecture direction?

Moderately.

It strengthens the case that structural contradiction is a more meaningful
signal than generic confidence drop, because v17 beats v15. But it does not
yet show that event-local factor repair beats the best simpler recalibration
strategy.

### What is the next smallest stronger test if v17 works?

The next smallest stronger test should keep the same conflict idea but move to
one explicit revision micro-window around the first high query/binding
contradiction cluster, rather than revising the full suffix after the trigger.

## Bottom Line

Conflict-triggered factor revision is a credible arithmetic repair mechanism,
but it is not the new best stronger-mismatch result. It improves on weaker
adaptive variants, yet the current best result on this harder setting remains
the simpler v14 midpoint switch.
