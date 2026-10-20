# 16-window zeta projection (#246)

The router's previous spectral path selected one of sixteen fixed 32-element
slices by raw L2 norm and returned a hard-coded eigenvalue vector. This was a
window/eigenvalue stub: the design matrix and the query's zeta phase never
participated in the projection.

The replacement is compiler-side and deterministic:

1. Build sixteen logarithmically spaced windows over `X_MIN=1e4` to
   `X_MAX=1e6`, each with 257 samples and `rho=4`.
2. Select the sparse gamma neighborhood using the prime-router rule
   (`SPARSE_RADIUS=0.3`) over the first 512 pinned zeta zeros.
3. Compute a reduced complex QR basis with modified Gram-Schmidt once per
   process, behind `OnceLock`.
4. Project the 512-dimensional query state into every basis, center/L2
   normalize the signal, and score the coefficient norm with the existing
   deterministic identity bias.
5. Compute covariance eigenvalues from six temporal subwindows. The
   covariance has rank at most six, so the implementation diagonalizes its
   equivalent 6x6 Gram matrix and returns the eight-value API shape.

The spectral grounder also blends a deterministic content signal into the
session state. This makes query text observable: identical session states can
select different windows. Sparse ranges are carried through `WeightedRoute`,
the routed result, corpus indexing, and retrieval instead of being rewritten
as the old 32-element ranges.

## Measurement

The focused quality harness is:

```bash
cargo test -p uor-r4-router --offline --test zeta_projection_quality -- --nocapture
```

It reports top-1 and mean reciprocal rank for the existing #245
content-reconnection cosine baseline and for the new routed retrieval path on
the same six-sentence/six-query fixture. The printed delta is an empirical
criterion, not a general retrieval guarantee. The current run reported:

```text
baseline top1=0.167 MRR=0.444
sparse-QR routed top1=1.000 MRR=1.000
delta top1=0.833 MRR=0.556
```

## GloVe decision

No GloVe dependency is reintroduced. The source path is absent/tombstoned and
the pinned Rust router already has deterministic zeta-seeded word vectors.
Adding an unavailable embedding source would make the build and artifact
reproduction less deterministic without supplying a validated quality gain.
