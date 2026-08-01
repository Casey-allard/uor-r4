# D2 canonical deterministic compile mode

Issue #265 adds a target-independent compiler mode for certificate-bearing
artifacts. Enable it with:

```bash
cargo run --release --bin r4 -- compile \
  --source /path/to/pinned-model \
  --canonical-deterministic
```

The mode is selected by `TLESS_CANONICAL_DETERMINISTIC=1` and currently does
the following:

- disables Accelerate/NEON/AVX2 teacher matmul in favor of the ordered scalar
  path;
- routes teacher `sqrt`, `exp`, `pow`, `sin`, and `cos` through the portable
  pure-Rust `libm` implementation;
- routes transformerless compiler softmax, power-of-two packing, projection
  normalization, and graph-cover normalization/entropy through the same
  portable math family.

`--exact-scalar` remains the faster local-iteration switch. It selects scalar
matmul only and does not claim cross-platform byte reproducibility.

The mode is compiler-side only; the deployed runtime contract is unchanged.
The remaining D2 acceptance work is a macOS/Linux differential compile of a
pinned checkpoint and corpus, followed by recording the mode in the artifact
certificate and re-pinning the canonical fixture under maintainer review.
