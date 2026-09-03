# UOR integration source audit — public findings

Snapshot: 2026-09-03T03:10:48.234699+00:00. Current R4 reference: `e627252e525201815169ffd8364184953a46018d`. This is source review and planning evidence. No upstream code was executed, built, tested, installed or merged. Runtime/benchmark verification is **NOT_RUN**.

## Coverage and identity

The public inventory covers **547 repositories**: all 18 public UOR-Foundation repositories, all 9 public Hologram-Technologies repositories, 438 public `auser` repositories and 82 public `afflom` repositories. Every public organization default branch SHA was resolved. Personal rows marked `DISCOVERY_ONLY` received metadata triage, not a full source audit. Source manifests record 80 explicitly fetched source files at pinned commits; selected trees and READMEs are cached separately. No queried tree was truncated. The current UOR-Foundation/uor-r4 repository is explicitly excluded from import.

A separate **restricted local-only** manifest contains 23 authenticated-accessible private repositories; none of their names, descriptions or code appears in this public report. One private repository is empty and has no commit to resolve. All other accessible organization heads resolved.

Alex Flom is explicitly identified by his own [GitHub profile](https://github.com/afflom), which displays Alex Flom, UOR-Foundation and [his personal site](https://www.alexflom.com/). Thus `afflom` is verified primary attribution, not a guessed identity. `auser` is the exact user-requested handle; its [profile](https://github.com/auser) displays Ari.

## Existing dependencies and compatibility

| Dependency | R4 pin | Upstream default head | Consequence |
|---|---|---|---|
| uor-addr | `165b51e3e2113ee5d032730cde709335d4fe9b60` | same | Already current; extend the existing adapter rather than introducing another address implementation. |
| Framework foundation/SDK/verify | `51c01382200b0179d6640b07e9c8119364ab69a1` | same | Keep the `[patch.crates-io]` source unification; it prevents duplicate incompatible Rust type identities. |
| uor-matmul | `b13c98449948174f590e337c4dc25dfc394a07d0` | `3cc5882f210667f9ac00fd8c02c5b5957b493f5d` | Upstream is 7 commits ahead, 10 changed paths returned; a separate numeric/API compatibility decision is required before repinning. |

Current R4 already calls `uor_addr::json::address_blake3` for model/attention manifests, `uor_addr::cbor::address_blake3` for graph/TLA boundaries, and `PrismModel`/`run_route`/`verify_trace` for the existing UOR facade. [Current manifest](https://github.com/UOR-Foundation/uor-r4/blob/e627252e525201815169ffd8364184953a46018d/Cargo.toml), [existing UOR bridge](https://github.com/UOR-Foundation/uor-r4/blob/e627252e525201815169ffd8364184953a46018d/src/tless_uor.rs#L1404), [attention identity](https://github.com/UOR-Foundation/uor-r4/blob/e627252e525201815169ffd8364184953a46018d/crates/uor-r4-core/src/recursive_geometric_attention.rs#L4360).

## The κ contract must be typed

κ is not one interchangeable mathematical object across these repositories. Every adapter must carry at least **kind, hash axis, canonicalization/realization version, framed input schema and digest**. Persist identity provenance separately from empirical model behavior.

1. **Opaque byte identity:** `kappa-registry` hashes the supplied bytes, and its `kappa_from_value` first serializes dCBOR. `KappaLabel` supports six axes, up to 135 ASCII bytes. [UOR-Foundation/kappa-registry/crates/kappa-core/src/kappa/compute.rs:127](https://github.com/UOR-Foundation/kappa-registry/blob/2af86560a177fc9651b6c0e92e7974140ed77dd5/crates/kappa-core/src/kappa/compute.rs#L127), [UOR-Foundation/kappa-registry/crates/kappa-core/src/kappa/compute.rs:143](https://github.com/UOR-Foundation/kappa-registry/blob/2af86560a177fc9651b6c0e92e7974140ed77dd5/crates/kappa-core/src/kappa/compute.rs#L143).
2. **Canonical structural identity:** `uor-addr` canonicalizes according to a declared format/realization. JSON+NFC, model skeleton canonicalization, and ring bytes are different contracts. ONNX defines its own canonical-form version 3 and documents version-dependent labels. Equal labels are evidence for that normal form, not arbitrary behavioral equivalence of two learned models. [UOR-Foundation/uor-addr/crates/uor-addr/src/onnx/mod.rs:25](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/onnx/mod.rs#L25), [UOR-Foundation/uor-addr/crates/uor-addr/src/json/mod.rs:62](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/json/mod.rs#L62).
3. **Codec artifact identity:** uor-matmul addresses a schema-tagged manifest including codebook/code digests. Equal-decoding transcodes can intentionally have different artifact identities. [UOR-Foundation/uor-matmul/crates/uor-matmul-codec/src/kappa.rs:1](https://github.com/UOR-Foundation/uor-matmul/blob/3cc5882f210667f9ac00fd8c02c5b5957b493f5d/crates/uor-matmul-codec/src/kappa.rs#L1).
4. **Derivation/memo key:** hologram addresses opcode, scalar parameters and ordered operand labels without hashing result bytes. This is sound only under the declared deterministic operation semantics and complete framing. It is not automatically a result-content hash. [Hologram-Technologies/hologram/crates/hologram-archive/src/address.rs:108](https://github.com/Hologram-Technologies/hologram/blob/94ecb886811115491a77c8229e494965bea03fc2/crates/hologram-archive/src/address.rs#L108).
5. **Composite identity:** CS-G2 canonicalizes a binary pair. Pairwise commutativity does not imply associativity or permutation invariance of a left fold. Hologram archive's `compose_model` uses the unsorted left fold; hologram-ai explicitly sorts labels because the fold over three or more operands is order-sensitive. For a model graph, role/position/topology must remain represented; sorting raw components alone cannot certify that their wiring is the same. [UOR-Foundation/uor-addr/crates/uor-addr/src/composition/canonicalize.rs:95](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/composition/canonicalize.rs#L95), [Hologram-Technologies/hologram/crates/hologram-archive/src/address.rs:150](https://github.com/Hologram-Technologies/hologram/blob/94ecb886811115491a77c8229e494965bea03fc2/crates/hologram-archive/src/address.rs#L150), [Hologram-Technologies/hologram-ai/crates/hologram-ai/src/address.rs:117](https://github.com/Hologram-Technologies/hologram-ai/blob/d5337ec2b3289fc8462abac2e627d165342079b2/crates/hologram-ai/src/address.rs#L117).

Do not replace an existing `blake3:` digest with a different canonicalization merely because both outputs are called κ. Define an explicit migration/version boundary and verify old/new typed meanings.

## Exact arithmetic and modulo 256 / CRT

The framework describes rings `Z/(2^n)Z`; its ring-address realization serializes the Witt level plus a coefficient and currently admits four levels. The exact domain and byte width are part of identity. [UOR-Foundation/UOR-Framework/foundation/src/kernel/address.rs:10](https://github.com/UOR-Foundation/UOR-Framework/blob/51c01382200b0179d6640b07e9c8119364ab69a1/foundation/src/kernel/address.rs#L10), [UOR-Foundation/uor-addr/crates/uor-addr/src/ring/mod.rs:23](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/ring/mod.rs#L23).

**Important implementation distinction:** Prism's `Gf2NumericAxisN` implements XOR addition and AND multiplication on independent bits. This is a Boolean product ring, not wrapping addition/multiplication modulo 256, and not polynomial GF(256) arithmetic. Example: `1 XOR 1 = 0`, whereas `1+1 mod 256 = 2`. Do not reuse that axis as a mod 256 arithmetic backend. [UOR-Foundation/prism/crates/uor-prism-numerics/src/ring.rs:1](https://github.com/UOR-Foundation/prism/blob/507995bab43c0cb06ec244c96e6fc25b3f502204/crates/uor-prism-numerics/src/ring.rs#L1).

The following are mathematical preconditions for a proposed integration, not newly established project measurements:

- Modulo256 is `Z/256Z`, with zero divisors: `2*128=0 mod 256`. Division by an even element is not valid as a ring inverse.
- Ordinary CRT reconstruction requires pairwise coprime moduli and is unique modulo their product `M`. Repeated powers of two do not give independent coprime channels; generalized noncoprime CRT needs compatibility conditions and reconstructs modulo the lcm.
- Recovering an integer sum exactly requires an a priori range. For a dot product with `K` terms and absolute operand bounds `A,B`, `|sum| <= KAB`; signed reconstruction needs `M > 2KAB` (plus any epilogue growth). Without that bound, only a residue class is recovered.
- Real-valued softmax, normalization, division and arbitrary learned tensor operations do not become exact modulo 256 simply by encoding their inputs as bytes. Each lowering needs a commuting encode/operate/decode statement, a complete finite domain or explicit approximation error, and a reconstruction/overflow condition.
- A lookup is constant-time with respect to its fixed indexed domain after construction; table generation, operand hashing, memory, misses and growing-domain costs must be charged separately. Hologram documents 256-entry byte tables and content-addressed memo reuse; that does not establish constant-time arbitrary inference. [Hologram-Technologies/hologram/README.md:102](https://github.com/Hologram-Technologies/hologram/blob/94ecb886811115491a77c8229e494965bea03fc2/README.md#L102), [Hologram-Technologies/hologram/README.md:113](https://github.com/Hologram-Technologies/hologram/blob/94ecb886811115491a77c8229e494965bea03fc2/README.md#L113).
- uor-matmul's float semantics are exact dyadic accumulation with one final rounding, intentionally different from sequential FMA/BLAS rounding. Keep that arithmetic contract separate from preservation of a frozen neural checkpoint's existing output behavior. [UOR-Foundation/uor-matmul/README.md:155](https://github.com/UOR-Foundation/uor-matmul/blob/3cc5882f210667f9ac00fd8c02c5b5957b493f5d/README.md#L155), [UOR-Foundation/uor-matmul/README.md:217](https://github.com/UOR-Foundation/uor-matmul/blob/3cc5882f210667f9ac00fd8c02c5b5957b493f5d/README.md#L217).

No reusable general CRT API was established by the inspected source excerpts. A named exact operation must be selected before introducing a CRT dependency. Whole-repository absence is not claimed.

## Import decisions

| Candidate | Proposed use | Boundary |
|---|---|---|
| Existing Framework + uor-addr + Prism | Extend the current typed UOR facade and canonical manifest adapters | Preserve one dependency source and versioned identity rules. |
| uor-matmul | Audit upstream delta, then separately decide whether a frozen operation warrants repinning | Exact arithmetic semantics and causal/output parity are distinct checks. |
| hologram archive/store/compute crates | One selected content/store or finite-operation adapter | Avoid importing a second whole runtime or replacing the geometric attention model. |
| hologram-ai | Download progress, model intake, `LmSession`/`SessionProvider` API design | Its manifest uses a forked hologram revision and moving holospaces branch; dependency closure first. |
| kappa-registry | Later external artifact/provenance service, disk pressure and verified ingestion | dCBOR/raw-byte identity differs from JCS/model canonical identity; retain adapter. |
| auser/uor-semantic | Legacy R4G1 interchange and bounded parser/scorer reference | No attention; source itself excludes target-runtime/teacher parity. |
| auser/archon | Optional multi-repo API extraction reference | Current manifest suffices; no automatic CLAUDE.md rewrites. |
| LexLean, lean4-prod | Later one named formal spec/proof-to-Rust prototype | Explicit toolchain and correspondence trust; no automatic proof transfer. |
| F1, UOR-Atlas-UTQC, WASM-GEMM-GNAF | Scoped external mathematical references | Respect source-specific open obligations and theorem assumptions. |
| emporous | Later supply-chain/provenance composition | Vendored registry plus corrections differs from upstream. |
| hologram-live, holospaces, workflow-graph | Later service/lifecycle or UI components | Preserve the product's current browser-over-Rust direction. |

Exact pinned source links for each candidate are in `integration-candidates.json`. Useful concrete seams include [`LmSession`](https://github.com/Hologram-Technologies/hologram-ai/blob/d5337ec2b3289fc8462abac2e627d165342079b2/crates/hologram-ai/src/engine.rs#L90), [`VerifiedContent`](https://github.com/UOR-Foundation/kappa-registry/blob/2af86560a177fc9651b6c0e92e7974140ed77dd5/crates/kappa-core/src/verified.rs#L101), and [`export_r4g1`/`replay_r4g1`](https://github.com/auser/uor-semantic/blob/1ba84e726f34893485e59ce89bd56197c9188c4a/crates/uor-semantic-compiler/src/lib.rs#L25).

The popular auser repositories `poolparty`, `alice`, `beehive` and `wonderland` are old archived infrastructure/frontend projects. Their popularity does not make them appropriate numeric/runtime dependencies. `afflom/matmul` currently contains the empty repository template, despite its mathematical description. `kappa-distribution` is a README-only repository; its implementation is in kappa-registry's module. These are discovery/history entries, not import candidates.

## Formal and licensing limits

`WASM-GEMM-GNAF` explicitly reports `WorkloadIncomplete`: global optimality is unestablished, and its abstract cost objective is not physical runtime. `F1` keeps RH/Hodge-index positivity open. `UOR-Atlas-UTQC` distinguishes block-specific density evidence and classical-lemma assumptions from quantum advantage. Those limitations should remain attached when reusing any lemmas or constants. See the pinned READMEs in the candidate manifest.

GitHub's detected license is sometimes missing or disagrees with package metadata (for example uor-matmul's GitHub MIT detection versus its Cargo Apache-2.0 declaration). The inventory preserves both fields where inspected. Before copying or linking selected code, review its actual license files and package declarations; do not infer permission from a public repository alone.

## Disk-aware flow

- Keep this catalog and pinned source excerpts as the first stage. GitHub `size` is stored as `github_size_kib`; it is not a build/model/LFS forecast.
- For a selected repo, use one external partial clone or bare object store keyed by owner/repo with a pinned commit, sparse-checkout only required crates/specs/licenses. Avoid cloning all histories, running dependency setup, or downloading LFS/model artifacts during discovery.
- Use isolated worktrees from the shared object store when an actual integration begins. Maintain a manifest containing repo, commit, included paths, license, dependency pins and artifact CIDs. Never update a frozen run to follow upstream main.
- Inventory cache/build/source/evidence/model bytes separately. Set a storage budget before build/download; retain evidence and unique inputs. No cache or worktree cleanup was performed.
- Large candidates require special care: `hologram-apps` reports about 2,007,021 KiB GitHub repository size; its default tree has 10,992 files. A whole-history checkout is unnecessary for API research.
- Prefer one existing CAS and explicit typed adapters. A new registry may deduplicate artifacts but also adds metadata/database/staging/keys that must be budgeted and backed up.

## Next concrete integration step

Create a small reviewed **UOR integration contract** around existing dependencies: enumerate the product's actual manifest/state/artifact types, map each to its canonicalization and hash axis, record ordered versus unordered composition, and expose verification through the existing Rust API. Select one useful external seam only after that table is coherent. Preserve the current construction-only exposure/cancellation diagnostic as the immediate scientific next step; this catalog does not supply the missing causal model result.
