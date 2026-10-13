# Prime Transport Framework Enforcement Step 1

## Purpose

This step implements the first missing framework primitive required by the
framework lock:

- native `spin_H`, or if full `spin_H` is not yet possible, an explicit native
  truncated `spin_h`

This is not a benchmark step.
This is not a tuning step.
This is not a transport redesign step.

It only makes spin identity explicit enough that the framework gate can refer
to a real spin-class object instead of a hidden proxy.

## What Was Added

`lock1` adds an explicit native truncated `spin_h` with fixed horizon:

- `h = 4`

The implementation does **not** rename `spin_bits`.
It computes a new object:

- `spin_h(state_t) = (a_t, a_{t+1}, a_{t+2}, a_{t+3})`

where each `a_k` is the admissibility bit produced by rolling the current
native transport law forward from the current native state.

Near the end of a bounded sample, if fewer than `4` future transitions remain,
the object is still present and is carried explicitly as:

- fixed declared horizon `h = 4`
- `valid_length < 4`
- padded bits with sentinel `-1`

So the truncation is explicit and mechanically represented, not hidden.

## Spin Semantics

### What `spin_h` means

`spin_h` is the bounded future admissibility word of the current native state
under the current native transport law.

More concretely:

- start from the current native state
- roll forward under the native transport law for up to `h` steps
- record the admissibility bit at each step

This is a forward object.
It is not the old backward-looking `spin_bits` buffer.

### How it is computed

For the current state `s_t` and remaining token path `x_t, x_{t+1}, ...`:

1. apply the current native transport step on `x_t`
2. record the produced admissibility bit
3. repeat for up to `h` steps
4. if the bounded sample ends early, keep `h` fixed and pad with `-1`

### What it preserves

It preserves:

- explicit finite-horizon future admissibility structure
- declared spin horizon
- enough structure for a lawful spin-class identity scaffold

### What it discards

It discards:

- any admissibility information beyond horizon `h = 4`
- the full exact `spin_H` object from the exact recursive-system layer

### Why this is the correct first framework primitive

The framework lock said the first missing primitive was explicit spin identity.

`spin_h` is the smallest honest implementation of that requirement because:

- it is native state
- it is future-oriented
- it is explicitly truncated
- it is mechanically computed from the current native transport law
- it is not a renamed hidden proxy

## Gate Scaffold Added

`lock1` also adds the first executable scaffold for lawful comparison checks.

For every audited source/candidate transport pair, the scaffold computes:

- radial class: `r`
- fiber class: `phi`
- spin class: explicit native `spin_h`
- composite compatibility class: ordered shared-prime masks over `{2,3,5}`

Current scaffold rule:

- pass only if source and candidate match exactly on all four class identities
- no lawful lifts are implemented in this step
- this step logs pass/fail only; it does not yet reject inside the live
  transport path

## Required Honesty Section

### Was exact native spin identity added?

no

### Was an explicit native truncated spin_h added and declared as such?

yes

## Verification Summary

The verification CSV is:

- [prime_transport_framework_enforcement_step1.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_framework_enforcement_step1.csv)

It reports, on a bounded sample:

- whether native `spin_h` is present for every audited state
- whether full gate inputs are computable for every audited source/candidate
  pair
- scaffold gate pass/fail counts

Observed verification counts:

- native `spin_h` present for audited states:
  - `256 / 256`
- gate inputs computable for audited source/candidate pairs:
  - `3072 / 3072`
- scaffold direct-comparison pass count:
  - `228 / 3072`
- selected side-transport pass count:
  - `88 / 512`
- fail counts:
  - radial mismatch: `1512 / 3072`
  - fiber mismatch: `2124 / 3072`
  - spin mismatch: `768 / 3072`
  - composite compatibility mismatch: `1695 / 3072`
  - incomplete gate input: `0 / 3072`

## What This Step Does Not Do

- it does not rebuild transport
- it does not benchmark performance
- it does not enforce rejection inside the live transport path yet
- it does not claim full exact `spin_H`

## Single Next Implementation Step After lock1

Implement one pre-scoring legality gate module that consumes the explicit class
tuple

- `(radial_class, fiber_class, spin_h, composite_compat_class)`

and rejects ordinary candidates before scoring whenever the direct comparison
rule fails and no named lawful lift has been declared.
