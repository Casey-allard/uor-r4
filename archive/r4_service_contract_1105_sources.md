# #1105 service/API and artifact-ownership source audit

2026-09-03. Parent #1084; R4 base `85d9eb8beca2a59ccda47e290afd483f7838982c`.
Disposition: **SOURCE_INSPECTED_NOT_EXECUTED**. This is independent input to the
ADR, not its final review. No implementation, model/asset deserialization,
build, fitting, forward, replay, browser launch, download or Git mutation ran.

The public knowledge index was queried first for `frontend-port-plan`,
`NEMESIS`, `W33`, `1084`, `kappa_from_value` and `uor-r4-wasm-chat`. The plan
record `kb:79fe0fff2588b12d7759be0f13f99d2156ab731166e23d71e899790ce48dba44`
and original registry record
`kb:6b8e284c02461ae77686686561ce481cea3ff3017b300af2c9d17f6da1108430`
were retrieved. Current repository docs, #1086 source manifest and #1102
public handoff were then read. Historical index state was not treated as
current eligibility.

The companion [source manifest](r4_service_contract_1105_source_manifest.json) records 12 original-source identity
rows and six current R4 source/metadata rows, including exact repository,
revision, path, SHA256, bytes, relevant locations and licensing. Eight original
research/UOR files were rehashed and their Git blob identities matched against
the accepted #1086 manifest. Three donor files were read using `git show` at
the exact pin and their Git blob identities verified. The W33 license and
NEMESIS page-marked extraction were also read and rehashed. No source was
copied into the worktree.

## Decisive transfer boundaries

1. **One Rust host owns readiness and immutable loaded model state.** The
   frontend can select a service-returned model/operation and display its
   artifact/state/qualification identity. It cannot infer readiness or
   permission from cache presence, a friendly model name, transport success,
   or another binary's qualification. The current `LoadedResearchReference`
   owns artifact bytes, parsed manifest, weights, frames and vocabulary;
   `qualify()` checks supplied receipt bindings, while `answer()` refuses
   without attached qualification. This is source evidence for the ownership
   boundary, not observed ordinary serving behavior.

2. **Transfer the donor's presentation selectively.** Original donor
   `Casey-allard/uor-r4-wasm-chat@5a10305126df62e838cadfec5fd509e0c9705fa7`
   provides useful CSS tokens (`index.html:23`), conversation rail/session
   selection (`:5584`), composer (`:2300`), model selector (`:5401`), Monaco
   editor (`:3987`), and explicit proposed-diff/apply/discard presentation
   (`:7324`, `:7373`). Only the first bounded operation belongs in the initial
   service flow. Editor/workspace/Git/preview features remain later explicit
   service capabilities. Browser session history must remain UI storage;
   the four-fact operation has no conversation-state continuation contract.

3. **Rewrite donor identity and execution wiring.** `index.html:2925` and
   `assets/js/uor_model_worker.js:58` label Qwen2.5-0.5B weights as GLM-5.3.
   The worker's fallback at lines 220–236 changes source while retaining the
   requested `modelId`. Its stop handler at 463–467 changes a flag and sends
   an acknowledgement; the inspected inference call at 414–431 has no
   cancellation signal/stopping criterion. The callback stops publishing
   chunks when the flag clears. These are concrete source reasons to require
   truthful identity and actual completion/cancellation semantics in a new
   provider. This audit did not measure cancellation behavior.

4. **Cache state is not artifact admission.** Donor `index.html:5227`
   labels a model cached if any matching ONNX filename exists; source
   substring purge at `:5378` is not a complete, typed artifact manifest.
   Native artifact ownership should use the admitted bytes and explicit
   identities, with loaded/qualified/running states distinguished. The
   donor's selected-model persistence and lifecycle presentation can be
   retained; its cache completeness and successful-stop claims cannot.

5. **The service must not reuse the measured CLI identity.** Current #1102
   public evidence accepts bounded native preservation and publishes
   qualification SHA256
   `61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369`.
   The `qualify()` implementation accepts a caller-supplied `RuntimeIdentity`
   and validates review/result hashes as hash-shaped strings; it does not
   discover the executable or interpret review evidence. The host must
   verify the external trust chain and its actual binary/runtime before
   providing the trusted digest. A newly linked service needs its own
   truthful host qualification decision. The ordinary `qualify()` success
   plus `answer()` flow remains **NOT_RUN**, and the measured executable
   has no service endpoint. #1105 should freeze that handoff without reopening
   #1102's consumed envelope.

## Research sources and identity conclusions

