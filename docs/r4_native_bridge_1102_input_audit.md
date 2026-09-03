# Independent input and release audit — #1102

**2026-09-03 — `INPUT_METADATA_VERIFIED_RELEASE_NOT_ADMITTED`.** The original
authoring raw/reference files are available and match their accepted byte
commitments. This is an input-identity observation, not a new parser/model
result or an execution release. Independent release review remains pending
the concrete implementation, runtime, native binary, input/output bindings and
bounded supervisor. Later dispositions are appended to this record.

The audit starts from protected revision
`93613bf82782ca78406fe2739dcc8d9e1d0f2b9e`, which delivered #1086 through
PR #1103. The live #1102 issue is open and assigned to `Casey-allard` for the
current activation; its earlier parked handoff text remains historical. The
reviewer is independent of the new loader, exporter, numerical implementation,
parser and harness authors, and edits only this audit and its input metadata
[companion](r4_native_bridge_1102_input_audit.json). No implementation is authored
or executed by this reviewer.

## Authority and unchanged decision

The [#1086 machine contract](r4_native_reference_1086_contract.json) is SHA256
`e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`.
Its [normative prose](r4_native_reference_1086.md),
[independent review](r4_native_reference_1086_review.md) and operator/source
audits remain the accepted definition. Current `AGENTS.md` and the live issue
activate only the named comparison and its admission checks. Broad QA is dormant;
the protected queue's compatibility acknowledgements are transport status.

The one question is whether the exact exported artifact and native implementation
preserve the pinned Python raw-text reader/core/R4 reference on the complete
existing authoring stratum. Both arms use B=1, their own fresh-process replay and
the frozen exact discrete criteria. All four full f32 tensors are compared with
per-row/per-tensor maximum absolute error at most `1e-5`, relative tolerance zero,
including the unused query-location soft role. This candidate's numerical
adequacy and resource feasibility have not been observed.

## Original authoring files and independence

The original corpus root is
`/Users/casey.allard/.codex/uor/issue-1094-curator`.
Only its two authoring payload files were streamed as uninterpreted bytes for
SHA256 and byte-count verification. No JSONL row, reference tensor, label or raw
request was parsed in this audit. The selected files and checked parent paths
were not symlink aliases; both files are regular files with mode `0444`.

| Relative path | Verified bytes | Verified SHA256 |
|---|---:|---|
| `authoring/raw.jsonl` | 215,866 | `58ecc23bf6a587508355266e039bb0cb3bdbf7597204fec40aee2c34d7f89503` |
| `authoring/reference.jsonl` | 456,980 | `d6740ea56caa6e7df9dea7be4b9636880056629549c5e72783ba988a8ca3c660` |

The exact resolved paths are the corpus root joined with these relative paths.
Their combined **672,846 bytes are counted once** as reused authoring files in
the new byte ledger. Copies, if made, are additional new bytes. The separate
#1094 historical debit and consumed execution envelope do not become a new
#1102 allowance, and the old full corpus is not an input to this comparison.

Two metadata files at the original corpus root were also rehashed and matched
their embedded committed records:

| Metadata | Bytes | SHA256 |
|---|---:|---|
| `selection.json` | 8,982 | `892e3239773e8a14e72ee650dc12c98ee4e1a5b432b69365a60cef8b15c9b5fa` |
| `population.json` | 1,565 | `ad5bf0fdecb66b0de9e28c98941cf0fb2c6f737c7e1be3cbf48570822c65ba30` |

The source of these commitments is the protected
[public #1094 curation record](r4_text_clause_adapter_1094_curation.json).
The accepted [authoring preflight](r4_text_clause_adapter_1094_evidence/authoring-input-preflight.json)
is 3,032 bytes with SHA256
`f79df8623038961d899ba727d99bb69b39754b1878d10bbed9da0bfe03e5ee82`.
It records `AUTHORING_INPUT_EXACT`, 320/320 valid rows and 16/16 refusals, twenty
valid rows in each of sixteen form/profile cells, with zero model loads/forwards.
Those counts are retained observations, not a newly executed preflight.

The [original independent source review](r4_text_clause_adapter_1094_review.md)
and [curation history](r4_text_clause_adapter_1094.md) establish that a curator
separate from the adapter author prepared the raw text and evaluator annotations.
The curator did not import the adapter, model, or historical render/parse/decode
helpers. Its source identity is
`4d8012f8647cbbf57136999de6522af7b27bf994466744d0efac20e435a5defb`.
Selection was frozen before text generation. This audit independently verifies
the retained files; it does not reauthor their content or expand their population.

The four authoring base groups, in the original selected order, are:

| Family | Original source group | Group SHA256 |
|---|---:|---|
| same owner | 180 | `00491608a8c670917815d7f837af2b004dd8cd11fe16f5f921a08538271cc9c2` |
| same owner | 674 | `00b3d9cd871cf8e619ec126022e9690802edf46179dbd722074880daa2a42282` |
| same object | 631 | `000e45452183f64b6e6b176eadc919bfa0de71b1b66174d4c9d02593aff577f6` |
| same object | 493 | `000e7c92141ae16685e949f552fa60255419176183be58fde5c53ce24ce34164` |

The retained valid-row order is selected group, variant 0–4, fact form 0–3,
surface profile 0–3. The sixteen refusal rows remain in their original file
order and represent one case per frozen refusal family. References and targets
belong only to the independent evaluator; learned inference receives raw request
bytes through its adapter and then only derived IDs/lengths. Four known groups
and their renderings are not independent semantic generalization trials.

## Knowledge retrieval and original research

The public project index was queried for `1086 native authoring` and
`NEMESIS W33 carrying`. The native criterion record
`kb:707c06ae968e06b816ff4b8e9a3f7e1b48a54bfa2d7d4290774d2dad81c33c24`
retains `Unproven / SPECIFIED_NOT_RUN`; its body SHA256 is
`2f1b3491ef1d0547b72f94cf275adaed6ad33cf32f2275749df575d3525f3b7d`.
The retrieved historical relevance note is
`kb:2531e7ab3b4d69cea48cdbfa53cf620caedbb2cd5c654392875b56d597eb9038`,
body SHA256 `490e88c487c32245a33160545a187e006922d491e7cd1fa2b868746457256e5b`.
These records are retrieval aids; current native state and protected source
records resolve their scheduling and evidence status.

The original sources were checked within the limited boundary needed here:

- **NEMESIS**, revision `0d106967843c2c96477cf3e57aeff213e7db1c97`:
  Mark / NEMESIS 3D Studio's carrying report, pages 1–3. The retained original
  PDF was rehashed at 99,374 bytes, SHA256
  `697d48b70a1499a1fd70d8f1a4c285606a198a3831250425ae11439f37b395cc`.
  Its page-marked extraction was rehashed and read at 6,910 bytes, SHA256
  `4ea32402e9398f7830a243e5c5f8db30df2ff497e1d8cf559c5949228e2f8376`.
  State correspondence and preservation of transitions motivate explicit
  model/operator identities; the report supplies no implementation proof,
  error bound or native behavior evidence for this reader/core.
- **W33**, revision `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`:
  original `analysis/w33_fractal_microvm_runtime.py`, lines 38–58 and 314–329,
  rehashed at 58,882 bytes, SHA256
  `875b53408cc5312b60b5a6254dbac80a9a1324c89cdf24936488d7a4744e90ca`.
  Explicit instruction admission, canonical JSON and preserved content-store
  entries are relevant implementation patterns. They do not implement or
  qualify R4's export, loader or numerical computation.
- **UOR kappa-registry**, revision
  `2af86560a177fc9651b6c0e92e7974140ed77dd5`: original
  `crates/kappa-core/src/kappa/compute.rs`, the byte/value hashing definitions,
  retrieved as `kb:6b8e284c02461ae77686686561ce481cea3ff3017b300af2c9d17f6da1108430`
  with body SHA256
  `e363cfffd908383d502ee8fdea50cd861bba621fbc705c7b56595c9cad4262b2`.
  Raw byte hashing and canonical structured-value hashing are separate
  realizations. Neither a digest nor its type proves model-output preservation.

Original URLs, precise pins and source provenance remain in the accepted
[#1086 original-source manifest](r4_native_reference_1086_source_manifest.json).
No upstream source, proof witness or model was executed, no code or report was
vendored, and no dependency pin was changed by this audit.

## Independent release requirements

`NOT_ADMITTED` remains binding until the following concrete objects and their
relationships can be inspected together:

1. **Executing source and binary.** Exact final loader/exporter/parser/numerics/
   harness/coordinator source closure and revisions; the protected contract
   hash; native release binary; rustc/target/flags/lockfile and the pinned libm
   profile. Executing paths must correspond to these identities, rather than
   merely matching another checkout's recorded files.
2. **Runtime and access.** Exact Python 3.12.14/Torch 2.7.1 CPU Accelerate
   interpreter/package/runtime identities, four intra-op and one inter-op
   threads, the native scalar profile and clean child environments. Bind the
   actual hardware and all runtime resources. Keep evaluator references,
   labels and prior outcomes outside the inference path. Any claimed process
   access restriction needs its concrete implementation and evidence; the
   historical #1096 readiness result alone cannot establish a changed profile.
3. **Inputs and provenance.** Recheck these authoring file commitments at
   admission; bind original asset/state/codec/policy/frame identities before
   any source asset deserialization. Preserve immutable input order. The
   exporter may not initialize a model, fit, normalize or change accepted
   values. No withheld path, raw/reference payload or old revealed correction
   is admitted as a #1102 input.
4. **Outputs and durable accounting.** Exact exclusive output/build paths,
   safe path relationships, immutable release and durable attempt/start/
   progress/stop/completion records. A consumed or interrupted envelope cannot
   silently restart. External monotonic supervision must include final writes,
   process termination and receipt tails; any stop overrides apparent success.
5. **Frozen work and decision.** At most two byte-identical exports; twelve
   zero-forward loader attempts, including eleven rejected classes and one
   successful engine reused for state/qualification checks then unloaded.
   Four scoring/replay engines consume 1,280 logical forwards with zero
   updates/refusal forwards. Overall successful loads are five engines and ten
   reader/core model states; rejected attempts report up to twenty-two partial
   model-state validations and every reached wire/tensor component. Persist
   all four arms/phases' full compared tensors and complete reference errors.

The fixed resource limits remain: one offline native build, at most 900 seconds
and 2 GiB new build output; a separate 120-second export/integrity phase,
120-second comparison and 120-second replay, 360 seconds cumulative; 3 GiB
combined coordinator/active-worker RSS; 128 MiB complete experiment byte ledger.
Original authoring files count once, while all new copies, both exports,
mutations, temporary files, results and receipt writes count. The four full
tensor streams require 26,910,720 bytes before metadata. No deletion may hide
an overrun. Missing provenance/reference/runtime yields unavailable, a valid
reference plus candidate failure yields mismatch, and any budget/interruption
stop prevents promotion. No outcome authorizes automatic tuning or retry.

## Present limits and next review boundary

There is no current authoring-file availability or identity blocker. Runtime,
binary and release admission are pending concrete evidence, not assumed ready.
The original #1094 unavailable preparation, #1096 runtime-only result, accepted
bounded #1094 comparison, #1079 weak-control result and #1082 descriptive finding
remain unchanged. #973 remains open and #954 blocked.

This audit has performed zero model loads, deserializations, forwards, fitting,
exports, comparison, replay, build or runtime probes. It has read no withheld
payload and changed no corpus or runtime permissions. The next reviewer action
is to inspect the completed executing code and exact release bindings before
deciding whether the sole bounded export/comparison envelope is admitted.

## Runtime-mode design assessment — not an execution release

The proposed fixed-target native admission check may read FPCR with one
architecture-gated `mrs` instruction, retain the raw value and reject an
incompatible mode. On AArch64, nearest rounding is RMode bits 23:22 equal to
zero; FZ is bit 24 and FZ16 is bit 19. FZ16 names the half-precision control,
not bit 16. Requiring AH bit 1 and FIZ bit 0 to be zero additionally excludes
alternate/input-flush behavior where those fields are implemented. These field
definitions are taken from Arm's
[FPCR architecture register specification](https://documentation-service.arm.com/static/6526e1bd9e189a266cef8412).

This is suitable as direct evidence of the measured thread's control state.
Read and check on the actual native inference thread before arithmetic and
after successful or failed worker execution. A metadata probe in another
process does not establish the subsequently executing thread's FPCR. No `msr`
write, rounding/flush-mode repair or unsafe payload cast is needed. Unsupported
architectures must report unavailable for this fixed candidate.

A small documented unsafe assembly block can use an integer register output
and `nomem`, `nostack`, `preserves_flags`. It must not use `pure`: the result
depends on execution-context state rather than supplied operands. These option
obligations follow the [Rust inline-assembly reference](https://doc.rust-lang.org/reference/inline-assembly.html).
Keep the architecture-specific observation outside any portable module that
forbids unsafe code. The review accepts this limited design direction; the
actual code and receipts still require inspection. No FPCR read or other
runtime probe was executed by this reviewer. Control-state evidence is neither
a mathematical proof nor a substitute for the frozen numerical comparison.

## Early source findings — implementation still in progress

The first loader/wrapper inspection found the following release blockers in
the in-progress source. These are source observations, not executed failures;
their disposition will be checked against the final source identities.

| Finding | Required correction before release |
|---|---|
| `INPUT-AUDIT-1102-01`: historical frame-tree recipe | The draft built records containing only `path` and `cid` and omitted the historical LF. `provenance.py::tree_cid` hashes sorted records containing `bytes`, `cid` and `path`, using `canonical_json_bytes` with its LF. Preserve that old recipe separately from the new manifest canonicalization. |
| `INPUT-AUDIT-1102-02`: qualification input duplicate keys | The draft parsed qualification JSON into `Value` and then checked field sets. Parsing can erase duplicate keys before those checks. Enforce the contract's duplicate-key rule at the byte/deserialization boundary; a trusted digest does not replace schema validation. |
| `INPUT-AUDIT-1102-03`: invalid component-name diagnostics | The draft emitted the supplied component name when the name itself failed validation. `NativeError.component` is a fixed contract component name or null. Use the expected template name or null for that mismatch. |

The inspected files are `crates/uor-r4-core/src/learned_reference/loader.rs`
and `mod.rs`; the historical recipe was traced to the actual public
`text_clause_adapter/contract.py` and `provenance.py` source. No model or frame
payload was opened to identify these issues. Release remains `NOT_ADMITTED`.
