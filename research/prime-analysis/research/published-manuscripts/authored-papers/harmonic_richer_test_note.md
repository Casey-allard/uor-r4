# Richer Harmonic Top-Layer Test

Dataset:
- 500 primes
- 500 semiprimes
- all numbers survive the same small-prime filter chain up to 23

Models compared:
1. harmonic-1 magnitude only
2. first 4 harmonic magnitudes
3. full harmonic embedding (12 harmonics, real/imag/magnitude, plus squared magnitudes)

Key result:
- the richer composed harmonic state outperforms the simpler projections
- this still does **not** yield a prime oracle
- but it does strengthen the case that the composed top-layer state contains nontrivial target-conditioned information beyond raw filter survival
