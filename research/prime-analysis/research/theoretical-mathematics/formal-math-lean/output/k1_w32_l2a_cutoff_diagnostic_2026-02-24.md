# K1 W32 L2A Cutoff Diagnostic (2026-02-24)

## Objective
Test whether the current L2A split cutoff can actually produce a non-empty finite band at the locked endpoint range `x in [10^21, 10^30]`.

## Method
Used:
- `/Users/adminamn/Documents/New project/research/k1_l2a_split_ledger.py`
- zero list `/Users/adminamn/Documents/New project/research/data/zeta_zeros_odlyzko_100k.json`
- locked `N=256`, `gamma_256 = 478.942181535`.

Runs:
1. Source-backed policy: `T(x) = max(gamma_256, exp(2*omega(x)))` (omega from VK model).
2. Exploratory policies: `T(x)=x^0.5` and `T(x)=x^0.2` for sensitivity (not source-locked).
3. Endpoint-valid scan and candidate: `T(x)=x^p` with `p` scan around threshold.

## Results

### A) Source-backed omega cutoff (blocking case)
Artifact:
- `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_split_ledger_2026-02-24.md`

Key facts:
- raw `exp(2*omega(x))` on `x in [1e21,1e30]` is only `27.73 .. 57.04`.
- enforcing `T >= gamma_256` forces `T(x) = 478.942...` for all sampled `x`.
- hence `k_cut = 256` for all sampled `x`.
- therefore `band_count = 0` identically.

Consequence:
\[
\mathcal E_{256,\le T}(x)\equiv 0,\qquad
\mathcal E_{>T}(x)\equiv \mathcal E_{256}(x)
\]
on the whole locked range.  
So the split gives no reduction at the endpoint; this is the current L2A bottleneck.

Crossing threshold:
- need `exp(2*omega(x)) >= gamma_256`.
- solved from current model: `x \approx 1.337e64`.
- this is far above the locked endpoint start `x1 = 1e21`.

### B) Exploratory x-power cutoffs (sensitivity only)
Artifacts:
- `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_split_ledger_xpow05_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_split_ledger_xpow02_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w32_cutoff_xpow_scan_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_split_ledger_xpow013_2026-02-24.md`

Observations:
- `x^0.5` cutoff swallows all available 100k zeros (`k_cut=100000`), so high-tail is numerically near-zero in this finite dataset.
- `x^0.2` gives non-degenerate split (`k_cut` from `17236` to `100000`), but this remains exploratory and finite-dataset dependent.
- threshold for non-empty band at `x1=1e21` is near `p > log10(gamma_256)/21 ≈ 0.1276`.
- candidate `p=0.13` is the smallest scanned value with non-empty band everywhere:
  - `k_cut` in `[295, 7766]`, so `band_count` in `[39, 7510]`.
  - fitted exponents in finite surrogate:
    - `eta_band ≈ 0.0213`
    - `eta_high ≈ 0.1160`
  - both exceed target `0.01` empirically.

## Mathematical interpretation
The blocker is now explicit:
- The source-backed omega cutoff does not create a usable finite-band term at `x >= 1e21`.
- Therefore L2A cannot close via this cutoff unless we either:
1. use a different truncation schedule justified by explicit-formula theorems for this endpoint range, or
2. directly bound the full omitted tail `E_256` without relying on a non-empty `E_{256,<=T}` band.

## Next immediate math step
Use endpoint-valid candidate schedule `T(x)=x^0.13` as the working L2A truncation template, and propagate explicit theorem constants (density + zero-free + truncation remainder) to obtain symbolic `C_256, eta_256` at `x >= 10^21`.
