# Cycle 3 working notes (scratch)

## Harness
- scratch crate c3 (path dep on repo training crate, repo [patch] pins, repo Cargo.lock copied)
- BPE (Rust, scratch): 256 bytes + 3840 merges; chunks: identifier runs, punct runs <=8, whitespace runs <=32, single space attaches forward; exact round trip
  - code: 25,483,108 train bytes -> 7,000,992 tokens (3.640 B/tok); valid 744,752 -> 206,844 (3.601)
  - wiki: 10,797,148 -> 3,044,794 (3.546); valid 1,121,681 -> 316,580 (3.543)
- throughput (4 cores): w256/T256/B16: 12.9 s/step 1 shard, 7.2 (2 shards), 4.7 (4 shards)
  w128/T128/B16, 4 concurrent single-shard runs: 3.04 s/step each (2,695 targets/s total)
- RSS 0.8-1.0 GB per run

## NoRead at init (measured, code valid, 32 windows, w128 T128)
- Dot s1: noread 0.060, nll_read 7.177, nll_noread 8.337
- Lorentz v0 (no offset) s1: noread 0.828, nll_read 7.639
- Lorentz v1 (offset) s1: 0.048, 7.134; s2: 0.071, 7.139
- step 100: dot 6.359 (noread .016); v0 6.362 (noread .388); step 200: dot 6.113 (.077), v0 6.320 (.551)

## Offset derivation
normalized input unit RMS; Glorot U(-L,L), L^2 = 6/(r+d); Var(q_j) = d L^2/3 = 2d/(r+d); E|q|^2 = 2rd/(r+d)
orthogonal equal-norm rows: z = 1 + E|q|^2; delta0 = arcosh(1 + 2rd/(r+d)): d=128 -> arcosh(86.33) = 5.151; d=256 -> arcosh(103.4) = 5.332
score = beta (delta - d): keys inside a hyperbolic ball of radius delta around the query outscore a zero NoRead logit.

## Integer arcosh (derived + checked in scratch int_arcosh_check.py, 4000 pairs, Q8 codes)
- identity: z - 1 = 1/2(|q-k|^2 - (q0-k0)^2), q0-k0 = (|q|^2-|k|^2)/(q0+k0)
- max |d error| vs float64 on the same codes: naive16 1.9e-2, naive24 8.7e-5, stable16 5.5e-4
- per candidate: <q,k> (same 64 products as Dot) + |k|^2, k0 stored at write + O(1) scalars + one table lookup
- table: arcosh(1+u) indexed by leading-bit exponent + 10 mantissa bits; u < 2^-30 -> 0; ~52K entries
