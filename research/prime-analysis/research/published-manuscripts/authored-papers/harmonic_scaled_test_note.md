# Scaled Harmonic Top-Layer Test

This experiment scales the earlier harmonic projection idea to a larger balanced sample.

## Dataset
- 300 primes in [100000, 500000]
- 300 semiprimes in [100000, 500000]
- all numbers survive the same small-prime filter chain up to 23

## Feature construction
For each number `n`, compute residues modulo the active primes, normalize to phases, and project into harmonic features.

## Main results
- A simple linear score built from the first 4 harmonic magnitudes achieves accuracy = 0.5083
- The separation between primes and semiprimes is not perfect, but it is nontrivial.
- The harmonic signal persists as the active cutoff grows from 11 to 23.

This does **not** prove a prime oracle, but it does support the idea that a composed top-layer state contains more information than the raw small-prime filter product alone.
