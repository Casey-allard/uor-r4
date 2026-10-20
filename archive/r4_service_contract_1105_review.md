# Independent specification review — #1105

**Date:** 2026-09-03
**Repository/base:** `UOR-Foundation/uor-r4@85d9eb8beca2a59ccda47e290afd483f7838982c`
**Issue:** [#1105](https://github.com/UOR-Foundation/uor-r4/issues/1105), child of [#1084](https://github.com/UOR-Foundation/uor-r4/issues/1084)
**Reviewer role:** independent specification reviewer; no authorship of the ADR or machine contract
**Disposition:** **ACCEPTED_FOR_PROTECTED_SPECIFICATION_DELIVERY**
**Accepted specification terminal:** `SERVICE_API_CONTRACT_SPECIFIED`

The exact ADR and machine contract identified below are internally consistent,
match the current learned-reference boundary, and are sufficiently closed to
govern the separately owned implementation. No unresolved specification blocker
remains. This acceptance applies to the specification and its source/metadata
evidence. It is not an implementation, runtime, model, cancellation, browser,
security-attestation or performance result.

## Exact reviewed records

| Record | SHA256 |
|---|---|
| `docs/adr/0006-native-four-fact-workbench-service.md` | `f6bd7135767868561bf65006e6167accf231193aafbb8ada3162d6e2c5aab330` |
| `docs/r4_service_contract_1105.json` | `337d66d025fc9ec3a1e8c21befc25198b015061235fefc98f9208f99412e7a7f` |
| `docs/r4_service_contract_1105_check.py` | `58d20401bd2fe029fe93587676fc4c2fc517405a2d4118761731c54d36ea7cee` |
| `docs/r4_service_contract_1105_checks.json` | `4a4f87df57f51f2253e061d6eda3219e47e160a2899b65fa365b69ea01edcc3a` |
| `docs/r4_service_contract_1105_sources.md` | `90a6c3616660007c3018bf22969e1ed2dbc32dd9038e1342481ead93e343b89d` |
| `docs/r4_service_contract_1105_source_manifest.json` | `963fd69c57a3c38e85cc1c236549c1db4744b82014a27eaf5a17b88baa9275e6` |
| `docs/r4_service_contract_1105_service_audit.md` | `a218da152385bcb327f6778931fd4f8e6d8b5aebb4f4c62306e179cb46fa6b67` |
| `docs/r4_service_contract_1105_storage.json` | `2134fbdaec66f32a001bcbf93f921977862b49507d0b8cfc856c56987f1f0ac6` |

I recomputed these eight digests from the final files. I also invoked the
retained metadata checker in its non-writing default mode. It completed in
0.232 seconds and independently reproduced `SPECIFICATION_METADATA_CONSISTENT`:
28 source rows representing 27 unique pinned source identities, 29 closed type
definitions, seven routes, five illustrative wire objects, 16 scenario IDs,
exact native error/refusal tags and original result fields, and no reassignment
of the historical qualification. The frozen receipt records its own 0.277-second
invocation and the same substantive result.

That checker establishes JSON unique-key parsing, type/reference closure,
illustrative object shape, exact source digests and selected historical identity
relations. It does not prove the proposed state machine, launch binding, HTTP
parser, process control, numerical behavior or browser behavior. The lifecycle
and trust conclusions below come from independent source and contract review,
not from interpreting the metadata receipt as runtime evidence.

## Current-source and retained-evidence grounding

The review compared the contract against the exact base-revision sources rather
than a previous service implementation:

- `crates/uor-r4-core/src/learned_reference/mod.rs`, SHA256
  `2ead8e40b9f3f095a057c5dec0c1d36bb38f0dd7ea61fc7ca7b5da7924401cec`,
  exposes `RawRequest { schema, text }`, `LoadedResearchReference::load`,
  `qualify`, synchronous `answer`, research-only `compare`, the caller-supplied
  `RuntimeIdentity`, exact `TextResult`, and the current `NativeError` fields.
- `crates/uor-r4-core/src/learned_reference/adapter.rs`, SHA256
  `90caf4d8873d754a3266f664eb284f52bb0cae118e96b4dc890d6139b9ea598e`,
  supplies the exact refusal spellings `UNSUPPORTED_SCHEMA`,
  `UNAVAILABLE_ARTIFACT`, `INPUT_LIMIT`, `INVALID_ENCODING`,
  `UNKNOWN_LEXEME`, `UNSUPPORTED_BOUNDARY` and `UNSUPPORTED_SYNTAX`. It
  accepts complete raw bytes, applies the 4,096-byte core policy, and constructs
  the fixed five-clause, thirteen-token representation internally.
- `crates/uor-r4-core/src/learned_reference/loader.rs`, SHA256
  `208fbe0a2447a650063909cc623e9c9c3804a839a4c0ba99ab39c79e8b26ec07`,
  remains the artifact/binding validation and owned-state intake boundary.
- `crates/uor-r4-core/src/learned_reference/environment.rs`, SHA256
  `307341a98cb20719604681e528966c20d8b622f107d29939fae6f4f02ff07bf0`,
  remains the actual arithmetic-thread floating-point environment check for
  the selected native profile.
- `docs/r4_native_bridge_1102_evidence/qualification-handoff.json`, SHA256
  `efc02551d493e255f12680ccf2e4ee99cca5f645e0ca3d7fcd6445419e963426`,
  distinguishes the accepted artifact/export binding from the measured #1102
  binary and explicitly forbids a newly linked host from borrowing that binary's
  qualification.
- `docs/r4_native_bridge_1102_evidence/qualification.json`, SHA256
  `61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369`,
  is the retained canonical twelve-field receipt for the historical executable.
  It is cited as history and is never assigned to the proposed executable.

The source manifest also pins the inspected donor UI, NEMESIS, W33 and UOR
materials. Their use is appropriately limited. The donor supplies presentation
patterns and an MIT provenance record; its model aliases, fallback, cache and
stop behavior are not adopted as evidence. NEMESIS contributes state/transition
questions but does not prove this service or its computational claims. W33
contributes immutable-object and receipt patterns without establishing durable
storage or authenticity. The UOR sources support typed content, realization and
derivation identity distinctions; digest equality is not treated as behavioral
or mathematical proof.

## Accepted interface and ownership decision

The selected boundary is implementable from the current API without changing
the numerical reader or core:

1. One opt-in `r4-workbench` executable owns loopback HTTP, same-origin assets,
   configuration, model/job state and the sole worker process. The same exact
   executable runs its private worker mode with pipes and no listening socket.
2. The worker owns the loaded immutable artifact and qualified
   `LoadedResearchReference`. Public model work follows only
   `load -> qualify -> answer`; `compare()` has no public HTTP or normal-worker
   IPC route.
3. The public operation is exactly `answer_four_fact_raw_text/v1`. Canonical
   padded base64 transports decoded raw bytes unchanged into `RawRequest`.
   The host does not trim, normalize, segment, join history or synthesize
   context. Inputs through 8,192 decoded bytes may reach the original 4,096-byte
   core policy; larger transport inputs receive the separate `413` error.
4. `JobSnapshot.result` carries the original `ModelToken` or `Refusal` value.
   A refusal is a completed supported operation, while native/loader/process
   failures remain typed errors. Published model-token policy, reader, core and
   frame identities must match the verified artifact, and raw-input hashes use
   decoded bytes.
5. The configured artifact identity is distinguished from worker-verified
   intake. A job's artifact is its configured admission identity;
   `ModelSnapshot.verified_artifact` and accepted `WorkerReady` establish the
   current worker's verified copy.

This preserves the accepted reader/core, vocabulary, query form and four-fact
context. It does not advertise chat, general generation, larger context,
reasoning, coding, retrieval, tools, a browser teacher fallback, or the final
integer/table kernel.

## Accepted lifecycle and cancellation semantics

The final contract gives public model generation and private worker generation
different meanings. Model generation advances on verified readiness and once
per spawned-worker invalidation; it does not advance again on reap. Worker
generation advances for each spawned child and appears on every private message.
Stale instance, model, job or worker messages cannot complete later work.

The one-slot state machine now closes the important race cases:

- Admission atomically binds the active job and model state. Load, answer and
  unload jobs progress through their corresponding nonterminal states; there is
  no waiting queue. A `202` returns the latest state reached, including a
  terminal snapshot if completion preceded response delivery.
- Result/cancel ordering has one serialized winner. If a result commits first,
  cancellation receives `ALREADY_TERMINAL`. If a stop with a spawned child wins,
  late output is discarded and `cancelled` or `failed` is not committed until
  that owned child is confirmed reaped.
- An accepted load stopped before child creation has no fictitious process to
  reap. Cancellation becomes `cancelled/unloaded`; a load deadline becomes
  `failed/error`; generation remains unchanged and work is zero.
- A cancelled or failed spawned worker invalidates model generation exactly
  once. Unload invalidates at admission and never increments again while
  stopping or reaping.
- Unload succeeds only after a valid `unloaded` reply and confirmed reap before
  its deadline. A winning unload deadline or missing reply follows the explicit
  failure path.
- Idle worker failure is a model-only transition. It cannot mutate the immutable
  terminal load or answer job. An unconfirmed reap blocks the slot and exposes
  `TERMINATION_UNCONFIRMED`; later resolution clears or restores error state
  according to the original stop reason.
- The model-level qualification digest means a receipt installed in the current
  worker. It is null before accepted readiness and after confirmed removal,
  while the host-level digest separately describes the adopted bundle.

The IPC contract permits one command at a time and exactly one command-specific
terminal reply, with only ordered optional progress beforehand. Native,
loader, numerical, protocol, crash and missing-reply failures preserve their
causal error, stop publication and require process cleanup. Worker stderr is
retained only through the declared cap, while the parent continues draining
excess bytes so diagnostic truncation cannot block computation.

These are architectural definitions. The current core has no cooperative
cancellation callback; the proposed process termination and reap behavior must
still be implemented and observed under its separately frozen integration gate.

## Accepted trust and empirical gates

The contract correctly separates three identities and decisions that cannot be
substituted for one another:

1. The artifact's original export-release digest remains immutable provenance
   required by the existing `compare()` check.
2. A fresh, independently accepted execution release must authorize any new
   host comparison. The exact final service executable must already include its
   non-listening private comparison mode in the source/build freeze. A different
   harness executable or a mode added after comparison cannot qualify it.
3. A new canonical twelve-field receipt may be created only after the new
   binary/runtime comparison result is retained and independently accepted.
   Ordinary serving then calls `qualify()` with that new receipt and uses
   `answer()`.

The operator evaluates the opaque result/review/release/runtime evidence before
adopting the `HostAcceptance` digest. At runtime, the service verifies exact
lengths and hashes transitively anchored by that adoption and parses only the
schemas frozen for runtime parsing: `HostAcceptance` and the canonical #1086
qualification. This avoids both trusting a self-asserted result and inventing an
unspecified semantic parser for future evidence records.

Numerical comparison and ordinary service acceptance remain different future
gates. A numerically qualified candidate may exercise the public path only under
its separately admitted integration check. Before ordinary use, an independent
behavior result/review must bind the exact contract, source/build, binary/runtime,
host acceptance, assets, inputs, release and outputs in the implementation issue,
delivery record and claim ledger. Product acceptance is external to runtime
startup and does not alter the twelve-field numerical receipt.

## Resolved review findings

The accepted bytes incorporate all material findings raised during independent
review:

- split idle worker failure from active-job failure so no completed job is
  rewritten;
- added the missing load-time failure/stopping path and removed overlapping
  spawned/reaped load transitions;
- made generation changes conditional on actual child creation and idempotent
  across stop/reap;
- distinguished configured job artifact identity from verified worker intake;
- required continued stderr draining after the retention cap;
- closed unload deadline, acknowledgment and reap outcomes;
- fixed decoded-input hash axis, load admitted/resulting generations and exact
  ordered result schemas;
- closed host-identity nullability and exact `WorkerReady` equality checks;
- declared product behavior acceptance as an external promotion gate;
- required the private comparison mode in the exact final binary before build;
- added numeric configuration, evidence, executable and asset intake caps;
- fixed operation availability and error mapping by model state;
- closed IPC command/reply order, native failure cleanup and causal-error
  retention;
- treated operator-adopted opaque evidence as a transitive trust decision rather
  than claiming an unspecified runtime evidence parser;
- removed the duplicate unload/unconfirmed transition;
- closed job-state, active-job and progress-stage relations; and
- defined current-worker qualification lifetime and model-token identity checks.

## Limits and closure

No service/worker crate or binary exists from this task. No Cargo build or test,
server/browser launch, artifact/model/corpus deserialization, successful
qualification call, fit, forward, export, replay, benchmark, sealed-input read or
dormant QA ran. The proposed platform mechanism that binds measured executable
bytes to the launched child is a future implementation requirement. The numeric
caps are specified policies, not measured resource use. The metadata scenarios
and wire objects are illustrative and were not sent to a service.

The mathematical status is unchanged: this review proves no neural, geometric,
identity or compression theorem. The measured status is also unchanged: #1102
remains `NATIVE_REFERENCE_PRESERVED` only for its exact artifact, executable,
runtime and known authoring stratum; ordinary `qualify()`/serving remains
`NOT_RUN`. #1094's unavailable preparation and bounded adapter result, #1079's
weak control and #1082's descriptive result remain recorded. #973 remains open
and #954 blocked.

#1105 may close as `SERVICE_API_CONTRACT_SPECIFIED` only after these exact
records land through the protected PR path. Parent #1084 must remain open. Its
one concrete next action is to implement the small host/worker/first-shell child,
including the private comparison entrypoint, then freeze and independently
review the exact source/build/runtime/comparison release before any build or
model work.
