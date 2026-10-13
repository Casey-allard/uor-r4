# Prime Transport Grouped-Packet Recovery Stage 2

## Purpose

This note records the second-stage grouped-packet recovery experiment run on the
aligned prime-transport datasets.

The target remained fixed:

- recover the existing empirical `C^2` quotient coordinates `z = (z1, z2)`
- from per-layer phase packets built from the aligned layer-packet datasets

This is a bounded recovery test. It is not a derivation of the backbone and not
a runtime-routing result.

## Families Tested

The experiment compared a small controlled family of packet encoders and
composition rules:

1. `baseline_phase_conjugate_sum`
   - encoder: `[(e^{i phi}), (e^{-i phi})] / sqrt(2)`
   - composition: normalized uniform sum across layers
2. `single_mode_sum`
   - encoder: `[e^{i phi}]`
   - composition: normalized uniform sum across layers
3. `symmetric_sum`
   - encoder: `[(e^{i phi}), (e^{-i phi})] / sqrt(2)`
   - composition: normalized uniform sum across layers
4. `fourier4_sum`
   - encoder: `[e^{i phi}, e^{-i phi}, e^{2 i phi}, e^{-2 i phi}] / 2`
   - composition: normalized uniform sum across layers
5. `symmetric_concat_linear`
   - encoder: symmetric 2-component packet
   - composition: concatenate all layer packets, then fit a linear projection
6. `fourier4_concat_linear`
   - encoder: four-component Fourier packet
   - composition: concatenate all layer packets, then fit a linear projection
7. `fourier4_adjacent_interactions`
   - encoder: four-component Fourier packet
   - composition: concatenate layer packets plus adjacent complex interaction
     terms `p_m * conj(p_{m+1})`

The evaluation used:

- within-scale deterministic train/test split by `j mod 5`
- cross-scale transfer by fitting on `W = 30030` train rows and evaluating on
  `W = 510510`

## Artifact Locations

- runner:
  [tools/prime_transport/run_grouped_packet_family_recovery.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_grouped_packet_family_recovery.py)
- detail table:
  [results/prime_transport_grouped_packets/grouped_packet_family_recovery_detail.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_grouped_packets/grouped_packet_family_recovery_detail.csv)
- summary table:
  [results/prime_transport_grouped_packets/grouped_packet_family_recovery_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_grouped_packets/grouped_packet_family_recovery_summary.csv)

## Conservative Interpretation

This stage should be read as a pressure test of the grouped-packet hypothesis.

Interpretation labels:

- promising:
  a low-parameter packet family materially improves recovery and transfers
  across scales without obvious collapse
- weak:
  some richer packet families improve over the baseline, but recovery remains
  loose or transfer remains poor
- unsupported:
  richer packet families still fail to recover the `C^2` quotient in a useful
  way

The conclusion should be based on the saved summary table, not on prior
intuition.

## Observed Outcome

The best-performing family in this run was:

- `fourier4_adjacent_interactions`

with:

- best within-scale test recovery error:
  `0.9865538737427051`
- average within-scale test recovery error across `W = 30030` and `W = 510510`:
  `0.9932769368713525`
- cross-scale transfer test error from `W = 30030` to `W = 510510`:
  `1.0132677110237265`

For comparison, the original baseline family
`baseline_phase_conjugate_sum` gave:

- best within-scale test recovery error:
  `0.9996218920441539`
- cross-scale transfer test error:
  `1.0005003916073962`

So the richer adjacent-interaction family does beat the original baseline in
within-scale recovery, but only modestly. Recovery remains poor in absolute
terms, and cross-scale transfer remains weak.

The concatenation-based linear families did not help in this first pass.

## Interpretation

The conservative reading is:

- simple per-layer grouped-packet composition is **not yet sufficient** to
  recover the existing empirical `C^2` quotient in a useful way,
- richer local Fourier packets plus adjacent layer interactions improve the fit
  somewhat,
- but the recovery remains too weak to support a strong explanatory claim,
- and the transfer result does not currently support a stable cross-scale
  packet explanation of the backbone.

So the grouped-packet hypothesis should currently be labeled **weak / currently
unsupported at this simple packet-composition level**, not promising.

At the same time, the slight gain from the richer adjacent-interaction family
suggests that the failure is not completely random. The current packet family is
likely missing important mixed structure rather than merely lacking regression
capacity.

## Next Narrowing Step

The most useful next narrowing step is not a larger search. It is a more
structured packet family that explicitly includes the base/fiber coupling that
the current per-layer-only packets omit.

The clean next test is:

- keep the target fixed at the existing `C^2` quotient,
- keep the experiment family small,
- add **base-conditioned layer packets** or **joint base-layer Fourier
  features** so packet composition can see interactions between:
  - `b mod 35`
  - the layer phases `phi_m`
  - and possibly the global fiber index

If that still fails, then the present grouped-packet explanation should be
treated as substantially unsupported for the current `C^2` backbone.

## Non-claims

This experiment does **not** claim:

- exact recovery of the layered torus state
- a proved group law for admissibility
- that the empirical `C^2` backbone has been derived from layer packets
- that a runtime routing mechanism has been found

It is only a controlled recoverability test.
