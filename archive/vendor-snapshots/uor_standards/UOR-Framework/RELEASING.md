# Releasing

## Prerequisites

- `CARGO_REGISTRY_TOKEN` org secret configured at `github.com/UOR-Foundation`
  (Settings > Secrets and variables > Actions). The token must have permission
  to publish `uor-foundation`.

## Release Process

1. Update `version` in the workspace root `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "X.Y.Z"
   ```

2. Regenerate the foundation crate and Lean 4 formalization, then commit:
   ```sh
   cargo run --bin uor-crate
   cargo fmt --all
   cargo run --bin uor-lean
   git add Cargo.toml Cargo.lock \
          foundation/Cargo.toml foundation/src/ \
          uor-foundation-sdk/Cargo.toml uor-foundation-sdk/src/ \
          lean4/
   git commit -m "Bump version to X.Y.Z"
   ```

3. Tag and push:
   ```sh
   git tag vX.Y.Z
   git push origin main --tags
   ```

4. The release workflow will automatically:
   - Validate the tag matches the `uor-foundation` Cargo.toml version
   - Run all checks (fmt, clippy, test, conformance)
   - Regenerate the foundation crate and verify no drift
   - Regenerate the Lean 4 formalization and verify no drift
   - Build the Lean 4 package with `lake build`
   - Verify `uor-foundation` packaging with `cargo publish --dry-run`
   - Create a GitHub Release with ontology artifacts
   - Upload Lean 4 cloud release build via `lake upload`
   - Publish `uor-foundation` to crates.io

## Published Crates

Two crates are published to crates.io in this release cycle:

1. `uor-foundation` — typed Rust traits for the ontology
2. `uor-foundation-sdk` — proc-macro ergonomics (`product_shape!`,
   `coproduct_shape!`, `cartesian_product_shape!`) for composing
   partition-algebra shapes from other `ConstrainedTypeShape` operands.

The SDK crate publishes **after** the foundation crate with a
wait-for-index step between them (see `release.yml`). This avoids the
classic crates.io ordering failure where the SDK's packaged manifest
depends on `uor-foundation = { version = "X.Y.Z" }` and the registry
rejects the SDK publish because the new foundation version is not yet
visible.

The internal crates (`uor-ontology`, `uor-codegen`, `uor-lean-codegen`,
`uor-conformance`, `uor-docs`, `uor-website`, `uor-clients`) are not
published.

## Lean 4 Package

The `uor` Lean 4 package is published via the Lean Reservoir
(reservoir.lean-lang.org). Reservoir automatically indexes this repo
because it has a root `lakefile.lean` and `lake-manifest.json`.

On release, `lake upload` attaches pre-built artifacts to the GitHub
Release so downstream users can skip building from source.

## Troubleshooting

- **Tag/version mismatch**: The workflow fails early if the tag version
  does not match `Cargo.toml`. Fix the version and re-tag.
- **Generated code drift**: If `git diff --exit-code foundation/src/ uor-foundation-sdk/src/` fails
  in CI, the committed generated code doesn't match the generator output.
  Run `cargo run --bin uor-crate && cargo fmt --all` locally and commit.
- **Version already published**: crates.io does not allow re-publishing
  the same version. Bump the version and create a new tag.
- **Lean 4 drift**: If `git diff --exit-code lean4/` fails in CI,
  the committed Lean code doesn't match the generator output. Run
  `cargo run --bin uor-lean` locally and commit.
