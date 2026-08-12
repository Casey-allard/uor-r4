# K1 W31 L2A Tail-Divergence Note (2026-02-24)

## Statement
For the unsmoothed oscillatory mode form currently used in the surrogate,
an absolute-value bound on the omitted tail from `N=256` cannot yield a uniform decaying majorant by itself.

## Reason
Each omitted mode contributes (up to phase) amplitude scale
\[
\asymp \frac{1}{\gamma_n},
\]
so naive absolute control uses
\[
\sum_{n>N}\frac{1}{\gamma_n},
\]
which diverges (slowly).

Convergent quantity:
\[
\sum_{n>N}\frac{1}{\gamma_n^2}
\]
is finite, but it does not directly bound the unsmoothed linear sum without additional cancellation/smoothing structure.

## Data check from local 100k zeros
Source:
- `/Users/adminamn/Documents/New project/research/data/zeta_zeros_odlyzko_100k.json`
- `gamma_256 = 478.942181535`

Partial tails (`n=257..M`) computed directly:

| M | sum 1/gamma | sum 1/gamma^2 |
|---:|---:|---:|
| 512 | 0.398919 | 0.000637 |
| 1,024 | 0.858306 | 0.001060 |
| 2,048 | 1.379892 | 0.001333 |
| 4,096 | 1.965098 | 0.001505 |
| 8,192 | 2.615097 | 0.001611 |
| 16,384 | 3.330875 | 0.001676 |
| 32,768 | 4.113273 | 0.001714 |
| 65,536 | 4.963010 | 0.001737 |
| 100,000 | 5.514356 | 0.001746 |

## L2A implication
To derive
\[
|\mathcal E_{256}(x)|\le C_{256}x^{-\eta},
\]
we must avoid a raw absolute tail sum and instead use one of:
1. Truncated explicit formula with remainder terms that are controlled by density inputs.
2. Smoothing/test-function weighting that makes the omitted spectral sum absolutely summable.
3. Signed cancellation arguments with explicit zero-density and zero-free envelopes.

Without one of those three, the L2 majorant derivation is structurally blocked.
