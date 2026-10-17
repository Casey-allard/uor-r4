# Preserve baseline-reuse resource refusals — 16 October 2026

**Later operational record, 17 October:** #2562 is delivered at95ee863c3;
the [first finite startup](../counterfactual-baseline-reuse-controls-2026-10-17/README.md)
stopped at driver admission before the instrument. Runtime parity/speedup remain
NOT_RUN; replacement readiness and a new release are required.

**Pre-lease source repair; no new model execution.** The independent readiness
review after [#2561](https://github.com/UOR-Foundation/uor-r4/pull/2561) found
that `BaselineReuseProfiler::resources()` discarded its queried values when
rejecting a runtime/AOT shared-memory mismatch. The finite runner persisted the
error, but its generic text could not identify the deciding values. This was
found by source review before a finite lease, not by a failed GPU observation.
[Commission6226887143](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6226887143)
limits the repair to that existing failure path.

The refusal now includes the complete already-queried `KernelResources`, the
compiler's reported shared count and the required512-byte count. The existing
blocked record retains this error. Query order, loader, predicate and return
behavior are unchanged. There is no second probe, new kernel, fixture, model
dose, host reference or relaxed admission. Query failures still propagate their
CUDA errors; no value is invented for a query that did not complete.

Construction #2561 established R PTX/ptxas application shared512,115 registers,
21536 stack bytes and zero spills. Raw ELF SHARED1536/reserved-cap1024 remains
separate evidence; reservation is an interpretation, not a measured runtime
subtraction. Runtime shared==512 and own-AOT register/local equality remain
NOT_RUN and fail closed. The repair makes a future refusal diagnosable.

The same readiness review also found that the external finite wrapper recorded
but did not prospectively admit its driver version. Before any finite release,
its replacement must enforce an independently admitted driver or receive an
explicit bounded policy. This is an operator-packet correction, not a reason to
expand the source patch or run a probe. The previous draft pin is unreleased
and superseded; re-pin the newly constructed executable after protected delivery.

## Verification and delivery

Scoped Rust formatting, claim wording and diff checks cover this preparation.
Package-wide formatting also reports existing differences in unchanged
`relation_qualification.rs`; that file is preserved. The scoped check uses pinned rustfmt1.97.1 directly.
An independent source review and actual exact-head CUDA check/strict standalone
Clippy are required before queue. The unchanged
[construction recipe](../counterfactual-baseline-reuse-implementation-2026-10-16/construct.sh)
retains source, executable, emitted images, resources and command exits. Final
exact-head construction/review/cost receipts belong on the repair PR before
queue; at this source freeze they are pending. No host test duplicates the
format string, and no CPU model/reference computation is introduced.

Previous source delivery #2561 is verified on fresh main6209e11ae6b534926009a3b1f783a73cff4063a8,
full tree53b748e9f4396ef9df74106f8743ccf8fe348c73 equal to approved d004f4ce5.
Its construction archive is `icloud:UOR-R4/results/codex/gate2561-r3-archive.tar`,
25275904 bytes,MD5 `55402801ec94fa8e62ac0f2064b8e145`; supplementary delivery
archive521216 bytes,MD5 `b61d383fd42e610520df0db16847cc35`. Cloud readbacks passed
before teardown/transport cleanup. The construction pod is deleted and fleet0.

Line: M1 qualification / cost tooling. Count0/3 and suspended headline37/232
are unchanged. This pre-lease finding adds no GPU charge; prior cumulative
32-lease Codex estimate$3.270958333 plus separate PM, storage/transfer USD
UNMEASURED. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE. Next: independent source
and construction acceptance, protected delivery, then a newly reviewed finite
packet and distinct release. Timing and all training remain unreleased.
