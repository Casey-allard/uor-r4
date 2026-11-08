# REPORT: one-model-end-to-end, deepseek-38

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Result:
[deepseek-36 REPORT](../deepseek-36/REPORT.md).

**The opening-weight result is unchanged: outcome row 5, no opening movement,
and weight 8 is closed.** This repair corrects only the label and arithmetic of
the pod-cost estimate.

## What did not work, first

- The fresh review at `957734b61` found that the launch receipt called its
  rate-derived estimate `cost_usd` and gave a value inconsistent with its own
  integer uptime.
- Recomputing `1610 * 0.563 / 3600` gives `0.2517861111111111`, not the recorded
  `0.2517863888888889`.
- No model reading, artifact hash, selector result or outcome classification was
  affected.

## What changed

The launch receipt now records `approx_cost_usd` and the exact formula
`uptime_seconds * usd_per_hour / 3600`, explicitly stating that this is a
rate-derived estimate rather than an invoiced charge. The prior repair REPORT
now states the same approximate `$0.252` used by the result REPORT and notebook.

No model, code, selector, outcome, notebook row or sealed artifact changed.

## Measurements

All four corrected/real result readings remain one click away in the
[deepseek-36 REPORT](../deepseek-36/REPORT.md) and the
[deepseek-37 REPORT](../deepseek-37/REPORT.md): selected step 256, guard 139,
first-target top-1/top-8 `0/0`, EOS top-1 `0`, free EOS stops 16, and zero
complete rows.

The pod ledger records `1,610` uptime seconds at `$0.563/hour`. Their product
over 3,600 seconds is `0.2517861111111111`, reported as about `$0.252`.

## Evidence

- Launch receipt:
  [launch.json](../deepseek-36/evidence/launch.json), field
  `pod.approx_cost_usd`.
- Reviewed runtime:
  `fd303c0a72d178a467ebed99b4fefc05347e193896fc74638b2023cac0b120a1`.
- Selected artifact:
  `eaaa3def2b3174d3f18aa50e654aeac9a53459920b4c389ab975c32fdc9ebb2f`.
- Seal: 91 SHA-256 entries, `BYTES=46,630,566`, manifest SHA-256
  `a6d21e64f51600f64129f66e40cfeed05b110792383a2010ab2e26411d85fae9`,
  `READBACK=verified`, `EXIT=0`.

## Checks run at the head

- Recomputed the approximate cost from the logged integer uptime and rate.
- Revalidated the launch receipt as JSON.
- Rechecked all 91 sealed entries with `sha256sum -c --status --strict`.
- Re-ran both REPORT checks, `loop pr-check --local`, `git diff --check` and the
  full repository gates before the final exact-head review.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84,
geometry and delivery rules were used directly.

## Still open

- The weight-8 configuration remains closed; no seed 2 or sweep follows.
- The first-token failure still requires an opening-formation or selection
  intervention.
- `m2-reply512` remains unavailable; no M2, useful-chat, mechanism or
  capability claim follows.

## Plain summary

The experiment result did not change. This repair makes the pod cost clearly an
estimate from the logged rate and time instead of presenting a slightly
inconsistent number as an exact charge.
