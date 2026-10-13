# Prime Transport Geometry-Native Microrepair

## Purpose

Test whether a very small conflict-centered repair window can outperform the
current stronger-mismatch recovery line by fixing only the locally broken
region instead of revising a whole suffix.

This step keeps the stronger v12 family, the same tiny bridge family, and the
same unchanged geometry-native engine. The only change is repair scope.

## Mechanism

The chosen mechanism is one conflict-triggered micro-window factor repair.

It works as follows:

1. choose the initial bridge from the same bounded family as v13-v17
2. detect one local query/binding contradiction cluster
3. open only a very small repair window around that cluster
4. within that micro-window:
   - retain shared prime factors where possible
   - replace only contradicted partner factors
5. leave the rest of the sequence under the original bridge

This is geometry-native because the trigger and repair remain entirely inside
prime-coded bridge factors, semiprime bridge states, and arithmetic
retain-versus-replace logic. No generic learned adapter is introduced.

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
- v16 GCD-style suffix revision:
  - geometry-native accuracy `0.592540919781`
  - geometry-native query accuracy `0.651993811131`
- v17 conflict-triggered factor revision:
  - geometry-native accuracy `0.597052812576`
  - geometry-native query accuracy `0.643229186535`
- v18 micro-window conflict-centered repair:
  - geometry-native accuracy `0.580117166042`
  - geometry-native query accuracy `0.636718750000`
  - geometry-native test loss `6.090361118317`
- v18 tiny transformer baseline:
  - accuracy `0.457589298487`
  - query accuracy `0.495560944080`
  - test loss `2.003650903702`

## Interpretation

This is a negative result for the micro-window hypothesis.

What remains true:

- v18 still beats the fixed v12 bridge
- v18 still beats the tiny transformer baseline

What fails:

- v18 does not beat v17
- v18 does not beat v16
- v18 is clearly below the current best v14 midpoint switch

Numerically:

- v14 accuracy `0.606119811535` vs v18 accuracy `0.580117166042`
- v14 query accuracy `0.660416662693` vs v18 query accuracy `0.636718750000`

So narrowing repair scope did not help. The likely lesson is that, once the
stronger mismatch is detected, the disrupted bridge state extends beyond a tiny
local contradiction cluster. Repairing only the local cluster is too weak.

## Answers

### Does micro-window repair improve on the current best stronger-mismatch recovery?

No.

It does not beat v14 and it also does not beat the stronger factor-revision
line from v16/v17.

### Does it remain geometry-native in a meaningful sense?

Yes.

The mechanism still uses explicit arithmetic conflict detection, prime-coded
bridge factors, and factor-level replacement inside the existing
geometry-native engine.

### Does this strengthen the adaptive divisibility-mediated architecture direction?

Only weakly.

It confirms that local structural repair is a coherent geometry-native idea,
but it does not support micro-window repair as the right scope. The stronger
results still come from broader recalibration or broader revision.

### What is the next smallest stronger test if v18 works?

The next smallest stronger test should move in the opposite direction from
micro-repair: use one bounded conflict-centered meso-window revision around the
first contradiction cluster, larger than the local cluster but smaller than a
full suffix.

## Bottom Line

Micro-window repair is not the missing capability on this stronger mismatch
setting. It stays above the fixed v12 bridge and above the transformer
baseline, but it weakens relative to broader revision and switching schemes,
and it does not beat the current best v14 midpoint switch.
