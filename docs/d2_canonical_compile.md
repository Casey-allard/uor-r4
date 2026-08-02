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
CI now runs a macOS/Linux differential compile of the pinned
SmolLM2-135M-Instruct revision on merge-group and main builds. Each runner
records SHA-256/byte manifests for the downloaded source and every emitted
bundle file; the compare job requires the manifests to match exactly. This
proves the canonical compiler path is byte-stable for the exercised target
and corpus size, without changing the shipped fixture.

The 500k/TLA7 fixture was re-run under `TLESS_CANONICAL_DETERMINISTIC=1` on
2026-08-02. It produced the existing 1,346,836-byte container and every
recorded κ unchanged, so the re-pin is an explicit canonical-mode adoption
with no artifact-byte delta. Gate E now invokes that mode and is required on
Linux as well as macOS; accelerated legacy teacher builds remain a separate,
non-canonical local-iteration path.
