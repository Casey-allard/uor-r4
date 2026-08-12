# Prime Transport Geometry-Native Mesorepair

## Purpose

Test whether a moderate-sized contradiction-centered repair region can beat the
current best stronger-mismatch recovery result on the v12 reduced-alignment
setting.

This step keeps the same stronger family, the same tiny bridge family, and the
same unchanged geometry-native engine. The only change is repair scale: larger
than the v18 micro-window, smaller than the suffix-wide revisions.

## Mechanism

The chosen mechanism is a fixed-size meso-window centered on the first strong
query/binding contradiction cluster.

It works as follows:

1. choose the initial bridge from the same bounded family as v13-v18
2. detect the first local contradiction window where:
   - query tokens and binding tokens both appear
   - an alternative bridge wins locally
   - joint query/binding conflict exceeds a threshold
3. center a moderate-sized repair window on that trigger
4. inside that window:
   - retain shared prime factors where consistent
   - replace contradicted partner factors
5. outside that window:
   - keep the original bridge unchanged

This remains geometry-native because the repair is still expressed through
prime-coded bridge factors, semiprime bridge states, and retained-versus-
replaced arithmetic structure.

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
- v17 conflict-triggered suffix revision:
  - geometry-native accuracy `0.597052812576`
  - geometry-native query accuracy `0.643229186535`
- v18 micro-window repair:
  - geometry-native accuracy `0.580117166042`
  - geometry-native query accuracy `0.636718750000`
- v19 meso-window repair:
  - geometry-native accuracy `0.587890625000`
  - geometry-native query accuracy `0.645161271095`
  - geometry-native test loss `5.766983985901`
- v19 tiny transformer baseline:
  - accuracy `0.447963953018`
  - query accuracy `0.484281718731`
  - test loss `2.139752149582`

## Interpretation

This is another negative result relative to the current best v14 midpoint
switch.

What improved:

- v19 beats the fixed v12 bridge
- v19 beats the tiny transformer baseline
- v19 improves on the v18 micro-window repair

What did not improve:

- v19 does not beat v14
- v19 also does not beat v17 on overall accuracy
- it sits in the middle of the broader repair line rather than at the top

So the honest reading is:

- repair scale matters
- but a simple fixed meso-window still does not beat the simpler midpoint
  recalibration

The likely lesson is that the mismatch is not solved just by choosing a repair
region between “tiny” and “global.” The exact repair placement and repair logic
still matter more than window size alone.

## Answers

### Does meso-window repair beat midpoint switching?

No.

Relative to v14:

- accuracy drops from `0.606119811535` to `0.587890625000`
- query accuracy drops from `0.660416662693` to `0.645161271095`

### If not, how close does it get?

It stays competitive but clearly below v14:

- accuracy gap to v14: `0.018229186535`
- query-accuracy gap to v14: `0.015255391598`

### Does this confirm that mismatch operates at a regional (mesoscopic) scale?

Not strongly.

It suggests that repair scope larger than a micro-window can help relative to
v18, but the result does not show that a mesoscopic repair region is the right
dominant explanation. A simple fixed midpoint switch still performs better.

### What is the next step if v19 succeeds or fails?

Since v19 does not beat v14, the next smallest stronger test should stop
searching over repair scale alone and instead test an event-local meso-window
whose boundaries grow until arithmetic contradiction stabilizes, rather than a
fixed-size centered region.

## Bottom Line

Meso-window repair improves on the micro-window variant but does not beat the
current best stronger-mismatch recovery from v14. That means the branch still
supports adaptive arithmetic geometry directionally, but repair scale alone is
not the missing ingredient.
