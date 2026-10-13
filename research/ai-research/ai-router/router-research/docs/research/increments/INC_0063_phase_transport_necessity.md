# INC-0063: Phase-Transport Necessity

## Status
Complete on corrected screen rerun. The original negative read is obsolete.

## Trigger
`INC-0062` established the right coarse-address control:
- `phase4d_hopf_base` routes on the Hopf base
- common fiber phase `alpha` is excluded from the coarse address
- the route is healthy and fast

That means the project now has the right baseline to test the user’s core claim:
- geometry itself should force the phase shifts
- phase should survive because it improves routing, not because it is left in as a free score heuristic

## Why This Was Reopened
The original `INC-0063` negative was confounded by a route-allocation bug:
at `K=25`, the transport family effectively had dead `alpha` resolution, so the
transport law could not move addresses even when the equations were live.

The corrected rerun repaired the triplet-bin allocation and reran the same
proxy screen with address-diff auditing.

## Hypothesis
There is a measurable gap between:
- no fiber phase in the address
- raw phase coordinates in the address
- geometry-induced transported phase

If the theory corpus is directionally right, then a transported-phase law should beat both:
- the no-phase coarse-address control
- the raw-phase routed family

without unacceptable cost or route collapse.

## Minimal Scope
1. Keep the current cheap static proxy schedule fixed.
2. Compare three branches:
   - `phase4d_hopf_base`
   - `phase4d_hopf`
   - a new transported-phase route law
3. Do not reopen event-driven or gated-intelligence branches.
4. Measure:
   - proxy quality/runtime
   - route health
   - phase-specific diagnostics:
     - transported-phase coherence
     - phase ablation deltas
     - whether any gain is attributable to transported phase rather than generic angular access

## Acceptance
- transported phase beats both controls on at least one primary task metric
- route health remains acceptable
- cost stays inside the current routed band
- transported phase changes route addresses relative to the no-fiber-phase control
- phase survives as a geometry-induced mechanism, not as a free heuristic

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0063_phase_transport_screen_corrected.json`
- Screen analysis:
  - `results/analysis/inc0063_phase_transport_screen_corrected.json`
- Address-diff audit:
  - `results/analysis/inc0063_phase_transport_address_diff_corrected.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_101344.md`

## Result
2-seed screen means:
- `HOPF_TRANSPORT_L050`
  - `mse=0.003912672`
  - `total=5.785s`
  - `phase_transport_shift_abs_mean=0.0995`
  - `phase_transport_alpha_bins=2.0`
  - health pass
- `HOPF_TRANSPORT_L100`
  - `mse=0.003912672`
  - `total=5.750s`
  - `phase_transport_shift_abs_mean=0.1990`
  - `phase_transport_alpha_bins=2.0`
  - health pass
- `HOPF_TRANSPORT_L150`
  - `mse=0.003899971`
  - `total=6.125s`
  - `phase_transport_shift_abs_mean=0.2971`
  - `phase_transport_alpha_bins=2.0`
  - health pass
- `HOPF_BASE_K25_PHI`
  - `mse=0.003900382`
  - `total=6.381s`
  - health pass
- `HOPF_K25_BASE_PHI`
  - `mse=0.003902717`
  - `total=5.876s`
  - health pass
- `R0`
  - `mse=0.003916428`
  - `total=6.907s`
  - health fail

Address-diff audit against `phase4d_hopf_base`:
- `phase4d_hopf_transport` with `lambda in {0.5, 1.0, 1.5}` changed:
  - `2465-2469` sector assignments
  - `176` shell assignments
- sector-diff rate:
  - `0.9860-0.9876`
- shell-diff rate:
  - `0.0704`
- `phase4d_hopf` changed:
  - `2500` sector assignments
  - `80` shell assignments

## Reading
- The corrected transport law is mechanism-live on the proxy schedule.
- `phase_transport_alpha_bins=2.0` confirms that fiber-phase resolution is
  active at `K=25`.
- The transported phase shift scales with `lambda`.
- The route family now changes addresses materially relative to
  `phase4d_hopf_base`.
- `phase4d_hopf_base` remains the correct no-fiber-phase control.
- Pure `phase4d_hopf` still provides the best raw full-phase comparator.
- The corrected result is not yet a final proof that standalone transport is the
  best operational route law, but it does falsify the earlier “address-inert”
  negative.

## Decision
- Replace the original negative closeout with the corrected screen result.
- Treat standalone Hopf transport as mechanistically positive:
  - geometry-induced phase transport does move addresses once `alpha` bins are
    live
  - the old inertness claim was a data-path artifact, not a mathematical result
- Keep the branch at screen-stage evidence strength:
  - positive mechanism result
  - not yet the routed quality lead
  - sufficient to justify the coupled-field follow-up

## Failure Meaning
The corrected failure meaning changed too:
- if a future confirm still fails to convert the mechanism into a stable quality
  win, the project should narrow the operational claim
- but it should no longer say that standalone transported phase is inert or
  falsified on the proxy schedule
