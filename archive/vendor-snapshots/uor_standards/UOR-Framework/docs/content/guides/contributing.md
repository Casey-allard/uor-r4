# Contributing

## Getting Started

```sh
git clone https://github.com/UOR-Foundation/UOR-Framework.git
cd UOR-Framework
cargo build
cargo test
```

## Development Workflow

Before submitting a pull request:

```sh
cargo fmt          # Format code
cargo clippy       # Check for lints (zero warnings required)
cargo test         # Run all tests
cargo run --bin uor-build         # Build ontology artifacts
cargo run --bin uor-docs          # Generate documentation
cargo run --bin uor-website       # Generate website
cargo run --bin uor-conformance   # Run full conformance suite
```

All checks must pass.

## Adding an Ontology Term

1. **Identify the namespace**: Select the appropriate namespace module in `spec/src/namespaces/`
2. **Add to the module function**: Add the class, property, or individual to the `module()` function
3. **Update counts**: Update the count assertions in `spec/src/lib.rs` tests
4. **Document it**: Add documentation coverage in `docs/content/` (or it will appear on the auto-generated namespace page)
5. **Run conformance**: Ensure all conformance checks pass

## Adding a New Namespace

A new namespace requires an amendment. Follow the pattern in existing namespace modules:

```rust
// spec/src/namespaces/my_ns.rs
use crate::model::{Class, NamespaceModule, Namespace, Property, Space};

pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "myns",
            iri: "https://uor.foundation/myns/",
            label: "My Namespace",
            comment: "Description.",
            space: Space::Bridge,
            imports: &[],
        },
        classes: vec![/* ... */],
        properties: vec![/* ... */],
        individuals: vec![],
    }
}
```

Then register it in `spec/src/namespaces/mod.rs` and `spec/src/lib.rs`.

## Documentation Standards

- Namespace reference pages are auto-generated — do not edit them by hand
- Prose pages use the `{@class}`, `{@prop}`, `{@ind}` DSL for ontology references
- Follow the Diataxis framework: concepts, guides, reference, tutorials
- See `conformance/standards/docs.md` for the full documentation standards

## Code Standards

- No `unwrap()` or `expect()` in library code
- Every `pub` item needs a doc comment
- Errors use `thiserror` in libraries, `anyhow` in binaries
- Run `cargo clippy -- -D warnings` before committing

## Pull Request Process

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-feature`
3. Make changes, run all checks
4. Open a PR against `main`
5. CI runs automatically: fmt + clippy + test + build + docs + website + conformance

## License

Contributions are licensed under MIT.