**NEMESIS** pin `0d106967843c2c96477cf3e57aeff213e7db1c97`, Mark / NEMESIS
3D Studio, *Integration of Hypercomplex Geometries as UOR Structure Carrying
Substrates*, pp. 1–3: PDF SHA256
`697d48b70a1499a1fd70d8f1a4c285606a198a3831250425ae11439f37b395cc`.
Its state-space, transition and primitive-interpretation criteria motivate
declaring exactly what the host preserves and which operation it exposes.
The report's arbitrary-network compression/query, zero-error and energy
assertions are not proof or measured capability for this Rust service.
No detected reuse license in the pinned inventory: attributed links and
independently written analysis only.

**W33** pin `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`,
`analysis/w33_fractal_microvm_runtime.py`, SHA256
`875b53408cc5312b60b5a6254dbac80a9a1324c89cdf24936488d7a4744e90ca`:
lines 51–58 define sorted compact ASCII JSON and SHA256, 314–329 implement
`ContentStore`, 746–795 resolve/rewrite a path, and 874–925 retain source,
target, route, message and prior receipt in a delivery chain. These are useful
immutable-object/receipt patterns. The exposed Python `blobs` dictionary and
`get()` do not themselves constitute tamper-proof storage, persisted atomic
writes, authenticity or a sandbox. No need to transplant radix-40 routing,
chamber mathematics or the microVM into the service. MIT; actual license
retains William Dembski Jr.'s copyright notice.

**UOR identities stay separately typed.** Re-inspected originals support the
following distinctions, with exact hashes in the manifest:

- `uor-addr@165b51e3e2113ee5d032730cde709335d4fe9b60` JSON realization uses
  JCS plus NFC; its commutative binary composition sorts operands. Neither
  convention is the #1086 ASCII receipt format or an ordered neural wiring
  identity. Preserve schema/version, hash axis, byte framing and role/order.
  Apache-2.0 license/workspace agreement is retained in the #1086 source audit.
- `kappa-registry@2af86560a177fc9651b6c0e92e7974140ed77dd5` separates opaque
  byte hashing from dCBOR structured-value hashing. Digest equality is an
  integrity comparison under a declared realization, not behavioral evidence
  or a mathematical injectivity proof. Cargo declares MIT OR Apache-2.0;
  no root license was found in the pinned original audit.
- `hologram@94ecb886811115491a77c8229e494965bea03fc2` separates content bytes
  from an ordered operation/parameter/operand derivation key. A derivation
  key requires deterministic semantics and complete length/count framing;
  it is not automatically the digest of the result bytes. Its unsorted
  left fold cannot inherit arbitrary-permutation invariance from binary
  commutativity. `hologram-ai@d5337ec2b3289fc8462abac2e627d165342079b2`
  explicitly sorts first. Neither collection identity represents typed
  topology without additional framing. Both declare MIT OR Apache-2.0;
  the archive's actual dual license identities are retained in #1086.
- `uor-matmul@3cc5882f210667f9ac00fd8c02c5b5957b493f5d` codec manifest binds
  schema and code/codebook digests, usefully distinguishing encoded artifact
  identity from decoded-state identity. This is upstream source review only;
  R4's older dependency pin stays unchanged. The retained MIT license versus
  Apache-2.0 Cargo declaration discrepancy remains unresolved; no code reuse.

The donor's MIT grant and Casey Allard / UOR-R4 Contributors notice were read
at the exact pin. Its loader references third-party libraries/CDNs; the donor
license does not establish the full asset/dependency closure of a future
bundled frontend. No dependency or branding asset was imported here.

## Claims and limits retained

The #1102 accepted result remains a measured CPU floating-point reference
on the fixed authoring stratum: known vocabulary/query forms, four facts and
one query. It supplies no broader-language, coding, conversation/context,
geometry-superiority, mathematical-proof or final integer/table-kernel claim.
#1079 remains `LANGUAGE_R4_PRESERVED_CONTROL_WEAK` (token control strong in
3/6 views against a required 6/6), #1082 remains `TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE`,
#1094 remains bounded `CLAUSE_ADAPTER_PRESERVED`, and its unavailable
preparation remains retained. #973 remains open and #954 blocked.

Boundary incident: a broad filename enumeration used to locate prior license
files encountered the existing withheld directory with `Permission denied`.
No withheld filenames or content were returned/opened. Subsequent reads were
restricted to exact public snapshot and repository paths. No user material,
models, evidence or source files were deleted.

Recommended ADR consequence: make the chosen raw-text operation and typed
host trust/ownership boundary explicit; carry the donor only as a presentation
reference. Defer serving implementation and its separately frozen qualification
check to the next owned task after this ADR is delivered.
