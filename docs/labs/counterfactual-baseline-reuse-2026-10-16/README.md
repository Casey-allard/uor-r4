# Exact counterfactual baseline reuse — design, 16 October 2026

**Docs-only pre-registration; TEST FITNESS FIT for design only; implementation and new
measurements NOT_RUN.** The [sheet](../../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md)
turns the protected [decomposition result](../counterfactual-decomposition-run-2026-10-16/README.md)
into one bounded native CUDA change: compute unchanged baseline eligibility
once per row and reuse it across sixteen virtual candidates in the same call.

The proposed opt-in kernel keeps grid1/block16, the geometric read/writer,
virtual computation, ordered checked sums and chooser. Sixteen lanes produce
up to64 baseline row records in512 bytes of call-local shared memory, then
synchronize once. Consumers retain the original row order and early exits;
unvisited rows cannot publish errors early. Gate/Flock/Relation, member
indexing, persistent caches, existing allocations and training are outside scope.

The design fixes64 finite proposal attempts plus16 admission refusals, then a
separately released32-proposal matched timing dose only after finite acceptance.
Every loss/status/proposal word and state hash must match. Full call and whole-unit
cost include the new audit work; fewer evaluations do not imply a speedup because
SIMT scheduling/locality also change. All four policy/operation cells must clear
a prospective range/spread bar before a later100-batch qualification is proposed.

Authority is the owner's delegated operational PM/self-direction and same-native-
math goal, recorded on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224381783).
Source basis is fresh main `ddb081da8f5d6902fb067192c006059db783c45f`, protected
#2559. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; apply0 required. M1 tooling
count0/3 and suspended headline37/232 unchanged. No language, energy, trained
quality, speedup or fresh-transfer result follows from this design.

This unit costs zero leases / $0 new GPU spend. Inherited29-lease Codex estimate
$3.174466388 plus separate PM receipts stays unchanged; storage/transfer USD
UNMEASURED. Independent [design FIT6226136376](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6226136376)
binds prospective head `925422da5`; source and execution approvals are separate.
**Next:** protected delivery and cleanup,
then the narrow source implementation and its independent construction review.
Finite controls and timing require distinct later execution releases.
