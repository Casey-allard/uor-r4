# Counterfactual compiled-image/resource binding — 16 October 2026

**Execution follow-up:** after protected #2556 delivery and distinct release,
the [Stage1 receipt](../counterfactual-controls-pass-2026-10-16/README.md) records
runtime/AOT equality and the complete40/128/16 finite-control pass, apply0.
The construction-era NOT_RUN boundary below is dated provenance; Stage2
remains separately gated and NOT_RELEASED.

**Source repair; runtime parity NOT_RUN; execution NOT_RELEASED.** The
[sealed Stage1 attempt](../counterfactual-controls-run-2026-10-16/README.md)
stopped at resource admission: runtime/AOT registers U90/88 and D122/126,
matching local bytes, zero AOT spills. GPU initialization and snapshot
validation ran, but the finite-control dose was zero. That is an instrument
provenance blocker, not a mechanism negative. This repair preserves its
zero-delta admission rule.

The old loader supplied PTX to the driver while the resource report described
a separately compiled cubin. The new construction chain is:

`unchanged CUDA → emitted and admitted PTX → cubin assembly → embedded cubin`

The ptxas report comes from that same assembly. All three ordinary/diagnostic
loading sites pass those exact embedded cubin bytes through the existing
vendored `Ptx::from_binary` driver API. There is no PTX fallback. The actual
device capability must match the compiled real target before initialization.
The build checks nonempty ELF framing; this is not a complete CUDA-image
validator. The construction gate must inspect CUDA architecture, entries and
resources on those same objects, and the driver must accept the actual image.

Independent adversarial design review required assembly from the **admitted
PTX**, correcting the initial proposal to reuse separately compiled cubins.
The [source work card](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224975357)
records that correction and the bounded decisions. Codex owns image-construction
hunks below the build scanner; AGY retains its scanner/interlock work.

## Identity and unchanged computation

`compiled_identity.cuda_images` uses `uor-r4.cuda-images/1`, format `cubin`,
explicit real target, both cubin SHA256 hashes and both raw report hashes.
It hashes the same constants loaded by the driver and parsed for resource
admission. The composite source identity now frames target, images and reports.
This is a **build-specific executable/report binding**: raw report contents can
vary with construction paths/toolchains, so same-source rebuild identity is
not promised. The independent executable checksum still applies.

The existing `kernel_sha256` and `decomposition_kernel_sha256` fields keep
their PTX-hash meaning. Controls and decomposition pins advance to version2
and require exact image bindings before creating a CUDA context. Only a
nonexecutable controls source template may carry `cuda_images: null`; its
execution admission still refuses. Old pins refuse the new executable.

Snapshot schema and payload stay unchanged. The active evaluation-only
checkpoint0 bridge changes only `source_sha256` in memory, retains the exact
original artifact/config/PTX/geometry checks and unchanged-payload hash, and
records the image binding in its version2 receipt. It writes no rebound
checkpoint and permits no training resume. The older frozen
`checkpoint_bridge.rs` target remains unchanged/refusing.

No changes to mathematical CUDA, generated diagnostic source, geometry,
entry/argument ABI, launch shape, resource thresholds or fixture predictions
are made. The new AOT counts need not equal the historical88/126 counts;
those are observations, not compiler tuning targets. Runtime/AOT counts must
still match exactly, with zero AOT spills and the existing shared/local/block
allowances. An unexpected count or target is a blocker, never a waiver.

## Validation and prospective execution boundary

Host metadata tests, exact-head source review and CUDA construction gates are
tracked in [validation.md](validation.md). They establish no runtime parity.
No CUDA context or model/reference computation is permitted by a construction
gate. The gate must retain exact admitted PTX, cubins, resource reports,
generated source, target, toolchain and binary identities; archived hashes
must equal the executable's no-context identity output. `cuobjdump` examines
the same archived cubins for CUDA target, entry inventory and resources.

Only independent review, protected delivery, a new executable-bound pin and
a **separate bounded execution release** can authorize the unchanged Stage1
inventory:40 U/D proposals,128 one-row endpoints and16 typed admission
refusals, seed1, epoch0 and apply0. The old release was consumed by the sealed
attempt. A same-object resource mismatch remains BLOCKED and is preserved.
A full finite-control pass may support consideration of the separate
32-proposal diagnostic; it does not automatically release it. Stage2 stays
NOT_RELEASED, including the six original FIT conditions and provenance
addendum6222721133.

Training/M2/Stage B remain HELD; Q4 NOT_EXECUTABLE. This is M1 qualification
repair, count0/3 and suspended headline37/232 unchanged. The next mathematical
optimization still requires the bounded decomposition evidence; this source
change makes no speedup, language or training claim. It supports the owner's
same-native-math/minimal-surface direction without adding a model mechanism.

Prior Codex26-lease estimate is $3.009592222 plus the separate PM cost;
storage/transfer USD remains UNMEASURED. New construction cost is recorded
with its actual lease in the validation record. Any lease keeps the existing
fleet≤4,≤$8/h,30GiB-floor and verified-archive-before-teardown rules.
