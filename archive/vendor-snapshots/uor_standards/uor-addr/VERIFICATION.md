# Verification & Validation — `uor-addr`

> V&V index. This document maps the conformance contract
> ([CONFORMANCE.md](CONFORMANCE.md)) onto a single reproducible
> acceptance gate: `just vv`. Each axis below names what is checked,
> how to reproduce it locally, and which conformance class IDs are
> satisfied by passing it.

## 1. The acceptance gate — `just vv`

`just vv` is the normative acceptance gate. PRs that fail any axis are
not mergeable. It runs the following in sequence, halting on the first
failure:

| #  | Axis                              | Command                                                | Classes covered           |
|----|-----------------------------------|--------------------------------------------------------|---------------------------|
| 1  | Format                            | `just fmt-check`                                       | (hygiene)                 |
| 2  | Lint (`-D warnings`)              | `just lint`                                            | (hygiene)                 |
| 3  | Workspace unit + integration tests| `just test`                                            | CS-*, CD-*, CT-*, CL-R-*  |
| 4  | Conformance suite (release)       | `just conformance`                                     | CD-*, CS-S01, CS-S02      |
| 5  | Analysis suite (release)          | `just analysis`                                        | CP-*                      |
| 6  | Replay round-trip (release)       | `just replay`                                          | CL-R-*                    |
| 7  | Runnable use-case examples        | `just examples`                                        | CT-T*, CT-E*, CL-R*       |
| 8  | Rustdoc (broken-intra-doc-links)  | `just doc-check`                                       | (hygiene)                 |
| 9  | Lean proofs                       | `just verify`                                          | CL-W,H,K,A,N,V,CT (formal)|
| 10 | Optional: live cross-validation   | `just cn` (skipped if `UOR_ADDR_LIVE` is unset)        | CN-*                      |

