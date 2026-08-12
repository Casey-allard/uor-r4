# Prime Transport Geometry-Native Divisibility Bridge

## Purpose

Test whether reduced-schema-alignment transfer is missing not another chart
selector, but a small arithmetic transition layer between entangled and clean
factorizations.

This step keeps the main geometry-native sequence engine unchanged and reuses
the same reduced-alignment transfer setting as v7-v10. The only change is the
realignment mechanism presented to the geometry-native engine before readout.

## Mechanism

The chosen mechanism is a bounded divisibility bridge built from three pieces:

- prime factors as atomic role increments: `{2, 3, 5}`
- semiprime bridges as pairwise entanglement objects between the proxy role and
  a recovered transition increment
- small divisibility hubs that map local state factors into one transition
  increment before the bridged role is reconstructed

Two small hubs are used:

- query hub:
  - built from `style`, `r`, and tag parity between `speaker` and `topic`
  - represented as a semiprime over the corresponding prime-coded factors
- binding hub:
  - built from `b`, `next_return_gap`, and `speaker != topic`
  - represented as a semiprime over the corresponding prime-coded factors

For each token:

1. compute the lossy proxy role already used in v7
2. recover a bounded transition increment through the relevant divisibility hub
3. form a semiprime bridge between proxy-role prime and transition prime
4. recover the bridged role class from the bridge divisibility pattern
5. rebuild `referent_role` and `referent_entity` for the unchanged downstream
   geometry-native engine

This is distinct from chart selection because it does not choose among local
charts. It inserts one arithmetic transition layer between the entangled proxy
and the cleaner recovered factorization.

## Comparison

Reduced-alignment transfer line:

- v6 shared-schema transfer:
  - geometry-native accuracy `0.994750976562`
  - geometry-native query accuracy `0.971504330635`
- v7 reduced-alignment transfer:
  - geometry-native accuracy `0.479204952717`
  - geometry-native query accuracy `0.544108569622`
  - tiny transformer accuracy `0.480468750000`
  - tiny transformer query accuracy `0.502776086330`
- v8 single-rule chart realignment:
  - geometry-native accuracy `0.568933844566`
  - geometry-native query accuracy `0.624918460846`
- v9 multi-chart selection:
  - geometry-native accuracy `0.547449469566`
  - geometry-native query accuracy `0.597468376160`
- v10 chart calibration:
  - geometry-native accuracy `0.539407193661`
  - geometry-native query accuracy `0.569453179836`
- v11 divisibility bridge:
  - geometry-native accuracy `0.883501827717`
  - geometry-native query accuracy `0.969032287598`
  - geometry-native test loss `1.610011100769`
  - tiny transformer accuracy `0.477366715670`
  - tiny transformer query accuracy `0.510322570801`
  - tiny transformer test loss `2.036663055420`

## Interpretation

This is the first reduced-alignment recovery result that materially improves on
v8 rather than merely staying above v7.

What changed:

- v11 raises transfer accuracy from `0.568933844566` in v8 to
  `0.883501827717`
- v11 raises transfer query accuracy from `0.624918460846` in v8 to
  `0.969032287598`
- v11 reopens a very large gap over the tiny transformer baseline on the same
  reduced-alignment setup

What did not happen:

- v11 does not fully restore the v6 shared-schema transfer regime on overall
  token accuracy
- so the bridge is strong, but it should still be read as partial recovery
  rather than perfect schema-independent transfer

The result strongly suggests that the missing capability exposed by v7 was
transition infrastructure, not merely better local chart picking. Once the
model is given a small arithmetic bridge between the lossy proxy role and the
cleaner latent factorization, the reduced-alignment collapse is largely
repaired.

## Answers

### Does the divisibility-bridge mechanism improve on v8?

Yes, materially.

Relative to v8:

- transfer accuracy improves by `0.314567983151`
- transfer query accuracy improves by `0.344113826752`

Relative to v7:

- transfer accuracy improves by `0.404296875000`
- transfer query accuracy improves by `0.424923717976`

### Is the gain geometry-native in a meaningful sense?

Yes.

The gain comes from a bounded arithmetic transition layer expressed directly in
prime-coded factors, semiprime bridge states, and divisibility hubs. The main
geometry-native computation engine is unchanged, and no generic dense adapter
or transformer block is inserted.

### Does this support the stronger thesis that adaptive geometry may require transition infrastructure built from divisibility structure, not just fixed local charts?

Yes.

This is the strongest evidence so far for that stricter claim. v8-v10 showed
that local chart manipulation can help. v11 shows that a structured arithmetic
transition layer can help much more. That makes divisibility-mediated
transition infrastructure a more plausible missing ingredient than chart
selection alone.

### What is the next smallest stronger test if v11 works?

The next smallest stronger test should keep the same bounded transfer family
but ask whether the divisibility bridge can be selected or calibrated from a
short held-back support window, rather than using one fixed deterministic hub
rule. That would test adaptive bridge selection while keeping the arithmetic
mediation idea intact.

## Bottom Line

The v11 divisibility bridge is the first reduced-alignment recovery mechanism
that clearly outperforms the earlier chart-based fixes. The result supports a
stronger version of the geometry-native thesis: adaptive geometry may require
explicit transition infrastructure built from divisibility structure, not just
better local chart choice.
