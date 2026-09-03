# Independent implementation review — #1102

**2026-09-03 — `SOURCE_REVIEW_IN_PROGRESS_RELEASE_NOT_ADMITTED`.** This review
reads source only. It does not authorize the export/integrity, comparison or
replay envelope. No build, model import, asset/fixture deserialization, export,
loader invocation or numerical forward was performed by this reviewer.

The authority is the protected #1086 contract, SHA256
`e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`, its
[normative prose](r4_native_reference_1086.md),
[operator audit](r4_native_reference_1086_operator_audit.md), current
`AGENTS.md`, and the [independent input audit](r4_native_bridge_1102_input_audit.md).
The base revision is `93613bf82782ca78406fe2739dcc8d9e1d0f2b9e`. The reviewer did
not author the native implementation, adapter, exporter, fault fixtures or
execution harness and edits only this record. Findings and their resolutions
are appended rather than replaced.

## Source boundary inspected

The initial pass inspected the five core `learned_reference` modules, the API
feature and facade, the research stdin binary, the direct Safetensors exporter
and the independent eleven-case loader mutation generator. The original Python
adapter, frame loader, vocabulary writer and native H4 sidecar schema were read
to resolve exact field/byte and operator conventions. The current implementation
is still being assembled, so this pass deliberately records no final source
closure or binary acceptance.

The scalar numerical path retains the fourteen original parameter tensors,
tied full 4,096-entry output head, continuous token frame fold and both explicit
R4 transport stages. The reviewed loops use ascending contracted indices,
separate typed products/additions, source transpose for encoding, destination
transpose times source for connections, complete soft mixtures, and destination
decode. Query owner/object are concatenated before a single 128-wide layer
normalization. The full f64 binding dot is rounded to f32 before division by
eight. The fifteen role mixtures are computed, including the unused query
location. These are source observations, not numerical-preservation evidence.

The adapter's lexical IDs, accepted grammar, schema/limit/encoding/boundary/
syntax precedence, byte spans, padding and derived-input framing agree with the
original Python source in this static pass. Native MODEL_TOKEN spelling comes
from the core vocabulary. No role label or target is an argument to numerical
inference. The feature remains opt-in and does not wire default serving, HTTP,
browser or R4G1 paths.

The loader now includes the historical frame-tree record byte counts and the
trailing LF, compares qualification bytes to their canonical re-encoding to
reject duplicate/alternate JSON spellings, and uses the fixed contract component
name for a rejected altered descriptor. These three earlier issues were checked
as corrected. Trusted artifact/source/state/codec/frame bindings remain external
to a manifest's assertions. The tiny architecture-gated FPCR assembly reads only
the current thread register; it does not write modes, dereference pointers or use
the `pure` assembly option. The numerical call checks FPCR before and after an
evaluation, including an evaluation returning an error.

## Initial actionable findings

1. **Compile trait mismatch.** `ComparisonOutput` derives `Debug`, but its
   `Diagnostics` member initially derived only `Serialize`. Resolve this before
   the single timed build preparation.
2. **Exporter source closure.** Its bin discovery matches `native_reference` or
   `native_bridge`; the declared binary name `r4-learned-reference-1102.rs`
   matches neither. Require the exact executing binary source in the closure.
3. **Loader stage order.** A combined nonparameter dtype/shape loop validates
   frame descriptors before completing the earlier codec/policy stage. Complete
   components 14–15 and their codec validation before entering the frame stage.
4. **Frame component order.** Multiplication/token-leaf ranges are checked before
   the preceding frame component's finite/bit agreement, and leaves before
   multiplication agreement. Complete each component in fixed order, as the
   contract requires when more than one fault is present.
5. **Over-limit refusal transport.** The initial native hex decoder caps the raw
   buffer at 4,096 bytes, preventing the frozen 4,097-byte INPUT_LIMIT request
   from reaching the adapter. Admit the bounded refusal buffer in research IPC
   and retain the adapter's 4,096-byte rule.
6. **Rejected-load accounting.** The initial rejected gate output reports zero
   loads without the required component/stage and partial reader/core validation
   account, and lacks the final FPCR observation. Add concrete accounting and the
   failed-path mode observation; do not report partial validation as absent.
7. **Forward quota enforcement.** The initial native worker checks the 320-row
   forward ceiling after inference. A wrongly accepted refusal could therefore
   execute a 321st forward. Enforce the remaining allowance before arithmetic,
   while still allowing zero-forward parsing/refusal decisions.

These findings were sent to the implementer before any build or model work.
The eleven independent immutable mutations match their intended loader stage
and declared error object on source inspection; this statement does not claim
that the native loader has executed them successfully.

## Release boundary remains closed

A later entry must identify the final executing source closure, one bounded
build and actual binary, exact Python/native runtime identities, input/output
paths, independent access probes and the complete durable external supervisor.
It must reconcile all resolved findings with the actual code. The accepted
contract's one-attempt limits and terminal precedence remain unchanged. No
cross-runtime tolerance result, native capability or mathematical proof is
established by this source review.

## Reference-worker source pass

The subsequently added `r4_native_bridge_1102_reference.py` was read alongside
the accepted Python worker's `_configure_runtime`, `_verify_bindings`, `load`,
`states` and `_adapter_record`, and the original `frame_assignment`. This new
wrapper calls no historical CLI or consumed run command. It verifies source
bytes and all four denied-path sentinels before project/model imports; the
accepted runtime setup still requires Python 3.12.14, Torch 2.7.1, CPU Accelerate,
four intra-op threads, one inter-op thread and deterministic algorithms.

