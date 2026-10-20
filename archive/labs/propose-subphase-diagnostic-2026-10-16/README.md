# Proposal sub-phase diagnostic — source-only instrumentation

Codex/Sol implements the [FIT-design pre-registration](../../harness/prep/m1-propose-subphase-diagnostic-2026-10-16.md)
under [owner instrument+optimize-first](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6210107580).
The later [sealed diagnostic](../propose-subphase-diagnostic-run-2026-10-16/README.md)
completed under PM's separate release after #2541/#2542 merged. All four cells
are counterfactual-dominated; the source-delivery details below remain historical
scope, not a repeat authorization. Qualification/training/M2/Stage B remain HELD;
Q4 NOT_EXECUTABLE. Line M1 qualification instrumentation, count 0/3 unchanged,
suspended headline 37/232 unchanged. No learned-capability or qualification claim.

## Delivered boundary

- `Device::propose` and `propose_profiled` share the same private implementation.
  The ordinary path creates no events or clocks. Profiling adds four precreated
  same-stream event records and disjoint host clocks, without changing model
  kernels, candidate order, status refusal or existing synchronization.
- `proposal_timing.rs` contains clock metadata and integer IEEE754-bit decoding,
  not a CPU model reference. Invalid durations and incomplete/non-partitioning
  host spans are refused. Elapsed-event retrieval is after existing completion
  and outside the original proposal timer; its own cost is separately recorded.
- The timing runner's separate `diagnose` command/pin/record schemas enforce
  checkpoint0, A/R, one excluded warmup and three timed batches. Original
  full-unit timing remains. No diagnostic headline training rate is emitted;
  all durations are coarse diagnostic evidence, never qualification or p95.
- Host regressions exercise diagnostic count/scope/identity/config refusals,
  distinct receipt schema and clock-bit/partition arithmetic. Existing metadata
  tests remain. No host model execution or GPU fixture is admitted here.

## Identity blocker preserved

Production source hashing includes `device.rs`, `lib.rs` and the added timing
module. Thus the original sealed checkpoint cannot load under this new source.
The [pin template](diagnostic-pin.json) now declares the exact original/new
source mapping while retaining original PTX/config/checkpoint-file pins.
The [in-memory bridge](../checkpoint0-identity-bridge-2026-10-16/README.md) must
be separately reviewed and merged before execution. No source hash is
silently rewritten; `Device::from_snapshot` is unchanged. The evaluation-only
checkpoint-0 identity bridge remains a mandatory execution dependency,
as reported to PM in [6210356978](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6210356978).

## Verification and delivery

Host validation: timing instrument 23/23 tests and strict all-target clippy pass;
production library 38 tests pass, one pre-existing ignored test. These are
metadata/contract checks, not GPU construction or model controls. Host-only
dependency dead-code warnings remain visible; no warnings are suppressed by
this change. The pure-clock tests include finite-bit decoding, invalid/overflow
refusal and exact host partition accounting.

Host and CUDA gate results are reported at the exact PR head; CUDA activity is
compile/lint only. DeepSeek holds exact-head source review; Kimi plus an independent
reviewer holds FIT. Neither authorship nor compilation is self-approval or
diagnostic execution permission. Full CUDA transcripts, archive MD5 and actual
lease cost accompany the PR comment before queueing. The next step is review
and identity-bridge disposition, not a model run or kernel optimization.

The measured predecessor remains the [sealed restart-5 record](../k16-qualification-restart5-2026-10-16/README.md).
Its roughly 14.5 seconds/batch is not a qualified training rate. The sheet's
~$0.027 estimate covers only sixteen proposal calls, not all-in lease cost.
