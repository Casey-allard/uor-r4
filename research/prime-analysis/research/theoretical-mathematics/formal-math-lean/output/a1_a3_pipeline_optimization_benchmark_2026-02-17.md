# A1/A3 Pipeline Optimization Benchmark (2026-02-17)

## Scope
Optimized:
- `research/a3_channel_energy_uplift.py`
- `research/a1_smoothing_uplift_pack.py`

Changes applied in both:
- Shared cache reads (`--cache-read-dirs`) across theorem pipeline caches.
- Vectorized event-to-grid projection (`searchsorted` + `cumsum`) replacing per-point `bisect` loops.
- Vectorized envelope/check computations where applicable.
- Added timing telemetry and cache write stats in JSON artifacts.

## Runtime comparison

### A3 channel-energy uplift
- Pre-opt: `research/output/a3_channel_energy_uplift_2026-02-17_preopt.json`
  - wall time: `47.20s`
- Post-opt warm: `research/output/a3_channel_energy_uplift_2026-02-17_postopt.json`
  - wall time: `1.64s`
  - telemetry total: `0.9941s`
- Post-opt cold (`--no-cache`): `research/output/a3_channel_energy_uplift_2026-02-17_postopt_nocache.json`
  - wall time: `27.98s`
- Numeric parity vs pre-opt: exact equality for `uplift_constants` and `checks`.

### A1 smoothing uplift pack
- Pre-opt: `research/output/a1_smoothing_uplift_pack_2026-02-17_preopt.json`
  - wall time: `95.73s`
- Post-opt warm: `research/output/a1_smoothing_uplift_pack_2026-02-17_postopt.json`
  - wall time: `0.71s`
  - telemetry total: `0.0141s`
- Post-opt cold (`--no-cache`): `research/output/a1_smoothing_uplift_pack_2026-02-17_postopt_nocache.json`
  - wall time: `66.71s`
- Numeric parity vs pre-opt: exact equality for `affine_maps_train`, `decomposition_constants`, and `checks`.

## Additional check
- A2 stage already lightweight:
  - `research/a2_infinite_tail_uplift.py` probe wall time: `0.15s`

## Unified pack refresh
- Optimized-input refresh artifact:
  - `research/output/uplift_theorem_pack_2026-02-17_after_a1_a3_a4_opt.json`
- Status: `unified_pack_holds=true`.

