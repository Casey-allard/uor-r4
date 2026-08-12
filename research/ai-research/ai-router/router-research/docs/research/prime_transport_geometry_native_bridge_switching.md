# Prime Transport Geometry-Native Bridge Switching

## Purpose

Test whether one bounded within-sequence bridge switch can recover more of the
stronger v12 mismatch loss than per-sequence bridge calibration alone.

This step keeps the stronger v12 family unchanged and keeps the same bounded
bridge family as v13. The only change is temporal adaptation: instead of one
bridge choice for the whole sequence, the controller can make one early bridge
choice and one mid-sequence re-choice.

## Mechanism

The chosen mechanism is one bounded mid-sequence recalibration window.

The bridge family is unchanged from v13:

- `v11_like`
- `v12_base`
- `hybrid`

For each test sequence:

1. split the sequence into two halves
2. choose the best bridge for the first half using the same support-window
   score as v13
3. choose the best bridge for the second half using a short support window on
   that later segment
4. concatenate the two recovered feature paths and run the unchanged
   geometry-native engine

This is still geometry-native because it only switches among a tiny fixed
family of arithmetic bridge rules using the existing readout as a bounded local
coherence signal. There is no generic learned adapter.

## Comparison

Stronger reduced-alignment line:

- v11 fixed bridge:
  - geometry-native accuracy `0.883501827717`
  - geometry-native query accuracy `0.969032287598`
- v12 stronger-mismatch fixed bridge:
  - geometry-native accuracy `0.552269339561`
  - geometry-native query accuracy `0.584205031395`
- v13 per-sequence bridge calibration:
  - geometry-native accuracy `0.593377947807`
  - geometry-native query accuracy `0.633928596973`
- v14 within-sequence bridge switching:
  - geometry-native accuracy `0.606119811535`
  - geometry-native query accuracy `0.660416662693`
  - geometry-native test loss `5.524695396423`
- v14 tiny transformer baseline:
  - accuracy `0.444289445877`
  - query accuracy `0.451562494040`
  - test loss `2.441468954086`

## Interpretation

This is another real partial recovery.

Relative to v13:

- transfer accuracy improves from `0.593377947807` to `0.606119811535`
- transfer query accuracy improves from `0.633928596973` to `0.660416662693`

Relative to v12:

- transfer accuracy improves by `0.053850471973`
- transfer query accuracy improves by `0.076211631298`

What it does not do:

- it still remains far below the v11 peak
- so one bounded switch point helps, but does not restore the stronger-mismatch
  line to the earlier moderate-mismatch recovery regime

The result is still meaningful because it points to the likely remaining
failure mode: the useful bridge can change over a sequence, and sequence-level
bridge choice alone is too coarse.

## Answers

### Does within-sequence bridge switching improve on v13 materially?

Yes, but modestly rather than dramatically.

It improves both overall transfer accuracy and transfer query accuracy, and it
reopens a slightly larger margin over the tiny transformer baseline.

### Does it remain geometry-native in a meaningful sense?

Yes.

The adaptation still happens through bounded switching among a tiny fixed
family of divisibility-mediated bridge rules. The main geometry-native engine
is unchanged, and there is no generic learned adapter or transformer block.

### Does this further strengthen adaptive divisibility-mediated geometry as the right architecture direction?

Yes, with the right caveat.

It strengthens the directional claim that adaptive divisibility-mediated
geometry is more promising than fixed bridge rules under stronger mismatch.
But it does not yet show a fully robust recovery, because the gap to v11
remains large.

### What is the next smallest stronger test if v14 works?

The next smallest stronger test should allow one event-triggered bridge update
instead of a fixed midpoint split, using the same small bridge family and the
same bounded support-window evidence. That would test whether the remaining
headroom comes from choosing the switch point more intelligently rather than
just allowing more switches.

## Bottom Line

One bounded within-sequence bridge switch improves on v13 and supports the idea
that stronger-mismatch recovery needs local temporal recalibration, not just
sequence-level calibration. The gain is real, but it is still only a partial
move back toward the v11 regime.
