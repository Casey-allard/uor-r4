# Afflom ecosystem follow-up — bounded source audit

Date: 2026-09-03. R4 baseline: `3e894820c520f3b7803a48c6a2eeeb5b7d7021c5`.
Status: `SOURCE_REVIEW_ONLY`. Public knowledge-index queries for LexLean,
uor-matmul, Prism, Atlas, lean4-prod and GNAF led back to the pinned original
files below. GitHub default heads and complete tree responses were refreshed.
This appends to [the earlier source audit](uor-source-audit.md); it does not
rewrite that snapshot. The [source manifest](afflom-ecosystem-sources.json)
records revisions, file digests, visibility and inspection coverage.

The project destination remains a local transformerless geometric reasoning
and coding model on current commercial hardware. These components offer
specific interfaces, arithmetic implementations and formal tooling toward that
destination. Source availability does not establish frontier capability,
geometric advantage, model correctness or a qualified final serving kernel.

## Decision for the active adapter

Keep #1094's separately frozen comparison unchanged. `R4TextToClausesV1` removes
caller-supplied boundaries while preserving #1077's reader, #1073's core,
#1079's coherent R4 execution, vocabulary, query form and four-fact context.
Its adapter produces token IDs and lengths; it cannot reconstruct semantic
roles or supply an answer. Substituting a proof-language parser, a matrix
backend or an Atlas classifier would change this contract. No new dependency,
fit, arithmetic substitution or upstream execution is admitted by this audit.
See [the accepted contract](clause-segmentation-1085.md).

Use these sources immediately as interface and provenance references: preserve
ordered inputs, name the serialization and hash separately, bind the source
closure, expose typed refusals, and keep model outputs independent from the
reference oracle. The concrete integrations belong to existing #1083, #1087
and #1089, under their own decisions. This creates no second backlog.

## Current identities and licenses

All eight default-head SHAs below match the prior public catalog. The audit
fetched 50 selected text files, 405,685 bytes in total, and inspected the
relevant source excerpts. No tree response was truncated. This is bounded
inspection of these interfaces, not a review of every declaration or repository.

| Repository | Refreshed default head | Actual licensing material inspected |
|---|---|---|
| `UOR-Foundation/prism` | `507995bab43c0cb06ec244c96e6fc25b3f502204` | Root MIT license; The UOR Foundation copyright. |
| `UOR-Foundation/atlas-12288` | `aef42a6fd5c323373222b6362050b439690136a1` | Root MIT license; Alex Flom copyright. |
| `afflom/UOR-Atlas-UTQC` | `b9717312e76c14c35ee7be88ac7ca71625448dff` | Root MIT license; Alex Flom copyright. |
| `afflom/LexLean` | `79371e16027d0864e014e3ce1f8f95745ce5caaa` | MIT and Apache-2.0 files; workspace declares the dual license. |
| `afflom/lean4-prod` | `f5367291b0146433ee17b58fc6ca49f593c22c52` | `prod-codegen/Cargo.toml` declares MIT; complete tree has no license-named file. Preserve this metadata/file distinction before copying. |
| `afflom/WASM-GEMM-GNAF` | `917306fd2b5a397ab02c5d38918fb8620fcc5ae0` | MIT and Apache-2.0 files; README offers either. |
| `afflom/matmul` | `40d85fd97634213d312a3bcf012758f531002294` | MIT and Apache-2.0 files; README offers either. |
| `UOR-Foundation/uor-matmul` | `3cc5882f210667f9ac00fd8c02c5b5957b493f5d` | Root MIT license credits Alex Flom; workspace declares Apache-2.0. Record the discrepancy before selecting new imported code. |

GitHub identifies `afflom/lean4-prod` as a fork of `auser/lean4-prod`. Its
audited revision and added interfaces must remain attached to any reuse; the
two owners' heads are not interchangeable. The two matmul repositories are
also distinct. Repository ownership and copyright notices provide bounded
provenance; this record does not assign sole authorship of every component.
License files and package metadata are linked and hashed in the source manifest.

## Concrete source findings and adoption boundaries

### Matmul: retain the existing library, distinguish the template

