# Native learned-reference artifact and behavior contract — #1086

**Status: `NATIVE_REFERENCE_CONTRACT_SPECIFIED`.**
Independent acceptance and its reviewed document identities are recorded in the
linked review; closure still requires verified protected delivery.
This specification at protected baseline `eade29f4b78435e9857936786426bb34e596b301`
answers [#1086](https://github.com/UOR-Foundation/uor-r4/issues/1086).
It freezes an export/loader contract and a separate empirical comparison.
Export, native implementation, model loading, comparison and replay are `NOT_RUN`.
This document is not an execution release or a native capability result.

The [machine-readable contract](r4_native_reference_1086_contract.json),
[operator audit](r4_native_reference_1086_operator_audit.md),
[original-source review](r4_native_reference_1086_sources.md) and
[independent review](r4_native_reference_1086_review.md) accompany this record.
The contract enumerates exact fields and tensor order; this prose defines their
meaning. A conflict between them stops admission until corrected and reviewed.

## 1. Decision, ownership and evidence boundary

**Definition — selected boundary.** `R4LearnedReferenceV1` is an opt-in CPU Rust
research reference for the accepted #1077 reader, #1073 binding core and #1079
two-stage coherent R4 execution, preceded by #1094's `R4TextToClausesV1`.
It supplies one stateless operation: `answer_four_fact_raw_text/v1`.
It must load a self-contained, verified artifact and compute the actual full-head
model token without a Python subprocess, provider, answer cache or fallback.

The future numerical implementation belongs to a new, narrowly scoped
`uor-r4-core::learned_reference` module behind an opt-in `learned-reference`
feature. Byte-oriented public types belong to
`uor-r4-api::learned_reference`, behind that same opt-in feature. These are
**proposed module locations, not existing symbols**. They do not change
`R4Engine`, `GraphView`, R4G1 sections, graph compilation, default serving,
HTTP/chat endpoints or the integer/table runtime. The native reference uses
floating point and allocation; no source-free/final-kernel guarantee is imported.
#1084 owns later service/UI wiring, #1087 final lowering, and #1083 typed UOR
integration. No new runtime architecture or crate family is proposed here.

#1094's [accepted comparison](r4_retained_comparison_1094.md) preserved all
1,600 valid inputs/tensors/answers and 96 refusals, with exact fresh-process
replay. These are renderings of twenty already-observed base groups. Its result
qualifies bounded raw-text entry to the Python reference. It does not establish
Rust parity. Its consumed envelope, resealed withheld population and original
receipts are not reused as a new run authorization.

## 2. Accepted inputs to the future exporter

**Definition — trust anchor.** A separately reviewed export release must bind
the accepted source-file and decoded-state identities in the machine contract,
the exact source closure, exporter/evaluator/native revisions, compiler/lockfile,
runtime/hardware, input commitments and resource envelope. No placeholder digest
may appear in an executable release. Paths are local locators, never identity.

The five previously accepted source assets are the reader/core Safetensors,
4096-entry vocabulary JSON, and original `h4-frames.json`/`token-frames.json`.
Their exact bytes, SHA256 and BLAKE3 file CIDs are retained in
[#1094 bindings](r4_retained_assembly_1094_evidence/bindings.json).
This specification reads those public bindings and actual source; it does not
reopen or deserialize the assets. The future offline exporter must verify actual
file identities before decoding, decode Safetensors bytes without constructing
randomly initialized model modules, and verify decoded state before emitting any
final artifact. The existing `load_model` constructor path is not the exporter.
No constructor RNG, refitting, tensor normalization, dtype conversion, padding change or vocabulary
regeneration is permitted. Export twice into separate exclusive destinations
from the same pinned inputs and require identical complete bytes before scoring.

Reader state is five f32 tensors (141,571 scalars), core state nine unique f32
tensors (286,976 scalars): **1,714,188 raw parameter bytes** in total. The core
`lm_head.weight` is an identity alias of `core.embedding.weight`; it is not a
second serialized tensor. All tensor names, C-order shapes and offsets are fixed
in the machine contract. No unexpected tensors, strides, quantization, sparse
layout or initializer/default fills are admitted.

## 3. Artifact wire format and identities

**Definition — container.** One regular artifact has exactly:

```text
8 bytes ASCII R4LR0001
u32 little-endian manifest byte count
manifest bytes
u64 little-endian payload byte count
payload bytes
EOF (no trailer)
```

The manifest is schema `uor-r4.native-reference-manifest/1`. Its exact fields are
listed in the contract. It is UTF-8 JSON with unique keys, ASCII strings, integer
numbers only, no BOM, no insignificant whitespace and keys sorted lexicographically
at every object level. Arrays keep their declared order. Escape quote/backslash
as `\"`/`\\`, controls as lowercase `\u00xx`; do not escape `/` or other ASCII.
No NaN, Infinity, fractional number, duplicate, unknown or missing field is accepted.
This deliberately small canonicalization is named `ascii-json-1086/1` and is not
UOR structural canonicalization. Runtime version text is provenance, not authority
to select different operators. Numeric semantic constants use exact IEEE bit strings.

Payload components occur in the fixed order in the contract, tightly packed
without gaps or alignment bytes. Each component has a name, kind, dtype, ordered
shape, payload-relative u64 offset, u64 byte count and SHA256 of its exact bytes.
Tensor scalars use IEEE f32/f64 or signed i64, little endian, C order. The loader
must use checked byte decoding; an unaligned offset is not permission for unsafe
casts. Exact component lengths and cumulative offsets must reconcile to payload
length; a byte cannot belong to two components. No external file/URL reference
may provide inference state after load.

Components comprise fourteen parameter tensors; original vocabulary and policy
bytes; original H4 and token-frame JSON bytes; and the exact decoded f64
`[120,4,4]` matrices, i64 `[120,120]` multiplication table and i64 `[8192]`
token-leaf map. The identity index is copied from both agreeing original frame
files. Original JSON remains provenance and a cross-check of decoded tables.
The 120 matrices (1,920 f64 scalars) are copied from their original bit representation,
never reconstructed from rounded f32 actions or newly evaluated irrational values.
The three original token-prefix witnesses are checked against the decoded fold;
this finite witness check is not a proof of all geometric laws.

**Definition — separate hash axes.**

- `artifact_sha256`: SHA256 of the entire container, supplied by the caller as
  an expected identity before loading. It is outside the manifest to avoid a
  self-hash cycle. A manifest's own component hashes are not a trust root.
- `reader_state_cid`/`core_state_cid`: the historical BLAKE3 state recipe, retained
  exactly: sort original unprefixed tensor names, append for each tensor the
  accepted sorted compact JSON `{name,shape,dtype:"torch.float32"}` followed by
  LF, then its contiguous little-endian bytes. There is no domain/count prefix.
  Exclude the tied `lm_head.weight`. Reproduce the fixed accepted state CIDs.
- `native_state_sha256`: SHA256 of `L("uor-r4.native-reference-state/1")`,
  u32 component count, then each fixed-order component's `L(name)`, `L(kind)`,
  `L(dtype)`, u32 rank, u64 dimensions, u64 byte count and exact payload bytes;
  finish with u32 identity index and `L(operator_profile)`. `L` is u32-LE length
  plus ASCII bytes. This binds tensors, codec/policy and frame tables together.
- Original reader/core file CIDs retain source-file provenance; they cannot be
  recomputed from header-free tensor payload alone. The exporter verifies files;
  the native loader verifies their recorded identities against its trusted
  accepted binding and independently reconstructs the decoded state CIDs.
- Policy SHA256 hashes its exact original bytes. The fixed vocabulary and frame
  file/tree CIDs retain their original recipes. No commutative composition,
  derivation label or generic “κ” is substituted for these identities.

No native-state or artifact digest is claimed to exist yet. A future export
receipt binds original file/state identities to actual container and native-state
digests. It is a provenance record; later behavior evidence remains separate.

## 4. Loader validation and capability admission

**Definition — native load.** The future API accepts owned artifact bytes and a
trusted `ExpectedBinding` (expected artifact SHA256, specification/contract
digest, accepted source/state/codec/policy/frame identities and operator profile).
It returns `Result<LoadedResearchReference, NativeLoadError>`. It performs no
network, Python, filesystem discovery or model forward. The host owns paths and
byte acquisition. Keep immutable verified bytes alive; do not verify a path and
later reopen it. A failed load exposes no usable partial engine.

Validation order is binding: container limit/header/version/lengths; expected
whole-file digest; strict manifest schema/canonical bytes; required profile and
trusted source identities; exact component layout/digests; tensor types/shapes/
finite parameters and tied-head declaration; vocabulary/policy mappings;
frame original/decoded agreement/ranges/witnesses; decoded state identities.
Within a stage use component order then increasing byte/element position.
The source-binding stage requires exact equality of manifest `contract_sha256`
to ExpectedBinding `contract_sha256`, manifest `export_provenance.release_sha256`
to ExpectedBinding `export_release_sha256`, and manifest `source_binding` to
ExpectedBinding `accepted_binding`; any mismatch is `SOURCE_BINDING_MISMATCH`.
The operator profile must equal the expected and supported profile or return
`UNSUPPORTED_PROFILE`. These checks precede component validation.
Errors are respectively `CONTAINER_LIMIT`, `INVALID_CONTAINER`,
`ARTIFACT_IDENTITY_MISMATCH`, `UNSUPPORTED_MANIFEST`, `UNSUPPORTED_PROFILE`,
`SOURCE_BINDING_MISMATCH`, `INVALID_COMPONENT`, `INVALID_TENSOR`,
`INVALID_CODEC_POLICY`, `INVALID_FRAME_TABLE` and `STATE_IDENTITY_MISMATCH`.
Carry only an optional component name and byte/element offset, never tensor
contents or a fabricated model result. A missing host file is `UNAVAILABLE_ARTIFACT`.

All lengths/ranks/products/conversions use checked integer arithmetic before
allocation. Enforce the 16-MiB container, 256-KiB manifest and fixed component
limits. Reject nonfinite parameter/frame values and out-of-range table IDs.
The exact original JSON and policy hashes must match trusted constants, and
their decoded values must agree with the raw table components. The loader is
not permitted to accept arbitrary alternate vocabularies or matrix tables just
because their self-declared digests are internally consistent.

Capability metadata separates `reference_evidence = CLAUSE_ADAPTER_PRESERVED`
from `native_behavior = NOT_RUN | EMPIRICAL_NATIVE_REFERENCE_PRESERVED`.
The second value requires a trusted accepted comparison receipt binding this
artifact, native binary/operator profile, schemas and runtime; it is not read
as a self-authorizing manifest flag. Without it, the loaded artifact is available
only to an explicitly admitted comparison harness. A regular host request must
return `UNAVAILABLE_NATIVE_QUALIFICATION`; discovery reports native qualification
as unavailable. No browser/chat wiring is authorized by this specification.
The metadata always reports the narrow operation/scope, artifact/codec/policy
identities, statelessness, floating-point reference execution and absence of
general generation/context/coding/final-kernel qualification.

## 5. Request, state and response

**Definition — transport-independent API.** The request remains exactly
`{schema:"uor-r4.text-to-clauses/1", text:bytes}`. Rust represents the complete
buffer as `&[u8]`/owned bytes. No JSON string/base64/HTTP mapping is chosen here;
#1084 must specify a transport without silently normalizing these bytes.
No external segmentation, IDs, roles, state, query selector or answer may enter.
Artifact/qualification admission precedes request inspection. After admission,
the unchanged #1085/#1094 byte limit, lexer, punctuation, grammar, error tags,
offsets and refusal precedence apply. Native refusal must emit the exact
`uor-r4.text-to-clauses-result/1` refusal object and perform zero model forwards.

The adapter's successful spans, `inputs[1,5,13]`, `lengths[1,5]`, raw SHA256 and
derived-input SHA256 follow the original policy byte-for-byte. Only IDs and
lengths reach learned computation. The reader may use IDs 52–57 as aliases,
but output decoding uses the original core vocabulary: ID 52 remains
`<unused-0052>`, never `not`. The model's actual full-4096 argmax is returned even
when unexpected; ID 11 `unknown` is a model token, not calibrated abstention.

The nested model result keeps exactly the #1085 fields and schema
`uor-r4.text-binding-result/1`, status `MODEL_TOKEN`, original source CIDs,
policy/raw/derived identities, token ID and core spelling. The optional outer
native receipt defined in the contract supplies native artifact/state/runtime
identities and work accounting without altering that result object. It adds no
`ANSWER`, `ABSTAIN`, `CONFLICT` or `CLARIFY` policy. No roles/labels/answer table
or cached oracle output may supply a model token.

One request replaces all four facts. There is no persistent conversation state,
KV cache, incremental update, retrieval or future-token input. Loaded parameters
and all policy/frame tables remain immutable. Per-call audit counters and scratch
are reset, cannot affect subsequent inference, and are not semantic memory.
Support one request at a time per engine; reject overlapping calls as `BUSY`.
The comparison harness may inspect complete tensors and role argmax as diagnostics;
they never route computation or enter the public request.

## 6. Operator and numerical contract

**Definition — `cpu-scalar-f32-f64-1086/1`.** The first native candidate is a
release-built CPU scalar implementation with fixed operation order, no fast-math,
FMA contraction, reassociation, quantization or algebraic gauge cancellation.
Use nearest-even rounding and preserve subnormals (no FTZ/DAZ).
Each product and addition is rounded separately to its declared type; reductions
start at positive zero in ascending contracted-index order. Bias is added after
the completed dot. Convolution contracts input channel then kernel offset, with
zero extension across clause edges. No arithmetic crosses a clause in the reader.
This is a new native reduction profile, not Torch/Accelerate's reduction order.
Its adequacy is an unverified empirical hypothesis.

The learned reader is f32 embedding → radius-two Conv1d → exact-form GELU
(`approximate="none"`, Gaussian CDF, not tanh) → three-role Linear → stable
softmax over each clause's valid tokens. Padding embeddings are zero before
convolution and masked logits are negative infinity only for padding. Softmax
subtracts the maximum, exponentiates and sums in ascending position order, then
divides; padded probabilities are exact positive zero. Finite unmasked inputs
and outputs are required. GELU uses `0.5*x*(1+erf(x/sqrt(2)))` with f32 operations
and f32 math; exact bit constants and separately rounded GELU steps are in the
profile. Freeze existing `libm 0.2.16` (`erff`, `expf`, `sqrtf`), `arch` enabled,
force-soft-floats/unstable features disabled, first target `aarch64-apple-darwin`
with generic CPU. The contract pins the Cargo.lock checksum and inspected source
identities. The later release binds exact rustc/lockfile/flags/binary and selected
architecture path. This is a floating-point erf-formula approximation, not exact
real arithmetic or a portable equivalence claim.

LayerNorm uses last-axis biased variance: f32 mean, f32 squared deviations from
that mean, f32 mean variance, then `(x-mean)/sqrt(variance+epsilon)`. Epsilon is
the nearest f32 to decimal `1e-5`, inside the square root. The 128/64 input norms
have no affine parameters; only the final 64-wide norm has learned scale/bias,
applied as multiply then add. Dense matrices retain `[out,in]` weight layout.

Every valid token, including punctuation, folds `current = multiply[current,leaf]`
continuously across fact0…fact3/query. Padding does not fold. Each clause uses
its end frame. For every 64-vector, sixteen consecutive 4-lane blocks are encoded
by the token/source frame transpose, transported by destination transpose times
source, weighted/summed, then decoded by the destination frame. These operations
are f64; f32 values/role coefficients are widened exactly and results are cast
to f32 only after decode. Compute all fifteen role mixtures, including the unused
query-location role. Consume fourteen soft roles, never their argmax.

Concatenate owner then object for the query and each fact, normalize each
resulting 128-vector once, then apply unchanged f32 Q/K projections.
Normalize locations and project V.
Append learned null K/V in the identity frame. Transport all five K/V entries
to the query's clause-end frame. The full 64-wide f64 query/key dot reduces by block 0..15 then lane 0..3 and is rounded
to f32 **before** division by 8 and f32 five-slot stable softmax. Weighted V and
decode stay f64 until the decoded context casts to f32. Then unchanged f32
output projection, affine LayerNorm and tied full-4096 head produce logits.
No residual, equality mask, top-k restriction, frame permutation or control is
enabled. The operator audit pins the exact source equations and indices.

Argmax chooses the lowest index on an exact finite tie, over all 4096 logits
or each valid role distribution. Nonfinite unmasked/intermediate/output values
return `NUMERICAL_FAILURE`, never a token. No epsilon tie rule or location filter.
The declaration specifies an algorithm; it is not a universal floating-point
refinement proof and does not predict cross-runtime bit equality.

## 7. Separate minimum empirical bridge decision

**Empirical Criterion — question.** Does the one exact exported artifact,
loaded by this native boundary, preserve the pinned Python R4 reference on the
complete existing authoring stratum? Use the original **320 valid and sixteen
refusal fixtures** from #1094 in their frozen file order. They cover four base
groups × five query variants × four fact forms × four surface profiles, and
one example per refusal family. This is engineering reuse of known fixtures,
not an independent semantic holdout or exhaustive malformed-input test.
No new population, withheld access or period-removal variants are necessary.

The later child must freeze exact authoring raw/reference file digests from
the accepted curation/preflight records, verify the actual files at admission,
and bind independent fixture ownership. Its harness gives only raw request bytes
to both engines; evaluator roles, labels and references stay outside them.
The reference uses the unchanged R4 wrapper (`execution="r4", control="none"`)
with the accepted reader/core and frame identities, Python 3.12.14/Torch 2.7.1
CPU Accelerate, four intra-op/one inter-op threads. **Both arms use B=1**, one
request at a time, in identical order. This explicitly differs from #1094's
batch128 numerical envelope; no old time or byte-equality result is inherited.
Native has one scalar worker because fixed scalar reduction order defines this
candidate, not because one core has been measured fastest. No accelerator search
or calibration belongs to this small comparison.

Before model work, a zero-forward loader gate must reject one independently
constructed mutation for each of eleven loader error classes (including wrong
expected artifact identity), plus a missing qualification receipt for regular inference. Derive
mutations from the actual export, freeze their bytes/expected errors independently,
and require all to refuse without inference. Each mutation uses a fixture-only
ExpectedBinding with recomputed whole-container SHA and, where needed, prior-stage
component digests to reach its intended check. Accepted source/state/codec/frame
constraints remain fixed; these synthetic bindings never qualify a real export.
The contract distinguishes layout/hash errors from tensor dtype/shape/finite errors;
all mutation copies count against the byte ledger. Compare decoded tensors/tables/codec
to accepted inputs exactly, verify the tied head is an alias in the implementation,
and require duplicate export byte identity. These are engineering checks, not
new mathematical proof. The gate permits at most twelve loader attempts: eleven
rejected mutations and one successful valid load for the qualification probe and
decoded-state checks. Reuse that gate engine for zero-forward checks and unload
before scoring. Report every stage and wire/tensor component reached by rejected attempts; at
most 22 model-state validations (reader/core, not wire tensor components) can
partially occur across those eleven
failures. All gate work remains under the export/integrity time/RSS/byte caps.
A missing asset/release or failed gate stops scoring.

One reference process and one native process execute sequentially; repeat both
in fresh processes once. Every valid row requires exact raw/derived input
identities, token IDs/lengths/spans, frame indices, full-vocabulary argmax/token
spelling and fourteen consumed role pointers. Compare all entries of f32
`role_attention[1,5,3,13]`, `role_vectors[1,5,3,64]`,
`binding_attention[1,5]` and `logits[1,4096]`, including the unused soft role.
All must be finite (padding probabilities zero), with **maximum absolute error
≤ 1e-5 per tensor per row, relative tolerance zero**. Report maxima and locations,
not just a pooled average. This newly frozen ceiling is a conservative engineering
criterion of the same scale as #1079's numerical criterion; it is not a derived
error bound or #1094's stronger byte-equality observation. Never tune it to native
outputs. Also report correctness against frozen labels for both arms, retaining
every reference error; require 320/320 answers and 4,480/4,480 consumed roles in
each arm. Reference nonfinite/error, failure of these reference floors or its own
fresh-replay failure is `UNAVAILABLE_NATIVE_REFERENCE`, with the errors retained.
It does not diagnose the native candidate. All sixteen refusals require exact tags/offsets and zero forwards.

Within each implementation, fresh replay must reproduce exact deterministic
results, tensor bytes, state identities, parser diagnostics and work counts.
Cross-runtime equality uses the declared numerical criterion; replay timing/RSS
are reported separately and excluded from deterministic digests. Persist
both arms' full compared tensor bytes in both phases, with shape/dtype/ordered row identities,
plus discrete outputs, reference errors, receipts and bounded failure detail.
Historical #1094 tensor digests locate prior evidence but cannot reconstruct its
removed raw oracle streams or replace this new reference observation.

## 8. Budget, stops and causal next actions

The separate child receives no execution authority from this specification.
Its exact release must name immutable outputs and durable start/progress/stop/
completion receipts. One admission consumes that envelope; partial evidence is
preserved, and any stop overrides apparent completion. No automatic retry,
post-result tolerance/batch/backend change, input repair, new fit or time reset.

The comparison permits 320 × two arms × two phases = **1,280 logical row
forwards**, four scoring/replay engine loads (eight model-state loads: reader and core per engine), zero
optimizer updates and zero refusal forwards. The gate separately permits one
successful engine/two model-state loads: total successful engine loads are at most
five, successful model-state loads ten, plus the explicitly bounded rejected gate
work above. At most 120 seconds for fresh
export/integrity admission, 120 for execution, 120 for fresh replay, 360 cumulative;
clocks include identity checks, process startup, all artifact/output writes and
cleanup. A monotonic external supervisor covers final write/exit tails and emits
its own wall receipt; no silent unmeasured tail may renew a budget. Native build
is a separately recorded one-time preparation cost (release, offline, hard
15-minute/2-GiB new-build-output cap), not evidence of model parity. A build stop
cannot launch model work. No native timing is claimed before implementation.

Combined coordinator + one active worker RSS is bounded by 3 GiB. The complete
export/corpus/results/temporary ledger is capped at 128 MiB, counting reused
authoring files once and both independent export copies plus all new writes.
Each artifact is capped at 16 MiB. The minimum four-phase f32 tensor payload is
4 × 320 × (195+960+5+4096) × 4 = **26,910,720 bytes**; exact source-derived weights
are 1,714,188 bytes per export, with codec/frame/policy overhead separately
measured. This arithmetic supports a small bounded bridge, not a performance
prediction. Pre-admission accounting must show the actual complete byte budget
fits; no payload deletion to hide an overrun. Preserve full compared tensors in
this smaller campaign; no temporary tensor deletion is required by this design.

| Terminal | Required consequence |
|---|---|
| `NATIVE_REFERENCE_PRESERVED` | All loader, identity, numerical, discrete, refusal, replay and resource criteria pass. Independently review and deliver only this bounded native-reference qualification, then hand its exact artifact/operation to #1084; #1087 remains a separate lowering contract and #954 remains blocked. |
| `NATIVE_REFERENCE_MISMATCH` | A valid, reproducing reference accompanies a candidate numerical/discrete/refusal/replay miss; a failed zero-forward loader gate records a candidate engineering failure without a numerical verdict. Preserve complete evidence, keep Python reference qualification and refuse native capability promotion; a separately scoped implementation repair needs a new reviewed envelope. |
| `UNAVAILABLE_NATIVE_REFERENCE` | Missing provenance/asset/runtime/input, incomplete export or invalid/non-reproducing reference; no model verdict and dependent work remains `NOT_RUN`. |
| `ABORTED_NATIVE_REFERENCE_BUDGET` | Any cap, interrupted process or incomplete final receipt. Preserve partial work and stop; no renewed allowance or promotion. |

## 9. Closure and preserved limits

#1086's DoD is the accepted schema/ownership/operator/comparison contract, not
implementation. After independent review and protected delivery, close this
specification and park its separately owned implementation successor. The later
child must implement and obtain independent approval of its concrete export and
comparison release before observing outcomes. Nothing in this task exports an
artifact, runs a model, opens the sealed population or implements that child.

Retain #1094 `CLAUSE_ADAPTER_PRESERVED`, its original unavailable preparation,
#1096 runtime-only readiness, #1079 `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, #1082's
descriptive limits and all earlier negative results. Source provenance and exact
finite identities do not establish native behavior, general parsing, semantic
novelty, variable/hierarchical context, generation, reasoning, coding or geometry
superiority. NEMESIS/W33/UOR sources supply attributed boundary questions and
identity patterns only; no outside proof or capability claim is imported.
#973 stays open and #954 blocked.

## 10. Specification checks and storage review

The [metadata reconciliation](r4_native_reference_1086_validation.json) checks
the static tensor inventory against the independent source audit, accepted asset
and state identities against retained bindings, component offset/shape arithmetic,
source-file hashes, logical-work/tensor-byte arithmetic and JSON field integrity.
It validates this specification only. Claim-wording and diff-whitespace checks
are named by the active issue; broader testing/QA remains dormant. Independent
operator and boundary reviews supply the decision evidence, not model execution.

The existing storage audit at
`/Users/casey.allard/Documents/Codex/storage-audit-2026-09-02.md` was consulted.
The isolated worktree occupies about 1.12 GiB; the review observed 43,501,548 KiB
available (about 41.5 GiB), compared with 46,031,300 KiB before creation. This is
a filesystem snapshot, not attribution of every intervening byte. No files were
deleted. The mixed main checkout, all referenced worktrees, original model/corpus
assets, sealed population, source caches and receipts remain preserved.
