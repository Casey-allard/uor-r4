# Prime Transport Radial Compatibility Audit

## Purpose

Audit the current native `r6` transition system to determine whether it is
comparing or transporting across incompatible radial/fiber/spin classes.

This is an audit step only. No model structure was changed.

## Concrete Proxy Definitions

All proxies are computed only from existing native state.

### 1. Radial length / fiber depth proxy

For a semiprime state `S = (r, semiprime)`:

- radial-length proxy = current chart radial bucket `r`
- fiber-depth proxy = `0` for repeated-prime semiprimes (`4, 9, 25`)
- fiber-depth proxy = `1` for mixed semiprimes (`6, 10, 15`)

The audit uses the combined radial/fiber class:

- `radial_fiber_class = (r, fiber_depth)`

For the current state this uses the current `r`.
For each candidate transition this uses the proposed next `r` plus the
candidate semiprime's fiber-depth proxy.

### 2. Spin class proxy

For a factor pair `(left, right)` under chart/spin state `(phi, spin_bits)`:

- `spin_class = (phi + sum(spin_bits) + left + right) mod 3`

For the current state this uses current `phi` and current factor pair.
For each candidate transition this uses proposed next `phi` and the candidate
`(anchor, partner)` pair.

### 3. Composite compatibility class proxy

For a semiprime `s` and the opposite composite `o`:

- `compat_class = shared-prime-mask(s, o)`

where the shared-prime mask is a 3-bit string over divisibility by:

- prime `2`
- prime `3`
- prime `5`

Example:
- `101` means shared factors via `2` and `5`, but not `3`

## Instrumentation Surface

Audited system:
- `r6` native joint anchor-plus-partner operator

Logged for every candidate transition per side (`query` / `binding`):
- current factor pair
- candidate `(anchor, partner)` pair
- candidate semiprime
- radial/fiber class proxies
- spin class proxies
- compatibility class proxies
- selection flag
- joint score/logit
- step correctness and loss

The audit CSV is:
- [prime_transport_radial_compatibility_audit.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_radial_compatibility_audit.csv)

## Was This Audit Measuring Real Incompatibility?

Yes, under the explicit proxies above.

## 1. Class Alignment Check

Selected `r6` transitions under normal evaluation:

- radial/fiber class mismatched: `50.82%`
- radial/fiber class matched: `49.18%`
- spin class mismatched: `70.90%`
- spin class matched: `29.10%`

All candidate transitions considered by `r6`:

- radial/fiber class mismatched: `67.21%`
- radial/fiber class matched: `32.79%`
- spin class mismatched: `66.67%`
- spin class matched: `33.33%`

Compatibility class:

- selected compatibility-class match fraction: `100.00%`

So the current issue is not opposite-composite compatibility drift under this
proxy. The visible mismatch is radial/fiber and spin class mixing.

## 2. Performance Correlation

Grouped over selected `r6` side-transitions during unfiltered evaluation:

### Radial/fiber class

- mismatched:
  - count `7806`
  - accuracy `0.7692`
  - query accuracy `0.7310`
  - mean loss `0.7942`
- matched:
  - count `7554`
  - accuracy `0.7061`
  - query accuracy `n/a` under this proxy bucket
  - mean loss `0.5248`

### Spin class

- mismatched:
  - count `10890`
  - accuracy `0.7306`
  - query accuracy `0.7605`
  - mean loss `0.4890`
- matched:
  - count `4470`
  - accuracy `0.7566`
  - query accuracy `0.7039`
  - mean loss `1.0824`

## Correlation Verdict

The raw per-selected-transition correlation is not clean.

- radial mismatch does **not** monotonically map to worse overall accuracy
- spin mismatch shows slightly worse overall accuracy, but better query
  accuracy and lower mean loss than the matched bucket

So the mismatch signal is real, but simple matched-vs-mismatched grouping is
too entangled with token type and transition context to act as a clean causal
performance explanation by itself.

## 3. Candidate Set Pollution

Using the radial/fiber and spin proxies together, the number of incompatible
candidates per selected side-transition is:

- average incompatible candidates per side-step: `5.3117` out of `6`

Distribution:

- `4` incompatible candidates: `3018`
- `5` incompatible candidates: `4536`
- `6` incompatible candidates: `7806`

This means the candidate set is heavily polluted by proxy-incompatible options.

## 4. Minimal Compatibility Filter Test

Constrained evaluation:

- keep the trained `r6` operator and readout fixed
- only allow candidate transitions whose radial/fiber class and spin class
  match the current state under the audit proxies
- if no candidate passes the filter, fall back to the model's original best
  choice

### Metrics

Before filter:

- loss `0.6617`
- accuracy `0.7382`
- query accuracy `0.7310`

After filter:

- loss `0.6512`
- accuracy `0.7436`
- query accuracy `0.7518`

Fallback rate:

- `50.82%`

So even this crude filter improves all three evaluation metrics despite being
active on only about half of side-transitions.

## Direct Answers

### Are incompatible radial/fiber classes being compared or transported?

Yes.

Quantified:

- `67.21%` of all candidate transitions are radial/fiber mismatches
- `50.82%` of selected transitions are radial/fiber mismatches

### Are incompatible spin classes being compared or transported?

Yes.

Quantified:

- `66.67%` of all candidate transitions are spin mismatches
- `70.90%` of selected transitions are spin mismatches

### Do mismatches correlate with worse performance?

Only weakly and inconsistently at the raw selected-transition level.

- the grouped matched-vs-mismatched performance splits are noisy
- but the candidate set is heavily polluted by incompatible options
- and the simple compatibility filter improves evaluation metrics

### Does a simple compatibility filter improve or degrade results?

It improves results.

- accuracy: `0.7382 -> 0.7436`
- query accuracy: `0.7310 -> 0.7518`
- loss: `0.6617 -> 0.6512`

### Is this likely a primary bottleneck in the rebuilt line?

It looks like a real bottleneck, but not the only one.

The strongest evidence is:

- incompatibility is frequent
- candidate pollution is very high
- a simple non-learned compatibility filter improves evaluation

The weaker point is:

- raw matched-vs-mismatched correlation is not clean enough to claim this is
  the sole bottleneck

## Honesty Check

### Are we currently comparing or transporting across incompatible radial/spin classes?

Yes.

Quantified on the current `r6` system:

- selected radial/fiber mismatches: `50.82%`
- selected spin mismatches: `70.90%`

This is frequent enough to treat class misalignment as a real structural issue
in the current native transport line.
