# Rust Implementation Standards

## Edition and Toolchain

- Rust edition **2021** is required for all crates.
- The stable toolchain is pinned via `rust-toolchain.toml`.
- `rustfmt` and `clippy` components are required.

## Code Quality

All crates enforce the following deny list via `#![deny(...)]` in `lib.rs` or `main.rs`:

```rust
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]
```

CI enforces:
- `cargo fmt --check` — zero formatting deviations
- `cargo clippy -- -D warnings` — zero warnings

## Error Handling

- Use `thiserror` for library error types.
- Use `anyhow` for binary / application-level error handling.
- Never use `unwrap()` or `expect()` in library code.
- All public functions that can fail must return `Result<T, E>`.
- All public fallible functions must have an `# Errors` doc section.

## Documentation

- Every `pub` item must have a doc comment (`///`).
- Doc examples must compile and run correctly.
- `missing_docs` is denied at the crate level.

## Safety

- No `unsafe` blocks without a documented justification comment immediately preceding the block.
- No `std::process::exit` in library sources (only in `main.rs` / `bin/*.rs`).

## Testing

- Unit tests use `#[cfg(test)]` modules co-located with source.
- Integration tests go in `tests/`.
- Test names must be descriptive (`fn validates_ring_quantum_property`).
- All count assertions include comments citing the authoritative source.

## Cargo.toml

Every member crate must declare or inherit:
- `edition` (via `edition.workspace = true` or explicit `edition = "2021"`)
- `license` (via `license.workspace = true` or explicit `license = "MIT"`)
- `description`

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Edition Guide 2021](https://doc.rust-lang.org/edition-guide/rust-2021/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
