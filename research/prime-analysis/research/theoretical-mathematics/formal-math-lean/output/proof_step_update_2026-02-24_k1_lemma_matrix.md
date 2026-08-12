# Proof Step Update (2026-02-24): K1 Lemma Matrix (Derived vs Nullified)

This matrix is the one-pass classification for the current frontier.

## Source target
- Open non-circular target: `PrimeRiemannBridgeOscillatoryReduction.ZeroToCosSinPhaseTransfer`
- Locked provider target: `PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.K1SourceNonCircularProvider`

## Classification

| item | file | status | class | note |
|---|---|---|---|---|
| `zero_to_cos_sin_phase_transfer_of_linear_phase_witness` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | derived | non-circular from linear-phase witness assumptions |
| `zero_to_cos_sin_phase_transfer_of_imported_linear_phase_witness` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | derived | compositional from imported witness record |
| `zero_to_cos_sin_phase_transfer_of_log_linear_phase` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | derived | non-circular from log-linear assumptions |
| `zero_to_cos_sin_phase_transfer_of_imported_log_linear_phase` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | derived | compositional bridge |
| `zero_to_cos_sin_phase_transfer_of_linear_phase_only` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | derived | non-circular from linear-phase-only assumptions |
| `zero_to_cos_sin_phase_transfer_of_imported_linear_phase_only` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | derived | compositional bridge |
| `zero_to_cos_sin_phase_transfer_of_linear_phase_kernel` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | derived | kernel-route reduction |
| `zero_to_cos_sin_phase_of_rh` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean` | closed | nullified/circular | vacuous contradiction from `RHStatement` |
| `importedLinearPhaseOnlyResultsOfImportedPublished` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean` | closed | nullified/circular | derives `hRH` from imported published results and contradicts `hs_gt` |
| `schlage_interval_core_provider_nonempty_of_zero_to_cos_sin_phase_transfer` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean` | closed | derived | K1 source -> Schlage provider |
| `zero_to_cos_sin_phase_transfer_iff_nonempty_schlage_interval_core_provider` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean` | closed | equivalence lock | frontier pinned to one provider boundary |
| `rh_iff_nonempty_schlage_interval_core_provider` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean` | closed | equivalence lock | RH lock at Schlage boundary |
| `rh_iff_nonempty_k1_source_non_circular_provider` | `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean` | closed | equivalence lock | immutable single-target class lock |

## What is still open
- Exactly one non-circular construction:
  - provide a concrete instance term for
  - `PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.K1SourceNonCircularProvider.theorem_term`
  - with no RH-in-hypothesis and no RH-equivalent imported contradiction step.