Its source enforces B=1, counts attempted forwards before inference, refuses a
321st forward, emits all four complete f32 tensor seams and all frame/role
diagnostics, checks positive-zero padding, preserves actual core-vocabulary
spelling and compares final reader/core states to initial states. Reference
errors retain exception details and attempted-work counters. There is no new
source blocker from this pass. The worker's base64 transport and the native
worker's hex transport must be derived from the identical original raw bytes by
the coordinator. Runtime-file identity, sandbox construction, external resource
accounting and the final immutable release remain pending; no worker was run.

## Resolution pass and remaining release work

The implementer's next source revision resolves findings 1–7: `Diagnostics`
derives `Debug`; the exporter discovers the `learned_reference` binary source;
codec and frame components are validated in their stage/component order;
research hex IPC admits up to 16 KiB while the adapter retains its 4,096-byte
limit; the loader now returns a separately collected `ValidationAudit` of actual
entered stages, component checks and reader/core partial validations; rejected
gate calls observe final FPCR; and the comparison path refuses arithmetic once
its supplied remaining-forward allowance is zero. These were checked by reading
the changed source, without invoking the loader or parser.

An additional failure-evidence finding remains for reconciliation: the native
worker originally counted only successful numerical returns and emitted no work
counters in its general error record. A numerical failure after arithmetic begins
must retain its attempted-forward count, successful/partial model-load state and
final FPCR. This belongs in separate harness audit evidence; the frozen three-field
`NativeError` object must remain unchanged.

The external supervisor initially applied the strict retained-evidence deletion
rule to Cargo outputs too. Its author has separated build accounting: observed
file generations retain their maximum byte debit while ordinary Cargo temporary
renames/removals are permitted; comparison evidence still rejects deletion,
replacement and shrinkage. Transient build-directory traversal races also need
to be handled without treating a legitimate compiler removal as a budget stop.
No previously observed bytes may be subtracted or preparation clock reset.

The supervisor's post-launch failure classification is consistent with the
frozen terminal table: an interrupted child or incomplete final receipt is an
`ABORTED_NATIVE_REFERENCE_BUDGET` stop even when the interruption is not a measured
time/byte/RSS excess. An orderly reported missing input/runtime/reference remains
`UNAVAILABLE_NATIVE_REFERENCE`. Both dispositions prohibit promotion and preserve
the consumed envelope. Final acceptance still awaits concrete source/binary,
runtime/access, path and byte-budget bindings.

## Preparation availability observation

Before the native build started, the implementer's offline dependency metadata
command reported that Cargo could not obtain pinned `aho-corasick` 1.1.4 from
the local archive cache. This reviewer independently inspected filesystem
metadata: the extracted source directory exists at
`/Users/casey.allard/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/aho-corasick-1.1.4`,
with `.cargo-ok` containing `{"v":1}`, but its matching registry `.crate`
archive and an extracted `.cargo-checksum.json` are absent. A bounded filename
search across `.cargo`, `Library/Caches` and `.codex` found no matching archive.
This is a local cache-availability observation, not an assertion about every
possible external backup. The frozen Cargo.lock archive SHA256 is
`ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301`.

No cache marker was fabricated, archive repacked, dependency resolution patched
or network dependency fetched by this reviewer. The extracted tree alone does
not establish a newly verified copy of that archive. The implementer is checking
the remaining pinned cache inventory before assigning the final preparation
disposition. A pre-build unavailability must remain distinct from a timed build
failure, export/loader failure or native numerical mismatch. No build or
comparison envelope has been consumed by this source review.

## Final draft disposition — preparation unavailable

**2026-09-03 — `PREBUILD_DEPENDENCY_UNAVAILABLE`; source review complete for
draft publication, execution release `NOT_ADMITTED`, native behavior `NOT_RUN`.**
This disposition supersedes the earlier in-progress review status. It accepts
preservation and review of the unfinished candidate in a draft PR only. It is
**not merge approval, build acceptance, export acceptance or native qualification**.

The final narrow source read confirms that the native harness now retains
`ValidationAudit` in its failure work record and sets the successful two-state
load count immediately after a successful audited load, before either gate or
scoring branches. Attempted forwards are incremented immediately before numerical
evaluation and retained if evaluation fails. The fallback error also preserves
the structured native error and final FPCR observation. Findings 1–7 and the
subsequent native failure-accounting finding are therefore resolved by source
changes in the inspected draft. The supervisor's build traversal now permits
ordinary disappearing Cargo temporary entries/directories while preserving every
previously observed generation's byte debit; comparison evidence remains strict.
These are source observations only. No loader, adapter, supervisor or reference
worker was invoked by this reviewer to validate them.

The concrete preparation blocker remains Cargo's unavailable pinned
`aho-corasick` 1.1.4 archive, lockfile SHA256
`ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301`.
The reviewed local source directory does not replace this missing archive in
the declared offline dependency path. The author's offline metadata observation
occurred before the native build, and no native binary exists for an independently
bound execution release. The source-only formatting, syntax and wording checks
reported by the author do not establish compilation, native loading, model
behavior, numerical preservation or runtime isolation.

Both the bounded native-build and export/comparison envelopes remain
**unconsumed** at this checkpoint. There are zero new exports, loader-gate
invocations, model deserializations, forwards, fits or replays. The intended
320-row/16-refusal comparison and all its frozen criteria remain unchanged and
unexecuted. The earlier #1094 Python qualification, #1079 weak-control result and
#1082 descriptive limits remain intact; #973 remains open and #954 blocked.

Keep #1102 open and parked with the draft PR referencing it. The remaining
implementation/export/qualification work must retain its issue home. The next
concrete action is to resolve the exact pinned offline dependency availability
and re-establish an independently reviewed preparation/release from this preserved
draft, before the sole build and model-work envelopes are admitted. This review
does not authorize a network fetch, dependency replacement, cache-metadata bypass,
new numerical profile, hidden retry or expanded comparison.
