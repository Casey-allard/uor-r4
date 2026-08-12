# Lemma A Channel Derivation Check

Generated: 2026-02-17T04:19:33.867603+00:00

## Identities Checked

- `Y11s`: `sqrt(3/(4*pi))*sqrt((1+gr/sqrt(gr^2+gi^2))/2)*(fi/sqrt(fr^2+fi^2))`
- `Y20`: `sqrt(5/(16*pi))*(3*((1-gr/sqrt(gr^2+gi^2))/2)-1)`
- `phase_vel`: `unwrap(arg(F_{n+1})-arg(F_n))`
- `su2_trace`: `2*cos(phase_vel/2)`

## Worst-Case Max Abs Differences

- `Y11s`: 3.915e-12
- `Y20`: 6.106e-16
- `su2_trace`: 0.000e+00
- `phase_vel`: 0.000e+00

## Per Base

- base=30 sample_count=266663 Y11s=3.915e-12 Y20=6.106e-16 su2_trace=0.000e+00 phase_vel=0.000e+00
- base=210 sample_count=228568 Y11s=3.915e-12 Y20=6.106e-16 su2_trace=0.000e+00 phase_vel=0.000e+00
- base=2310 sample_count=207789 Y11s=3.915e-12 Y20=6.106e-16 su2_trace=0.000e+00 phase_vel=0.000e+00
- base=30030 sample_count=191805 Y11s=3.915e-12 Y20=6.106e-16 su2_trace=0.000e+00 phase_vel=0.000e+00
