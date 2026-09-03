# Independent specification review — #1086

**2026-09-03 — `INDEPENDENT_SPECIFICATION_ACCEPTED`.** The reviewed
`R4LearnedReferenceV1` specification satisfies #1086's bounded definition of
done. No unresolved blocker remains for this specification decision. This
acceptance covers the export, loader, consumer and future comparison contract;
it accepts no export, native binary, execution release or native behavior.
Closure still requires verified delivery through the protected PR path.

The reviewer worked independently of the specification author from protected
baseline `eade29f4b78435e9857936786426bb34e596b301`, read the native #1086 issue,
the accepted #1085 contract and #1094 evidence, inspected relevant tracked
source, and reviewed the separate operator and original-source audits.
Only this review record was edited by this reviewer.

## Exact reviewed specification

| Record | Bytes | SHA256 |
|---|---:|---|
| [Normative prose](r4_native_reference_1086.md) | 29,120 | `568b4aae769f7c91f0ab5a0ebd7cf2d0263cc3198ec29c6b5096188392804341` |
| [Machine contract](r4_native_reference_1086_contract.json) | 25,469 | `e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115` |

These identities describe the specification reviewed here. They are not model,
native-state or native-artifact identities. The contract deliberately leaves
actual future artifacts and implementation/runtime release identities to the
separate implementation task; it does not admit executable placeholders.

## Decision coverage

| Obligation | Review conclusion |
|---|---|
| Accepted state and export | The five original asset bindings, historical reader/core state CIDs, frame-tree CID and policy hash remain distinct. Constructor-free export must check original bytes and decoded state, preserve all fourteen parameter tensors and the tied head, and produce identical complete bytes twice. |
| Wire format and hashes | Fixed magic, framing, field sets, canonical JSON, twenty-one ordered components, endian/layout rules and checked lengths define one container. Whole-file, component, historical state and ordered native-state hashes have separate coverage and trust roles. |
| Loader and admission | Caller-owned immutable bytes, expected whole identity, explicit trusted field equalities, ordered validation and focused errors precede usable state. A self-consistent manifest cannot select alternate assets, codecs, frames, arithmetic or its own qualification. |
| Input/state/output | The unchanged byte request, four-fact grammar, query form, vocabulary aliases, refusal precedence and exact original result schemas are retained. Actual full-4096 argmax is returned using the core codec; `unknown` remains a model token. Scratch is reset, parameters immutable and the operation stateless. |
| Consumer and ownership | The proposed opt-in core/API research modules are identified as future symbols. Native qualification requires trusted result evidence; default R4G1/serving, service transport, final integer lowering and typed UOR integration retain their separate owners. |
| Numerical candidate | The scalar f32/f64 reduction profile, pinned math primitives/constants, original frame values, two R4 transport stages, all fifteen soft roles and fourteen consumed roles are specified. This defines a candidate rather than demonstrating its adequacy. |
| Future comparison | Both arms use B=1 on the same existing authoring fixtures and retain all four complete compared tensors. Exact discrete criteria, the new absolute tolerance, independent replay and reference-failure disposition are frozen before observations. |
| Resources and consequences | Distinct build, gate, execution and replay accounting includes rejected loader work and full output retention. Every terminal has a distinct consequence; no stop, failed reference or candidate mismatch authorizes an adaptive retry or native promotion. |

## Findings resolved during review

1. **Loader mutations could stop at the wrong check.** A changed payload with
   its old whole-file hash only exercises artifact identity. The accepted
   contract permits fixture-only expected identity and prior-stage digest
   updates to reach the intended stage, while accepted source/state/codec/frame
   constraints remain fixed. These fixtures never qualify a real artifact.
   Layout/hash, tensor and frame failures now have distinct stage ownership.
2. **Trusted manifest fields needed literal equality obligations.** The final
   records explicitly compare the manifest contract digest, export-release
   digest and accepted binding against `ExpectedBinding`, returning
   `SOURCE_BINDING_MISMATCH` before component validation. The supported and
   expected operator profile is checked separately as `UNSUPPORTED_PROFILE`.
3. **A new reference failure could be mislabeled as a native defect.** The
   B=1 Python reference has not yet been observed. Reference errors, nonfinite
   values, failure of its answer/role floors or its own replay now produce
   `UNAVAILABLE_NATIVE_REFERENCE`, preserving the errors. A numerical candidate
   mismatch requires a valid, reproducing reference. A failed loader gate is
   separately an engineering failure with no numerical verdict.
