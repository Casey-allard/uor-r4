# ORDER for the #4059 discrete Clippy repair

Lab: codex. Item: [discrete-clippy-k16-coverage](../../../../roadmap/items/discrete-clippy-k16-coverage.md). References #3168. Source: [PR #4059](https://github.com/UOR-Foundation/uor-r4/pull/4059) by @Rajaykumar12 at `70628a91f7b0806392d37eaeaaac4d3694a59528`. Base: `c19edbb3f958bc0a00f9f8cb473a71506ee56bc2`.

## Cancel probe

The source head matches the eight-file contributor diff; the merge base with upstream main is 9465b92d6. The later main change is the timed integer parity test. PR #4060 supplies the #3168 roadmap item and scope. This adoption depends on its protected merge.

## Scope and done check

Adopt the eight Rust files and retain the author's commit. Compare default-feature library tests and strict all-target/all-feature Clippy with the unmodified sources. The Clippy run uses a local compiler stub for a host type check. Verify the driver pin. Preserve the existing hold guard and report other open #3168 work. Stop if a CUDA run, CI edit, pin change or a broader change in behavior would be needed.
