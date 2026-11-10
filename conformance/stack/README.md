# Stack contracts

This directory is the authoritative machine-readable description of a model
composition. A contract records the model shape, inputs, mechanisms, artifact
sections, reproduction commands, readings, boundary probes and explicit
non-claims.

The checked-in [model view](../../docs/model/STACK-CONTRACT.md) and
[machine view](../generated/stack-contracts.json) are generated. Do not edit
them by hand.

```sh
cargo xtask stack-contract --write
cargo xtask stack-contract --check
```

`--write` regenerates both views. `--check` fails when either committed view is
stale, when a contract is malformed, when a path does not resolve, when an
identifier is duplicated, or when a reading or probe names an unknown mechanism.

Adding a contract:

1. Copy an existing `.toml` file and give it a new lower-kebab-case `id`.
2. Pin the source and every input by commit or SHA-256.
3. Name an implementation and at least one evidence path for every mechanism.
4. State what each mechanism must preserve and what it does not establish.
5. Add semantic, surface, structural and parity readings separately.
6. Add boundary, metamorphic, parity, malformed and abstention probes where
   they apply.
7. Run `cargo xtask stack-contract --write`, inspect the generated diff, then
   run `cargo xtask stack-contract --check`.

A contract is not a capability claim. `PASS`, `FAIL`, `NOT_RUN` and
`UNAVAILABLE` remain result states; the contract defines what may be measured
and which result would answer each question.
