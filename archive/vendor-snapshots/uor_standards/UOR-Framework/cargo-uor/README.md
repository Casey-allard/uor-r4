# cargo-uor

`cargo uor` — UOR Foundation tooling. Three subcommands: `check`, `inspect`, `explain`.

## Installation

From the workspace root:

```sh
cargo install --path cargo-uor
```

## Usage

### `cargo uor explain <iri>`

Look up the `rdfs:comment` for an ontology IRI. Accepts both prefixed and full forms.

```sh
$ cargo uor explain reduction:GroundingFailure
GroundingFailure — https://uor.foundation/reduction/GroundingFailure

A failure reason indicating that the morphism:GroundingMap could not produce
a ring image for the input.

$ cargo uor explain https://uor.foundation/cert/InhabitanceCertificate
InhabitanceCertificate — https://uor.foundation/cert/InhabitanceCertificate

A ComputationCertificate verdict primitive that decides carrier non-emptiness
on a type:ConstrainedType. ...
```

### `cargo uor inspect <unit>`

Print the v0.2.1 const accessors for a named compile unit.

```sh
$ cargo uor inspect matvec_q32
cargo-uor inspect: matvec_q32
  GS_7_SATURATION_COST_ESTIMATE = (n × k_B T × ln 2) — n from CompileUnit
  OA_5_LEVEL_CROSSINGS         = δ-level crossings to reach unitWittLevel
  BUDGET_SOLVENCY_MINIMUM      = bitsWidth(unitWittLevel) × ln 2
  fragment_classification      = (computed at uor_ground! expansion)
```

### `cargo uor check`

Walk a target crate's `src/` for `uor_ground!` invocations and run the offline pipeline driver.

```sh
$ cargo uor check ./src
cargo-uor check: scanning ./src
```

## v0.2.1 status

`inspect` and `check` are stub commands in v0.2.1 that print the accessor names.
The full in-process pipeline driver lands in a follow-up release. The `explain`
subcommand is fully functional and resolves any ontology IRI to its
`rdfs:comment`.
