# Fixed Error pi(x)-Li(x) Probe (2026-02-24)

- Window: `[xmin, xmax] = [10000, 100000000]`
- Samples: `1200` (log-spaced unique integers)
- Sign changes in sampled sequence: `0`
- Positive count: `0`
- Negative count: `1200`

## Beta Ratios

| beta | max all | max tail (last 25%) | median tail | p95 tail |
|---:|---:|---:|---:|---:|
| 0.5000 | 2.363413e-01 | 1.105337e-01 | 7.173351e-02 | 9.503353e-02 |
| 0.5200 | 1.946781e-01 | 8.004073e-02 | 5.116680e-02 | 6.709935e-02 |
| 0.5500 | 1.469657e-01 | 4.932145e-02 | 3.053456e-02 | 4.073445e-02 |
| 0.5800 | 1.109468e-01 | 3.039209e-02 | 1.807464e-02 | 2.475467e-02 |
| 0.6000 | 9.198437e-02 | 2.200782e-02 | 1.280765e-02 | 1.774811e-02 |
| 0.6200 | 7.626289e-02 | 1.593653e-02 | 9.049960e-03 | 1.274378e-02 |

Interpretation: this is empirical finite-window behavior only; it is not a theorem-grade Ω result.