`afflom/matmul` currently contains the repository template, with no shipped
computational crate. Its README says so, its claim register is `id = []`, and
its ledger is `claim = []`. The complete 34-file tree contains the model,
conformance and xtask machinery. Its GitHub description names a matrix study,
but that description is not an implemented GEMM API. **Decision:** exclude this
head as a numerical backend; the claim-register pattern is already familiar
to R4. [README](https://github.com/afflom/matmul/blob/40d85fd97634213d312a3bcf012758f531002294/README.md#L3),
[register](https://github.com/afflom/matmul/blob/40d85fd97634213d312a3bcf012758f531002294/model/ids.toml#L18),
[ledger](https://github.com/afflom/matmul/blob/40d85fd97634213d312a3bcf012758f531002294/model/ledger.toml#L13).

The concrete `UOR-Foundation/uor-matmul` library is already in R4's lockfile at
`b13c98449948174f590e337c4dc25dfc394a07d0`. Its inspected facade exposes
`MatView`, `MatViewMut`, `Triple`, codecs, `gemm_auto`, `gemm_float`,
`gemm_tabulated`, `workspace_report` and `workspace_for_budget`. Shape/view
construction checks the product; the caller provides scratch. These are useful
operator and resource boundaries for #1087.
[Facade](https://github.com/UOR-Foundation/uor-matmul/blob/3cc5882f210667f9ac00fd8c02c5b5957b493f5d/crates/uor-matmul/src/lib.rs#L123).

The declared float operation accumulates decoded dyadic values exactly and
rounds once. It intentionally need not reproduce sequential floating-point
BLAS/FMA results. That semantic distinction applies even when all routes in
uor-matmul agree with each other. Its weight-artifact `Manifest` frames tier,
bound, shape, block, codebook digest, code digest and schema; equal decoded
weights can have different codec artifact IDs. **Decision:** retain the current
pin for #1094. Under #1083, distinguish this codec ID from R4 result-content and
ordered derivation IDs. Under #1087, select one actual operation, then declare
its rounding/output contract, cold/warm cost and scratch budget before a
compatibility experiment.
[Float contract](https://github.com/UOR-Foundation/uor-matmul/blob/3cc5882f210667f9ac00fd8c02c5b5957b493f5d/README.md#L155),
[identity schema](https://github.com/UOR-Foundation/uor-matmul/blob/3cc5882f210667f9ac00fd8c02c5b5957b493f5d/crates/uor-matmul-codec/src/kappa.rs#L28).

The refreshed comparison is seven commits ahead and zero behind the R4 pin,
with ten changed paths. Those paths are CI, lockfile, recipes, policy, tests and
two Cargo manifests; no numerical implementation file is changed in that
comparison. This narrows a future repin review but does not establish compiled
compatibility, performance or checkpoint output parity.
[Pinned comparison](https://github.com/UOR-Foundation/uor-matmul/compare/b13c98449948174f590e337c4dc25dfc394a07d0...3cc5882f210667f9ac00fd8c02c5b5957b493f5d).

### Prism: typed operation surfaces, with the algebra named

Prism re-exports the foundation and SDK plus crypto, numerics and tensor axes.
The tensor facade exposes `TensorAxis`, `ActivationAxis`,
`CpuI8MatmulSquare<DIM>`, and shape/dtype carriers; these are useful interface
references for a selected finite operator. They are not a general replacement
for the learned reference. The inspected `RingAxis` implementation,
`Gf2NumericAxisN<BYTES>`, adds by XOR and multiplies by AND in caller-provided
buffers. Its code therefore denotes the product of independent bit algebras,
not `Z/256Z` or polynomial `GF(256)`. For example, `1 XOR 1 = 0` while
`1 + 1 mod 256 = 2` is a direct counterexample to interchanging the two additions.
**Decision:** use the axis/shape interface in #1083's typed contract only where
the operation domain matches. Do not introduce a second address implementation
or substitute this ring under the learned reader.
[Facade](https://github.com/UOR-Foundation/prism/blob/507995bab43c0cb06ec244c96e6fc25b3f502204/README.md#L10),
[tensor surface](https://github.com/UOR-Foundation/prism/blob/507995bab43c0cb06ec244c96e6fc25b3f502204/crates/uor-prism-tensor/src/lib.rs#L11),
[ring implementation](https://github.com/UOR-Foundation/prism/blob/507995bab43c0cb06ec244c96e6fc25b3f502204/crates/uor-prism-numerics/src/ring.rs#L76).

### Atlas: finite coordinates need their own input and correspondence contract

`atlas-12288` exposes page/byte coordinates, an R96 classifier and budget-zero
helpers. The inspected Lean classifier is explicitly `b % 96`. The Rust
wrappers call the `_minimal` C symbols, whose implementation repeats those
operations without the Lean runtime. Thus the Rust wrapper is not itself
evidence of execution of the exported Lean code.
[Lean classifier](https://github.com/UOR-Foundation/atlas-12288/blob/aef42a6fd5c323373222b6362050b439690136a1/lean/UOR/Prime/Structure.lean#L42),
[Rust FFI](https://github.com/UOR-Foundation/atlas-12288/blob/aef42a6fd5c323373222b6362050b439690136a1/runtime/rust/src/lib.rs#L16),
[minimal C implementation](https://github.com/UOR-Foundation/atlas-12288/blob/aef42a6fd5c323373222b6362050b439690136a1/ffi/c/minimal_wrapper.c#L17).

Two source-level deductions matter for any reuse. First, bytes 0 and 96 have
the same class, so the classifier alone cannot losslessly recover arbitrary
input bytes. Second, Lean's `phiEncode` packs the raw page, whereas the minimal
C encoder first reduces it modulo 48. At page 48, byte 0, their encoded values
are respectively 12,288 and 0; restricting the input page to 0–47 removes this
particular difference. Also, the `UInt32`/C budget addition wraps: the source's
Rust test expects `truth_add(u32::MAX, 1)` to return true. This does not establish
the comment's unrestricted conjunction law or semantic factual correctness.
These deductions follow from the displayed definitions; no Lean proof or FFI
execution was performed here.
[Lean packing/budget definitions](https://github.com/UOR-Foundation/atlas-12288/blob/aef42a6fd5c323373222b6362050b439690136a1/lean/UOR/FFI/CAPI.lean#L30),
[C packing](https://github.com/UOR-Foundation/atlas-12288/blob/aef42a6fd5c323373222b6362050b439690136a1/ffi/c/minimal_wrapper.c#L35),
[wrapping test](https://github.com/UOR-Foundation/atlas-12288/blob/aef42a6fd5c323373222b6362050b439690136a1/runtime/rust/src/lib.rs#L311).

`UOR-Atlas-UTQC` is a different source: its selected API resolves modeled
`UseCaseParams` and provides E8 Cartan/Gram constructions. Its own ledger
distinguishes externally sourced facts, scoped constructions and open
measurements. Its README limits the quantum framing to structural/simulation
work and does not claim measured quantum advantage. **Decision:** keep both
Atlas sources as scoped references for #1083/#1089. Select a specific
constructor or finite witness and prove its mapping to the actual R4
representation before reuse; neither is a text segmenter or an attention
qualification.
[Atlas API](https://github.com/afflom/UOR-Atlas-UTQC/blob/b9717312e76c14c35ee7be88ac7ca71625448dff/crates/tqc-atlas/src/lib.rs#L33),
[status types](https://github.com/afflom/UOR-Atlas-UTQC/blob/b9717312e76c14c35ee7be88ac7ca71625448dff/model/status.toml#L8),
[scope](https://github.com/afflom/UOR-Atlas-UTQC/blob/b9717312e76c14c35ee7be88ac7ca71625448dff/README.md#L3).

### LexLean and lean4-prod: connect a named proof, not arbitrary prose

LexLean's `Engine` provides `load`, `lock`, `check`, `snapshot`, `build`,
`verify` and `format`. Its controlled lexicon and typed semantic representation
produce Lean and canonical document artifacts. Its build manifest separately
records source, semantic and build IDs, compiler semantics, selected modules,
inputs and outputs. `verify` runs the pinned Lean 4.32.1 tools and audits the
observed axioms; `build`/`check` alone do not establish verification. These are
good #1089 proof-authoring and #1083 identity reference interfaces, but replacing
#1094's segmenter with a semantic parser would violate its zero-role-input seam.
[Engine API](https://github.com/afflom/LexLean/blob/79371e16027d0864e014e3ce1f8f95745ce5caaa/crates/lexlean/src/api.rs#L629),
[manifest](https://github.com/afflom/LexLean/blob/79371e16027d0864e014e3ce1f8f95745ce5caaa/crates/lexlean/src/artifact/manifest.rs#L44),
[verification boundary](https://github.com/afflom/LexLean/blob/79371e16027d0864e014e3ce1f8f95745ce5caaa/README.md#L151).

The audited lean4-prod fork offers `Prod.exportModule` and `Prod.exportNames`
over Lean LCNF and Rust `generate_module`/`generate_def`. Its source reports
unsupported constructions and maps Lean `Nat`/`Int` to bounded `u64`/`i64` with
fallible checked arithmetic. An unbounded theorem therefore requires explicit
range/refinement obligations when used through that ABI. Its existing UOR
fixture covers Witt-level width and primitive-operation commutativity metadata,
not the complete UOR model or R4's numerical runtime. **Decision:** under #1089,
choose one named finite operator, record its Lean theorem/axioms and toolchain,
then review extraction, lowering, bounded integer semantics and Rust execution
correspondence. Generated Rust does not automatically inherit a proof about
Lean's unbounded source function. No such prototype was executed in this audit.
[Named export](https://github.com/afflom/lean4-prod/blob/f5367291b0146433ee17b58fc6ca49f593c22c52/lean/Prod/Export.lean#L213),
[code-generator contract](https://github.com/afflom/lean4-prod/blob/f5367291b0146433ee17b58fc6ca49f593c22c52/rust/prod-codegen/src/lib.rs#L34),
[UOR fixture scope](https://github.com/afflom/lean4-prod/blob/f5367291b0146433ee17b58fc6ca49f593c22c52/README.md#L248).

### GNAF: preserve the explicit incomplete release chain

WASM-GEMM-GNAF remains `WorkloadIncomplete`. Its status source distinguishes
public amended-Core encode/decode agreement from an emitted, semantically
correct GEMM with release cost/resource evidence. The public compiler,
execution refinement, arithmetic-mode chain, universal coverage and released
global-optimality result remain open. A separate source lemma proves that the
recorded cover check depends only on its recorded partition/candidate/root
components; that is not universal coverage of all admissible programs.
**Decision:** use its exact obligation decomposition and selected lemmas as
#1087/#1089 references. Do not select it as a shipped globally optimal GEMM,
interpret charged abstract cost as hardware latency, or infer language ability.
Formal declarations were inspected but not rechecked here.
[Release status](https://github.com/afflom/WASM-GEMM-GNAF/blob/917306fd2b5a397ab02c5d38918fb8620fcc5ae0/WasmGemmGnaf/Theorems/Status.lean#L30),
[coverage-scope theorem](https://github.com/afflom/WASM-GEMM-GNAF/blob/917306fd2b5a397ab02c5d38918fb8620fcc5ae0/WasmGemmGnaf/Atlas/CoverageScope.lean#L20),
[cost scope](https://github.com/afflom/WASM-GEMM-GNAF/blob/917306fd2b5a397ab02c5d38918fb8620fcc5ae0/README.md#L71).

## Evidence status and next action

This review establishes which inspected source interfaces exist and what their
declared or directly visible semantics permit. Mathematical counterexamples
above are deductions from the displayed finite definitions. Upstream theorem,
benchmark, example and release claims remain attributed to their sources:
local build, Lean checking, FFI execution, model evaluation and performance
verification are all `NOT_RUN` for this source audit.

Preserve #1079 `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, #1082's descriptive-only
result, the earlier failed capacity/grounding criteria and the distinction
between the dense research reference and the intended final kernel. No imported
source reverses those measurements. No source code was vendored, no model
assets downloaded and no evidence or user material deleted.

The immediate action is to complete #1094 under its unchanged contract. The
concrete subsequent ecosystem action is #1083's typed identity/arithmetic ADR:
use the existing R4 result/input manifest boundary to distinguish raw content,
structural identity, codec identity and ordered derivation keys; name its
verification API and migration rule. Carry the Prism, matmul, Atlas and
LexLean counterexamples into that review before selecting any new adapter.

## Independent selected-source review — 2026-09-03

An independent reviewer fetched 14 pinned files from the six Prism,
atlas-12288, matmul, uor-matmul, lean4-prod and WASM-GEMM-GNAF repositories:
106,738 bytes, with every SHA256 matching the source manifest. The reviewer
read the relevant definitions and call sites. The
[independent review receipt](afflom-ecosystem-review.json) binds each public
source revision, SHA256 and BLAKE3 content ID. This narrower review did not
repeat the complete-tree, license, LexLean or UOR-Atlas-UTQC inspections and
does not certify every upstream declaration.

The inspected Atlas definitions support the page-48 counterexample and the
R96 collision directly; the Rust facade calls the minimal C symbols. Prism's
XOR/AND loops operate on independent bits, so the Boolean-product domain
qualification is necessary. The empty afflom/matmul register and ledger agree
with its template README, whereas uor-matmul's facade exposes the stated GEMM
interfaces. GNAF's status pointer explicitly retains `WorkloadIncomplete`,
and its coverage lemma concerns equality of a recorded check under three
equal recorded components. These source observations support the adoption
boundaries above. They are neither fresh numerical measurements nor a
re-execution of the upstream formal proofs.

**Additional extraction qualification:** lean4-prod's type spelling maps
`Nat` to `u64` and `Int` to `i64`, while its inspected checked-arithmetic
emitters explicitly cast the receiver to `u64`. The type spelling alone
therefore does not establish support or correspondence for every signed
primitive. Its left-shift emitter uses `checked_shl`: Rust checks the shift
count, but a successful result may still discard high value bits. A future
Nat-shift refinement must constrain the resulting value as well as the
shift count; the word "checked" is not that proof.
[Type mapping and emitters](https://github.com/afflom/lean4-prod/blob/f5367291b0146433ee17b58fc6ca49f593c22c52/rust/prod-codegen/src/lib.rs#L594),
[shift emission](https://github.com/afflom/lean4-prod/blob/f5367291b0146433ee17b58fc6ca49f593c22c52/rust/prod-codegen/src/lib.rs#L1284),
[Rust checked-shift semantics](https://doc.rust-lang.org/std/primitive.u64.html#method.checked_shl).

The review required no builds, tests, proof execution or model access. All of
those remain `NOT_RUN`; #1094's frozen policy and sealed inputs were untouched.
