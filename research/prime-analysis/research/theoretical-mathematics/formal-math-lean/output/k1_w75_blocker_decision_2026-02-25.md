# K1 W75 Blocker Decision (2026-02-25)

## Objective
Decide whether the current multimode source-shape route can support the single open blocker:
- `K1SourceNonCircularProvider.theorem_term`

## Completed Evidence
| file | zeros_used | x-window | max_modes | best_mode_count | tail_sup/amp_total | eta | tag |
|---|---:|---:|---:|---:|---:|---:|---|
| `k1_w74_multimode_phase_probe_2026-02-25_beta062_x1e21_1e24_m30.json` | 30 | [1.0e+21, 1.0e+24] | 16 | 13 | 0.638307 | 0.197496 | near_strict_multimode_tail_dominance_finite_range |
| `k1_w74_multimode_phase_probe_2026-02-25_beta062_x1e21_1e24_100k.json` | 100000 | [1.0e+21, 1.0e+24] | 8 | 8 | 1.679446 | 0.108792 | weak_multimode_dominance_finite_range |
| `k1_w74_multimode_phase_probe_2026-02-25_beta062_x1e21_1e24_100k_m16.json` | 100000 | [1.0e+21, 1.0e+24] | 16 | 16 | 1.479682 | 0.055528 | weak_multimode_dominance_finite_range |
| `k1_w74_multimode_phase_probe_2026-02-25_beta062_x1e23_1e25_100k.json` | 100000 | [1.0e+23, 1.0e+25] | 8 | 8 | 1.617502 | 0.118465 | weak_multimode_dominance_finite_range |

## Decision
- Small-zero near-strict behavior is a false-positive signal for closure.
- High-zero (100k) runs do **not** reach near-strict dominance (`tail_sup/amp_total < 1`) on tested windows, even with 16 modes.
- Therefore this route is not theorem-grade stable yet and cannot currently close the blocker.

## Required Pivot
- Move from finite multimode fitting to direct theorem-source attack:
  construct/import explicit linear-phase witness assumptions without relying on low-zero finite fit dominance.