Total wall-clock budget on a 4-core dev machine: ≈ 4 minutes (axes 1–9).
Axis 5 runs at sample sizes pinned in [CONFORMANCE.md §CP](CONFORMANCE.md#cp--probabilistic-class--empirical-scaling)
(up to 1 000 000 samples) — the release profile keeps it under 60 s in
practice. Axis 6 (replay) runs in milliseconds; it exercises the TC-05
round-trip through `prism_verify::certify_from_trace` for one input
plus the 12-fixture baseline.

## 2. The V&V axes — what each one proves

### 2.1 Architecture (axes 1, 2, 8)

Format, lint, and rustdoc cross-link checks. `clippy --all-targets -- -D warnings`
denies all warnings, so any drift in idiom or doc cross-references fails
the gate. The rustdoc step uses
`RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links"` to catch the most
common doc-rot.

### 2.2 Typed-surface invariants (axis 3)

`cargo test --workspace` runs **361 tests across all realizations**, plus the 19,074-vector × 5-identity UCD 15.1.0 `NormalizationTest.txt` suite exercising the in-crate NFC normalizer.
Each shipped realization carries a dedicated published-spec test
vector suite:

| Suite | Count | Pins |
|---|---|---|
| lib unit tests | 114 | per-module structural invariants |
| `common_surface.rs` | 14 | `AddressInput` trait contract |
| `byte_identity.rs` | 8 | JSON 12-fixture byte-identity baseline |
| `conformance.rs` | 14 | JSON CD-\* + CS-S\* source-grep invariants |
| `typed_input.rs` | 16 | JSON typed-input bounds + structural distinction |
| `analysis.rs` | 8 | CP class (empirical scaling) |
| `replay.rs` | 3 | TC-05 round-trip |
| `jcs_rfc8785.rs` | 7 | RFC 8785 §3.2 + UAX #15 NFC + ECMA-262 |
| `sexp_conformance.rs` | 7 | Rivest §4.2/§4.3 + cross-format |
| `sexp_rivest_examples.rs` | 9 | Rivest §6 worked examples |
| `xml_c14n_1_1.rs` | 19 | W3C XML-C14N 1.1 rules §1.1.3 – §1.1.5 |
| `asn1_x690_der.rs` | 18 | ITU-T X.690 Annex A + DER canonical rules |
| `ring_amendment_43.rs` | 11 | Amendment 43 §2 layout coverage |
| `codemodule_ccmas.rs` | 14 | CCMAS grammar conformance |
| `schema_org_conformance.rs` | 19 | schema.org/Photograph + Article subtypes |
| `in_toto_statement_v1.rs` | 19 | in-toto Statement v1 + predicate variants |
| `variant_storage.rs` | 5 | ADR-048 cost-model variant |
| `all_realizations.rs` | 19 | cross-realization integration |

This axis pins:

- The ψ-residuals discipline (`CS-V01`, `CS-V02`) — the verb's term
  arena contains exactly the ψ_1/ψ_7/ψ_8/ψ_9 variants and no
  σ-residual primitive ops.
- The algebraic-closure shape (`CS-T01`–`CS-T03`, `CS-B01`–`CS-B02`) —
  71 disjoint Site constraints with the right capacity ceilings.
- Byte identity against the 12 reference fixtures (`CD-D02`, `CD-D03`).
- The pipeline invariants (`CD-D01`, `CD-I01a–d`, `CD-S01a`, `CD-W01`,
  `CD-G01`).
- Typed-input case distinction, structural equivalence, and bound
  enforcement (`CT-T01`–`CT-T05`, `CT-E01`–`CT-E04`, `CT-B01`–`CT-B04`,
  `CT-C01`, `CT-P01`–`CT-P02`).
- TC-05 replay round-trip (`CL-R00`–`CL-R02`).

If `cargo test` passes, the typed-iso surface is structurally correct
for every input in the fixture set.

### 2.3 Conformance — runtime invariant suite (axis 4)

`just conformance` runs four conformance suites at release-mode
speed (parametric extensions tractable):

- `cargo test -p uor-addr --release --test conformance` —
  JSON conformance (CD-\*, CS-S01, CS-S02) + the source-grep
  invariants over `json/verbs.rs`, `json/resolvers.rs`,
  `json/pipeline.rs`.
- `cargo test -p uor-addr --release --test common_surface` —
  common architectural surface (`AddressInput` trait + verb-arena
  ψ-Term composition + `AddressLabel` IRI / site-count).
- `cargo test -p uor-addr --release --test sexp_conformance` —
  S-expression conformance against Rivest 1997 §4.2/§4.3.
- `cargo test -p uor-addr --release --test variant_storage` —
  cost-model variant conformance against ADR-048 + QS-06.

Tests are named `<id>__<short_description>` so failures trace back
to a CONFORMANCE.md row by ID.

### 2.4 Analysis — empirical scaling (axis 5)

`just analysis` runs `cargo test -p uor-addr --release --test analysis`
covering the CP class. See [ANALYSIS.md](ANALYSIS.md) for the
mathematical statement, derivation of the sample sizes, and the
significance level for each test. The tests use a deterministic PRNG
seed so failures are reproducible.

### 2.5 Use-case examples — executable conformance demos (axis 7)

`just examples` runs the five `cargo run --example` invocations
under `crates/uor-addr/examples/`. Each example panics on a failed
invariant, so passing the axis is a structural requirement — the
examples function as runnable conformance demos for the use cases
in [README.md §Use-case examples](README.md#use-case-examples).
The four JSON examples cover content-address minting (CT-T\*),
structural equivalence (CT-E\*), typed distinction (CT-T\*), and
TC-05 replay round-trip (CL-R\*); the `sexp_address` example
demonstrates the S-expression realization end-to-end.

### 2.6 Lean proofs — universally quantified guarantees (axis 9)

`just verify` runs `cd uor-addr-lean && lake build`. The Lean library
imports `UOR.Prelude` from the
[UOR-Framework](https://github.com/UOR-Foundation/UOR-Framework) at
revision `main`. Theorems are listed in
[CONFORMANCE.md §CL](CONFORMANCE.md#cl--formal-class--lean-mechanised-theorems).
Two flagship theorems:

- **`UorAddr.KappaDerivation.kappa_determined_by_digest`**: the
  κ-label is a *function* of the digest — universally quantified
  over all 32-byte digest values. This pins
  [CD-D01](CONFORMANCE.md#cd--deterministic-class--per-input-byte-identity)
  to *every* possible canonical-form byte sequence, not just the
  empirical sample.
- **`UorAddr.AlgebraicClosure.euler_char_eq_site_count`**: the
  Euler-characteristic identity is mechanically verified — not just
  asserted at compile time via the `const _: () = { … }` block in
  `resolvers.rs`.

Lean theorems extend the conformance guarantee from "true at the
sample" to "true for all valid inputs" (universal quantification
over the typed-input domain).

### 2.7 Cross-validation — network axis (axis 10)

`just cn` is gated behind `UOR_ADDR_LIVE=1`. It runs the
`crates/uor-addr/tests/cross_validation.rs` integration tests, which
issue HTTP requests to `mcp.uor.foundation/tools/encode_address` and
compare the κ-labels byte-for-byte. CI does not require this axis (the
reference may be down) but the gate is provided so external operators
can re-establish CN-RC01/CN-RC02 confidence on demand.

## 3. Precision policy

`uor-addr` is verified to be valid for **arbitrary use cases to
arbitrary precision** in three converging senses:

1. **Universal quantification** (axis 9, Lean). The κ-derivation
   identity and algebraic-closure encoding are proved for *every*
   well-formed input by mechanically checked theorems — not by
   sampling. The Lean axis is the ceiling of precision: the property
   holds without bound on input domain or precision threshold.

2. **Cryptographic precision** (axis 4 + axis 5, sensitivity tests).
   CD-S01 and CP-A01 establish that distinct canonical-form bytes
   yield distinct κ-labels with collision probability ≤ `2^{-128}`
   across any feasible-N input set — i.e. SHA-256's full standard
   security margin. Tighter precision requires a different HashAxis
   selection from `prism::crypto` (e.g. `Sha512Hasher`,
   `Sha3_256Hasher`, or `Blake3Hasher`); each is a separate
   PrismModel declaration with its own κ-label width and a separate
   conformance contract.

3. **Empirical precision** (axis 5). The CP class establishes
   distributional uniformity of digest bytes at α = 0.001 over
   N = 10⁶ samples. Larger N tightens the achievable α toward
   `2^{-128}` asymptotically; the CP test consts in
   `tests/analysis.rs` are the dial. Sample sizes can be raised
   without a contract change provided the test still passes.

The composition of (1) and (2) is the operative answer to "is this
implementation correct for an arbitrary downstream use case at any
precision the downstream cares about?" — yes, up to the security of
SHA-256 itself, and universally quantified by the Lean axis.

## 4. Reproducing locally

```bash
# Full gate (≈ 4 min)
just vv

# Individual axes
just fmt-check
just lint
just test
just conformance
just replay
just analysis
just examples
just doc-check
just verify
UOR_ADDR_LIVE=1 just cn   # optional, requires network
```

The Lean step requires `lake` (provided by `elan`) and pins
`leanprover/lean4:v4.16.0` via `uor-addr-lean/lean-toolchain`. The
devcontainer at [.devcontainer/devcontainer.json](.devcontainer/devcontainer.json)
ships `elan` and `just`; running `just verify` once will install the
pinned toolchain on first use.

## 5. CI coverage

Mirror `just vv` in CI. The recommended pipeline:

| Job          | Command            | Required for merge?            |
|--------------|--------------------|--------------------------------|
| lint+test    | `just ci`          | yes                            |
| conformance  | `just conformance` | yes                            |
| analysis     | `just analysis`    | yes (release-mode, ≤ 60 s)     |
| replay       | `just replay`      | yes (TC-05 round-trip)         |
| examples     | `just examples`    | yes (runnable use-case demos)  |
| doc-check    | `just doc-check`   | yes                            |
| verify       | `just verify`      | yes (Lean build is hermetic)   |
| cn           | `just cn`          | no (live external dep)         |

If any required job fails, the contract has drifted. The PR must either
update [CONFORMANCE.md](CONFORMANCE.md) (declaring an explicit contract
change) or fix the code.

## TC-05 replay — GGUF and ONNX

Both container realizations expose the witness surface
(`*_with_witness` over C ABI and the WASM Component Model
`gguf-address-with-witness` / `onnx-address-with-witness`), minting a
`Grounded<AddressLabel>` that replays through
`prism_verify::certify_from_trace` to a byte-identical κ-label without
re-invoking SHA-256. The Lean side states the realization soundness
theorems (`UorAddr.Gguf.Theorems`, `UorAddr.Onnx.Theorems`):
`canonical_form_deterministic`, `canonical_form_is_unique`,
`kappa_label_admits_through_psi`, `distinct_commitments_yield_distinct_labels`,
`recurse_terminates_at_descent_bound`, plus `wire_format_round_trip`
(GGUF) and `topological_canonical_unique` + `external_data_dereference_total`
(ONNX). `lake build` compiles them sorry-free.