4. **Gate loads were absent from the scoring load count.** The final gate has
   at most twelve loader calls: eleven rejected cases and one successful engine
   reused for decoded-state and missing-qualification checks, then unloaded.
   The latter adds two model-state loads to the four scoring/replay engines and
   eight model-state loads: five successful engines and ten model-state loads
   in total. Up to twenty-two partial reader/core model-state validations are
   reported across rejected attempts. These count reader/core states, not the
   fourteen individual parameter tensors; every wire/tensor component reached
   is also reported. All gate work consumes the export/integrity envelope.
5. **Cross-runtime and replay predicates needed separation.** The final
   criterion is maximum absolute error at most `1e-5` for every compared tensor
   on every row, relative tolerance zero, plus exact discrete results. Fresh
   replay is byte-exact within each implementation. Neither predicate inherits
   #1094's batch-128 timing or its same-Python byte-equality result.

## Bounded checks and evidence

The reviewer used standard-library JSON, hashing, integer shape arithmetic and
float-bit framing over tracked public source and specification metadata. No
project model/runtime module was imported. The following checks passed:

- Strict final JSON parsing with duplicate-key rejection, eleven ordered
  loader stages, specification-only authorization and zero recorded model work.
- Exact comparison of all five source asset metadata records and the four
  state/frame/policy identity fields against retained #1094 bindings. This
  compares recorded provenance; it does not reverify external model files.
- All twenty-one fixed offsets and dtype/shape byte products reconcile to
  **2,160,742 payload bytes**. Reader/core inventories have 141,571 and 286,976
  scalars, respectively, totaling **1,714,188 parameter bytes**.
- The tracked original policy remains 6,769 bytes and has its accepted SHA256.
  Historical state-CID framing agrees with the actual
  `zoology_release/development.py::_tensor_mapping_cid` and
  `provenance.py::canonical_json_bytes` source, including its LF delimiter.
- `320 × 2 × 2 = 1,280` logical forwards and
  `4 × 320 × (195 + 960 + 5 + 4096) × 4 = 26,910,720` retained f32 tensor
  bytes. Gate accounting is additional zero-forward work under its named caps.
- The parked implementation-successor draft preserves the admission order,
  caps, independent release, known-fixture boundary and causal outcomes. Its
  creation would be a handoff, not an execution release.

The [operator audit](r4_native_reference_1086_operator_audit.md) independently
reconciles actual source equations, tensor names/shapes, aliasing, frame folding,
cast points and pinned math-profile details. The
[original-source audit](r4_native_reference_1086_sources.md) attributes the
NEMESIS/W33/UOR carrying and identity patterns without importing them as proof
of this loader or neural behavior. The root
[metadata reconciliation](r4_native_reference_1086_validation.json) retains
its separate static check record. These are complementary specification and
source checks; they are not an implementation test suite or model result.

## Limits and handoff

The complete existing 320 valid authoring rows and sixteen refusal fixtures are
sufficient for this minimum engineering bridge decision. They retain all four
authoring groups, five query variants, four fact forms and four surface profiles,
plus one case per refusal family. An extra sixteen derived boundary examples
would not supply a distinct decision here. The selected fixtures are neither an
independent semantic population nor exhaustive malformed-input coverage.

The native tolerance, B=1 reference floors, fresh replay and resource feasibility
remain unverified hypotheses. No artifact export, native implementation,
deserialization, model forward, fitting, comparison, replay or withheld read ran
in this review. Finite identity comparisons and tied-head checks are engineering
requirements, not mathematical proofs. The specification does not qualify
general parsing, new semantic groups, variable context, generation, reasoning,
coding, geometric superiority or the final integer kernel.

Preserve #1094's bounded accepted result and unavailable preparation history,
#1096's runtime-only readiness, #1079's weak-control result and #1082's descriptive
limits. #973 remains open and #954 blocked. After protected delivery, #1086 may
close as a specification; its implementation/export/comparison successor stays
parked and unassigned. The next action on that successor, when separately
activated, is to bind a concrete implementation to this accepted contract and
obtain independent approval of its exact export/input/runtime release before
new model work.
