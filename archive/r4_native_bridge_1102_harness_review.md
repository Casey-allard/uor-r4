# Independent coordinator and supervisor source review — #1102

**2026-09-03 — `DRAFT_HARNESS_SOURCE_REVIEWED_NOT_ADMITTED`.** This record
supports preserving the implementation as a draft. It does not admit a build,
export, loader invocation, reference/native comparison or replay. Issue #1102
remains open, and native behavior remains `NOT_RUN`.

The authority is the frozen [#1086 machine contract](r4_native_reference_1086_contract.json),
SHA256 `e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`,
its [normative prose](r4_native_reference_1086.md), and current `AGENTS.md`.
The base revision is `93613bf82782ca78406fe2739dcc8d9e1d0f2b9e`.

The reviewer did not author the coordinator or supervisor and made no code
changes during this review. The reviewer authored the separate Python reference
wrapper; this record is therefore independent review of the two files below,
not independent acceptance of that wrapper. Its separate review is recorded in
the [implementation review](r4_native_bridge_1102_code_review.md).

| Source inspected | SHA256 at this review checkpoint |
| --- | --- |
| `scripts/r4_native_bridge_1102_run.py` | `0757c94ff0769cb0e40cd5085aa4d27c17b8182f8f68fa7d4a8f121234c98765` |
| `scripts/r4_native_bridge_1102_supervisor.py` | `4053b57c87a9d12a3e313733c54346263d2184d41a83b4892a85198570fcd2a2` |

These are source identities, not executing binary identities. Subsequent source
changes require reconciliation before any independent execution release.

## Boundary observed in source

Only the coordinator reads the authoring annotations. Each worker receives the
same original decoded bytes, the original request schema and request extras.
Python transport uses base64 and native transport uses hex. Row identity and
annotation fields are added to coordinator evidence after the worker responds;
they are not sent as model arguments. The original curator source confirms the
frozen file order: 320 valid rows followed by 16 refusal rows. No authoring or
withheld payload was opened to make this observation.

Both arms are launched one at a time through their declared sandbox profiles.
The coordinator requires a ready record, one response per request, exact final
320/16 counts, two model loads, zero parameter updates, successful process exit
and no trailing output before considering an arm complete. The actual sandbox
profiles, denial behavior, interpreter and binary are still unverified release
inputs; their source-level presence is not isolation evidence.

The comparison retains all four complete f32 tensor seams and checks their byte
lengths and finiteness before using them. It computes per-row, per-tensor maximum
absolute error and its flattened location against the fixed `1e-5` threshold,
with no relative tolerance. It compares original result and parsed fields,
frame indices and the fourteen consumed role pointers; the unused fifteenth
role mixture remains included in the full tensor comparison. Padding attention
is checked against exact positive-zero bytes. Answer and consumed-role floors
are fixed at 320 and 4,480. Each incorrect answer or consumed role now records
its row and expected/actual values.

Fresh-process replay compares exact result/tensor bytes and diagnostics within
each implementation, excluding only phase and resource measurements from final
receipts. The coordinator requires valid reference floors and reference replay
before a candidate preservation verdict. An orderly reference failure remains
`UNAVAILABLE_NATIVE_REFERENCE`. A valid reproducing reference with candidate
failures cannot be promoted. An explicit budget stop takes precedence and
prevents subsequent work.

All eleven independent loader mutations are frozen with their complete bytes,
expected bindings, digests and exact error objects before the first loader call.
The coordinator checks zero forwards and zero successful model loads for each
rejected call. It reuses the twelfth, valid gate engine for the missing-
qualification probe and decoded-state evidence before scoring engines launch.
No loader result has been observed by this reviewer.

The external supervisor uses an exclusive consumed-envelope marker, monotonic
phase clocks that cannot be reset by repeated events, one execution/replay
transition each, and retained start/progress/stop/completion records. It includes
child output, flushes, exit and final receipt writes in its checks. An explicit
worker budget stop becomes an external stop; stop records override completion.
RSS covers the coordinator's process group and discovered descendants, including
workers surviving coordinator exit. RSS and filesystem observations are sampled;
they are not a mathematical resource bound or an OS write-isolation mechanism.

## Findings resolved before this checkpoint

1. Large complete mismatch details formerly exceeded the supervisor's 8,192-byte
   event limit and could change the terminal into a protocol stop. Full details
   now stay in `coordinator-result.json`; the completion event contains only its
   compact path, byte count and digest.
2. Native budget-error propagation formerly allowed replay to continue. Both
   native phases now return the budget terminal immediately, and the supervisor
   converts that terminal into its durable stop path.
3. The native error protocol formerly omitted attempted work. Its separate
   harness error record now carries attempted numerical forwards and model-load
   progress without changing the frozen three-field `NativeError`. The normal
   coordinator summary derives successful loads from worker evidence rather
   than unconditionally reporting five engines and ten model states.
4. The build ledger formerly rejected ordinary Cargo temporary-file removal or
   replacement. It now retains charges for observed file generations while
   allowing build cleanup. It tolerates `FileNotFoundError` caused by mutable
   temporary-file/directory traversal races. Comparison evidence still rejects
   deletion, replacement and shrinkage, and earlier charges are not subtracted.
5. An invalid worker event formerly disappeared during coordinator validation.
   The last bounded raw event is now preserved as `failure-event.bin` before
   structured error evidence is written. This also retains tensor hex bytes
   already emitted in an offending event without duplicating every successful
   event's full tensor storage.
6. Rejected gate responses now explicitly require zero forwards and zero
   successful model loads. Per-row incorrect answer and consumed-role evidence
   supplements the aggregate reference floors.

## Remaining limits and pre-admission work

The draft still needs conservative accounting for a coordinator-side validation
failure without a worker error counter. At this checkpoint, the aggregate
`attempted_forwards` fallback can report zero despite earlier completed rows or
an offending response proving nonzero work. Such a failure must retain a known
lower bound and an unknown/explicit upper bound rather than assert zero. Also,
the successful-engine aggregate must distinguish a validated ready record from
a non-null ready object that failed validation. These are source-level failure
accounting follow-ups; no affected run occurred.

The implementation review records a separate preparation-availability finding:
offline Cargo dependency metadata could not obtain the pinned `aho-corasick`
1.1.4 archive, and the extracted tree lacks a checksum-verified source inventory.
That observation belongs to its named reviewer and the implementer's retained
command evidence; this harness reviewer did not run Cargo or inspect dependency
payloads. A draft must not describe this as a failed timed build or as a native
numerical result. Restoring the exact pinned dependency cache, reconciling source
findings, producing the bounded build, and independently reviewing the concrete
source/binary/runtime/input/output/access bindings remain necessary before an
execution release.

No script was executed or imported during this review. Build, model imports,
asset or fixture deserialization, exports, loader calls, numerical forwards,
fits, parameter updates, evaluation, replay and withheld reads performed by this
reviewer are all zero. No new mathematical proof or measured native behavior is
claimed. The earlier negative/control results and the frozen comparison contract
remain unchanged. Preserve this work through a draft protected PR with
`References #1102`; do not close #1102 or activate a successor from this record.

## Failure-accounting resolution — same-day source follow-up

The two source findings listed above under remaining pre-admission work are
resolved in coordinator SHA256
`5b3542f774ff014757ce8dbdc591dc15a7843b2cb53f8a0dab028ead0d0083e9`.
The supervisor remains at SHA256
`4053b57c87a9d12a3e313733c54346263d2184d41a83b4892a85198570fcd2a2`.
This entry supersedes their outstanding status while preserving the preceding
review checkpoint.

`arm_run` now stores a response in `candidate_ready` and assigns the accepted
`ready` value only after its event kind, two-model-load count and zero-forward
count pass validation. The successful-engine aggregate therefore no longer
counts a rejected ready object.

`attempted_work` now reports each arm separately. It uses a complete done or
worker-error counter when available; otherwise `exact` is null and
`unavailable_counter` is true. Successfully retained result rows establish a
known lower bound, while 320 is explicitly labeled the admitted upper bound.
An absent counter is no longer presented as zero attempted work. The final
summary retains these records under `attempted_forward_counts`.

These corrections were verified by reading the current source and hashing the
two files. No execution or new model evidence was produced. No unresolved
coordinator/supervisor source finding from this review blocks preserving the
draft. The disposition remains `DRAFT_HARNESS_SOURCE_REVIEWED_NOT_ADMITTED`:
dependency availability, the bounded build, concrete runtime/access bindings,
independent execution release and the frozen empirical comparison remain open.
Issue #1102 must remain open.
