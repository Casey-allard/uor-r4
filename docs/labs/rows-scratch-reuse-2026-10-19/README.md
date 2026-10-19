# Row counterfactual scratch reuse — source implementation

M1 tooling/driver optimization, Codex cf1. Owner authorized implementation in the Codex chat, 19 October UTC; public scope [6258074111](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6258074111). No model result, headline movement, training release or paid resource follows.

## Change

The opt-in `rows_v1` proposal path retains reusable target/base/cost/flag/status storage on each `Device`. An exclusive cache guard spans storage preparation, kernel submission, status readback and stream completion. Repeated calls within capacity reuse those five allocations; larger admitted batches replace storage. Active scratch extents are reset each call, including the status word. This caches storage, never model-dependent values.

The sixteen losses and five proposal words remain independently allocated and owned by each returned `Proposal`. Outstanding proposals therefore do not alias scratch from later calls. Existing module caching, admission and model/stream identity checks, CUDA kernels, chooser, objective and learning schedule are unchanged. The BVG scratch-copy constructor initializes its own empty cache; its block algorithm is unchanged.

A normal operation error is followed by stream completion before cached storage is discarded. A failed completion quarantines the cache and subsequent row proposals refuse it. A poisoned lock also refuses reuse. No automatic resource retry is introduced.

## Verification boundary

Six host-only lifecycle tests passed: repeated reuse, failed-completion quarantine, completion-before-error eviction, dual-error preservation, concurrency exclusion through completion, and panic-poison refusal. Baseline and final `cargo check --locked -p uor-r4-discrete --features cuda --all-targets` passed using the explicitly synthetic host compiler stub. Independent spec, Rust/code-quality and safety source reviews found no blocking defects. GPU resizing/reset equivalence and timing remain unexecuted. The stub produces synthetic compiler outputs exclusively for Rust type checking; it is not an executable CUDA image, runtime identity or device qualification.

**CUDA equivalence and timing: NOT_RUN.** Before runtime/performance claims, use the actual admitted finite GPU controls to verify losses, proposals, commits, model epoch/cursor, repeated and varying batch sizes, outstanding proposals and refusal/fault behavior against the unchanged path. This source delivery neither allocates a pod nor releases those controls. No speedup, SM utilization improvement or memory-leak fix is claimed.

## Coordination and successor

Claude owns the successor table constructor, attention plumbing and Stage-T block integration. The merged draft #2738 uses TRAIN-predecessor placement with all256cells×16 and P8/A8/S8/P16/A16 zero-update arms; the earlier Codex32-cell suggestion is not selected. AGY owns consumer work. Kimi owns FIT/dispatch coordination. The failed injective preflight remains `INSTRUMENT_UNREACHABLE`; its resource is closed and its owner commands remain withheld.

The next paid decision must bind the final sheet/source/table/configs, complete controls and diagnostics, resource rate/GPU count, build-through-archive duration/cost and outcome-dependent actions before rental. Scratch reuse is independent source work; it does not repair checkpoint-zero admission or authorize student training/donor generation. GPU cost of this implementation: $0; local compilation/storage are not claimed as free resources or invoices.

This normal delivery also removes two historical Codex claim entries for delivered #2653 and #2720. Their source/custody and claim-release preservation were already verified in their original closures; fresh remote/local branch and worktree checks confirm absence. Peer claims are unchanged. This is not a new cleanup campaign.

The repository harness check reported 0 errors, 128 warnings and one informational finding; D23/D25 ratchets and append-only shard integrity passed. Warnings include existing claim/branch housekeeping and a dated disk-floor observation (29.78 GB during the check); a subsequent `df` reported31.26GiB available. These observations use their original units and are not a standing resource admission. The reused host target grew approximately293MiB to562MiB; no additional build cache or pod was created. Fresh complete storage reserve is required before any future allocation.
