# Native learned-reference bridge preparation — #1102

**2026-09-03 — `PREBUILD_DEPENDENCY_UNAVAILABLE`; draft source only.**
The opt-in native bridge and its comparison harness have been written and
independently inspected. The offline dependency metadata check failed before
the bounded native build began. No native binary, export, loader result,
comparison, replay or qualification exists. #1102 remains open and parked;
the draft PR references it and does not close it. No merge is approved.

The protected starting revision is
`93613bf82782ca78406fe2739dcc8d9e1d0f2b9e` (PR #1103). The source candidate is
`087303635b9674de02339bc057fca67e19bcd318`. The unchanged
[#1086 contract](r4_native_reference_1086_contract.json) has SHA256
`e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`.
The [prebuild record](r4_native_bridge_1102_prebuild.json) binds the candidate's
source files, observed dependency error, checks and zero-work accounting.

## Candidate source and independent review

The `learned-reference` feature in `uor-r4-core` and `uor-r4-api` exposes an
owned immutable 21-component research artifact, the strict raw-byte adapter,
trusted loader, bounded floating-point reference and a research-only stdin
harness. The source includes both explicit R4 transport stages, all fifteen soft
role mixtures, the tied full-4096 output head and the accepted core vocabulary.
The default R4G1 serving path is unchanged. This feature has **not compiled**.

The offline exporter decodes Safetensors directly without model constructors.
Separate source implements eleven rejected loader fixtures, one valid
zero-forward gate, the pinned B=1 Python reference worker, full-tensor
comparison/replay, and external phase/resource supervision. These are
implementations awaiting their admitted checks; the descriptions are not
observed runtime guarantees.

The [independent source review](r4_native_bridge_1102_code_review.md) and
[harness review](r4_native_bridge_1102_harness_review.md) preserve findings and
their resolutions. Corrected issues include historical frame-tree byte counts
and LF hashing; exact schema and error precedence; the over-limit refusal
transport; pre-forward quotas; and retained partial-load/attempted-forward
evidence on failures. Unknown counters remain unknown with bounds rather than
becoming zero. Review supports **draft preservation only**, without build,
execution or merge approval.

The [independent input audit](r4_native_bridge_1102_input_audit.md) rehashed the
original authoring files without parsing their rows. Raw input is 215,866 bytes
and reference annotation is 456,980 bytes; the combined 672,846 bytes remain
the fixed future corpus debit. It also inspected relevant original NEMESIS,
W33 and UOR source material at the accepted pins. Carrying criteria, immutable
content-store patterns and typed byte/value identities inform the implementation;
they supply no native numerical-preservation proof.

## Observed offline prerequisite failure

Both dependency metadata queries ended with exit 101:

```text
error: failed to download `aho-corasick v1.1.4`

Caused by:
  attempting to make an HTTP request, but --offline was specified
```

The lockfile requires archive SHA256
`ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301`.
The exact `.crate` archive was absent from the inspected local cache. The
extracted `aho-corasick-1.1.4` tree has a `.cargo-ok` marker but no corresponding
archive or `.cargo-checksum.json`; source presence does not verify the required
archive digest. No archive was fabricated, dependency substituted, pin changed
or network dependency download performed. The broader metadata inventory also
found other missing lockfile archives; it does not establish that all of them
are in this build's dependency closure.

This is preparation unavailability, not a timed build failure or a native
model mismatch. Rustc metadata identifies 1.97.1, target
`aarch64-apple-darwin`, LLVM 22.1.6. It does not establish a compiled native
binary or the executing libm feature closure.

## Work, evidence and preserved limits

Python AST parsing, formatting of the named Rust files, claim wording and diff
whitespace checks passed. These are syntax/documentation checks, not type
checking or behavior tests. No broad QA was activated.

Native builds, exports, loader calls, model deserializations/loads, forwards,
fitting, evaluations and replays are all **zero**. Both the native-build and
export/comparison envelopes remain **unconsumed**. No concrete execution release
has been admitted. The future 320-row/16-refusal comparison, full four-tensor
criterion, fresh replay and all resource limits remain exactly #1086's.

The original mixed checkout, model assets, historical evidence and sealed
withheld payloads are preserved. During local filename discovery, the sealed
withheld directory returned `Permission denied`; no payload was read and no
permissions changed. Storage review used the existing September 2 audit and
observed 44,421,256 KiB available at this checkpoint. No files were deleted and
no build output was produced. Retain this worktree because it owns unfinished
source and review evidence.

The accepted #1094 bounded Python result, #1079 weak-control result and #1082
descriptive limits remain unchanged. Native preservation is an **unverified
empirical hypothesis**. There is no new proof, general-English/context/coding
claim, final integer-kernel qualification or service/UI handoff. #973 stays open
and #954 blocked; #1084, #1083 and #1087 are not activated.

## Closure and one next action

This checkpoint is delivered as a draft PR through the protected PR path.
#1102 remains open; implementation qualification and protected merge are
unfinished. No successor issue or second task is activated.

**Next:** restore the exact Cargo.lock-matching archives to the local dependency
cache, then resume #1102's unconsumed offline build and independent concrete
source/binary/runtime/input/output release review. The frozen comparison still
must not begin before that release is accepted.

## Later checkpoint — native comparison completed

The cache restoration, one successful offline build, separately admitted
comparison and independent outcome acceptance are appended in
[r4_native_bridge_1102_execution.md](r4_native_bridge_1102_execution.md).
The later result is `NATIVE_REFERENCE_PRESERVED`, within its exact known-input
scope. This earlier preparation record and its failed offline queries remain
historical evidence. Both envelopes are now consumed; the current action is
protected PR #1104 delivery, then the separately activated #1084 interface ADR.
