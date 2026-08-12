# Prime Transport Layer-Packet Dataset Schema

This directory stores aligned datasets for grouped-packet recovery experiments.

## Source of Truth

The aligned datasets here use:

- per-row transport index `j` along `n(j) = 5 + 6j`
- the current `C^2` quotient pipeline from
  `tools/prime_transport/c2_shared_astar_unseen_wheel_test.py`
- finite-depth layer phases reconstructed from the transport fiber index
  `t = floor(j / 35)`

For a wheel `W`, define:

- `L = W / 6`
- `fiber_mod = L / 35`
- `t = floor(j / 35)`

Then:

- `b_mod_35 = j mod 35`
- `fiber_index = t mod fiber_mod`
- the layer primes are the prime factors of `fiber_mod`
- `phi_m_index = t mod p_m`
- `phi_m_angle = 2π * phi_m_index / p_m`

This should be understood as a finite-depth layerization consistent with the
current base/fiber chart. It is a concrete research assumption for the packet
experiment, not yet a uniqueness theorem for the exact layered state.

## Columns

Common columns:

- `W`
- `j`
- `n`
- `layer_depth`
- `b_mod_35`
- `b_angle`
- `t_index`
- `fiber_mod`
- `fiber_index`
- `fiber_angle`
- `admissible_bit`

Layer columns:

- `phi_1_prime`, `phi_1_index`, `phi_1_angle`
- `phi_2_prime`, `phi_2_index`, `phi_2_angle`
- `phi_3_prime`, `phi_3_index`, `phi_3_angle`
- `phi_4_prime`, `phi_4_index`, `phi_4_angle`

Unused layer columns are left blank on shallower wheels.

Quotient columns:

- `z1_real`
- `z1_imag`
- `z2_real`
- `z2_imag`
- `z_radius`

## Current intended use

These datasets are for testing whether small per-layer complex packets composed
across depth can recover or explain part of the empirical shared `C^2`
transport backbone.
