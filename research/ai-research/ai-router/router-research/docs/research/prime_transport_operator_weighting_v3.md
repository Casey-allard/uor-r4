# Prime Transport Operator Weighting V3

## Purpose

This step runs learned weighting over the first materially complete bounded
lawful operator algebra:

- `I`
- `T_b`
- `T_x`
- `T_c`
- `T_y`
- `T_z'`
- `T_r`

The active transport-side state is:

- `(phi, r, spin_h4, tau)`

No arbitrary candidates were introduced. No illegal transitions were scored.
Learning acts only over lawful operator-supported motion.

## Results

Bounded `v3` task comparison:

- `v3` reference:
  - accuracy `0.9978`
  - query accuracy `0.9878`
  - loss `0.0077`
- `r5` rebuilt reference:
  - accuracy `0.7503`
  - query accuracy `0.7927`
  - loss `0.4760`
- prior weighting `v8`:
  - accuracy `0.6909`
  - query accuracy `0.7016`
  - loss `3.4019`
- tau-aware weighting `v9`:
  - accuracy `0.6814`
  - query accuracy `0.7116`
  - loss `3.3806`
- radial-aware weighting `v11`:
  - accuracy `0.6837`
  - query accuracy `0.7001`
  - loss `2.2675`
- tiny transformer baseline:
  - accuracy `0.6958`
  - query accuracy `0.6707`
  - loss `0.7199`

Relative to `v9`:

- test accuracy improves slightly
- query accuracy gets worse
- loss improves substantially

So this is still not a clean overall behavior win.

## Generator Usage

For `v11`:

- `I = 0.0001`
- `T_b = 0.3237`
- `T_x = 0.3564`
- `T_c = 0.0000`
- `T_y = 0.0000`
- `T_z' = 0.0000`
- `T_r = 0.3198`

This is the first weighting run that makes real use of native radial motion:

- `T_r` usage rises from `0.0000` in `v8` and `v9` to `0.3198`

But the selector also collapses away from:

- `T_c`
- `T_y`
- `T_z'`

So the richer algebra is being used only partially.

## Lawful Motion Summary

- fraction changing radial class:
  - `0.3198`
- fraction changing fiber class:
  - `0.0000`
- fraction changing spin class lawfully:
  - `0.2865`
- illegal transitions scored:
  - `0.0`
- motion outside operator support:
  - `0.0`

## Required Honesty Section

### Was learning applied only over lawful operator support?

yes

### Were any illegal transitions introduced or scored?

no

### Did adding native radial transport improve operator behavior relative to the prior tau-aware weighting run?

no

## Interpretation

The radial generator is real and the selector uses it. But the behavior still
does not improve cleanly because the current radial law is probably too weak or
too crude relative to the rest of the transport geometry.

Evidence:

- `T_r` is used heavily
- loss drops a lot
- but query accuracy falls back below the tau-only run
- and the selector abandons the fiber-lift path `T_z'` entirely

That points more to:

- weak radial law

than to legality, missing `tau`, or missing support size.

## Output

The CSV is:

- [prime_transport_operator_weighting_v3.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_weighting_v3.csv)

## Single Next Step

Refine the radial law itself before more weighting work, because the bottleneck
now looks like weak radial transport rather than missing support, legality, or
recursive closure.
