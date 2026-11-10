# Stack contracts

This directory is the authoritative machine-readable description of a model
composition. A contract records the model shape, inputs, mechanisms, artifact
sections, reproduction commands, readings, boundary probes and explicit
non-claims.

Schema `/2` adds exact training and serving domains, the trained and served
contexts, the exported artifact schema, section owners and formats, structured
evidence, exact mechanism flags and dependencies, checkpoint sidecars,
load/cleanup orders, reading units and bars, and hashed executable probe
fixtures. Schema `/1` remains readable for historical records.

Artifact provenance is now declared explicitly:

- `required` means a new artifact must use `uor-r4.lut-stack/10` and bind the
  contract source SHA-256, contract semantic BLAKE3 and tokenizer SHA-256.
- `legacy-grandfathered` identifies a `/6`–`/9` artifact retained as exact
  evidence, including current or candidate decisions made before this policy.
  It cannot authorize a new export; reproduction requires
  `--legacy-artifact`.

The checked-in [model view](../../docs/model/STACK-CONTRACT.md) and
[machine view](../generated/stack-contracts.json) are generated. Do not edit
them by hand.

```sh
cargo xtask stack-contract --write
cargo xtask stack-contract --check
```

The repository's package binary is also callable as:

```sh
cargo run -q -p xtask -- stack-contract --check
```

`--write` regenerates both views. `--check` fails when either committed view is
stale, when a contract is malformed, when a path does not resolve, when an
identifier is duplicated, when a reading or probe names an unknown mechanism,
when a required flag does not match a command exactly, when dependency or
load/cleanup order is ambiguous, or when a fixture hash or fixture schema is
wrong.

Audit one contract end to end against whatever evidence is locally available:

```sh
cargo run -q -p xtask -- stack-audit \
  --contract conformance/stack/fbs-arm-r.toml \
  --config RUN/config.json \
  --checkpoint RUN/models/step-016384 \
  --artifact MODEL.uorlut \
  --tokenizer tokenizer.json \
  --chat-bin target/release/uor-chat \
  --out local/stack-audit.json
```

A check is `MATCH`, `MISMATCH`, `UNAVAILABLE` or `NOT_RUN`. A missing input is
never promoted to `MATCH`. With a chat binary, the audit runs the actual
`uor-chat --script` product path; with the artifact path it also checks every
registered probe fixture through that path.

New exports require all three identities:

```sh
cargo run -q -p xtask -- stack-contract-hash conformance/stack/example.toml

fbs-export \
  --stack-contract conformance/stack/example.toml \
  --stack-contract-semantic-blake3 <BLAKE3> \
  --tokenizer tokenizer.json \
  SNAPSHOT_JSON MODEL.uorlut
```

The resulting `uor-r4.lut-stack/10` artifact is refused by `uor-chat` when the
supplied tokenizer hash differs. Historical `/6`–`/9` artifacts remain
servable. Reproducing one is explicit:

```sh
fbs-export --legacy-artifact SNAPSHOT_JSON HISTORICAL.uorlut
```

Adding a contract:

1. Copy an existing `.toml` file and give it a new lower-kebab-case `id`.
2. Pin the source and every input by commit or SHA-256, and name both contexts,
   the training/serving domains and the exact artifact schema.
3. Name an implementation and at least one evidence path for every mechanism.
4. State what each mechanism must preserve and what it does not establish.
5. Add semantic, surface, structural and parity readings separately.
6. Add boundary, metamorphic, parity, malformed and abstention probes where
   they apply, each with a hashed JSON fixture and required execution layers.
7. Run `cargo xtask stack-contract --write`, inspect the generated diff, then
   run `cargo xtask stack-contract --check`.

A contract is not a capability claim. `PASS`, `FAIL`, `NOT_RUN` and
`UNAVAILABLE` remain result states; the contract defines what may be measured
and which result would answer each question.
