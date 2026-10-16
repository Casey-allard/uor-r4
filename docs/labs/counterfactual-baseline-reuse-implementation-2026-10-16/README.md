# Exact GPU baseline reuse — implementation, 16 October 2026

**Source preparation; runtime parity and speedup NOT_RUN.** This implements the
[FIT design](../../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md)
delivered in #2560, under [source commission6226210469](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6226210469).
Independent exact-head source and CUDA construction review precede protected
delivery. Neither source delivery nor construction releases model execution.
Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; apply remains outside this slice.

## Change and mathematical scope

The separate opt-in R entry computes the unchanged native baseline read/writer
once for each admitted row and stores its status and eligibility in 512 bytes of
call-local shared memory. Sixteen physical lanes produce `row = lane + 16*k`,
reach one unconditional barrier, then consume rows in the original order.
Producer faults become visible only when a consumer reaches that row. The
virtual read/writer, checked ordered sum, rival-removal MAX sentinel and chooser
are unchanged. There is no identity shortcut, persistent cache, candidate
pruning, member index, allocation pooling or different launch shape.

Ordinary `discrete.cu` is unchanged, SHA256
`f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008`.
The new module copies its helpers/writer and uses fail-closed source anchors for
the control-flow change. U's admitted PTX and cubin must remain byte-identical to
the [sealed comparator](../counterfactual-decomposition-run-2026-10-16/README.md),
respectively `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28`
and `1648564346740520c125fcfb5bc407cbbca40e66e95e2e3ad89b526a321ffcee`.
Construction must establish those identities before any model dose.

`Device::propose_reused` and `BaselineReuseProfiler` are explicit opt-in APIs.
Only Route read with Route/VectorBlock parameters is admitted; existing
Gate/Flock/Relation paths and default training dispatch remain unchanged.
Every call revalidates model/stream, batch, targets, parameter and epoch.
The precreated 1,280-byte audit is reset inside the measured call/event and records
all row producer outcomes plus six words per lane. Producer partitions/counts
are separate from consumer visited/eligible/virtual masks and terminal status.
Every terminal lane record is written even on fault. The same R image and stores
serve controls and timing.

R source/PTX/cubin/report/target have a separate exact image binding; the existing
U/D identity fields retain their meaning. The build admits R's one-entry integer
PTX before assembling its cubin; the loader consumes that cubin. The new resource
rule is U static shared0/R512, dynamic0, each runtime matching its own AOT
register/local report, zero spill loads/stores and block16 launchability.
No old U/D register or local-memory delta cap is imported. Runtime resource
equality remains NOT_RUN until separately released finite execution.

## Fixed instruments and admission

The existing k16 standalone instrument gains `reuse-template`, `reuse-plan`,
`reuse-check`, `reuse-identity`, `reuse-controls` and `reuse-timing` commands.
Templates are explicitly nonexecutable. Execution pins bind the exact source,
binary, fixture inventory, geometry and all three images; FIT and execution
references must differ. Timing additionally requires the accepted finite record
hash and a separate acceptance/release reference. URLs are externally reviewed
authority references, not authority minted by the executable.

Finite execution is fixed at 64 proposals:56 successes/eight expected status8,
plus16 exact typed admission refusals. It reuses the six literal fixtures and
adds repeated eligible rows of16,17,64. All sixteen losses, status, five proposal
words, audit masks and before/after state must agree. The virtual-ineligible
fixture distinguishes full producer coverage from early consumer termination.
The128 prior immutable endpoints remain sealed evidence and are not rerun.
CUDA2/4/9/12 reachability and isolated stale-epoch limitations keep the FIT scope;
no corrupt ABI or apply is introduced to manufacture a fault.

Timing is a distinct future release: original checkpoint0, seed1/epoch0,
V4096/B4/C192/K16, rows64, both policies and Route0/VectorBlock0. It has exactly32
proposals, eight excluded warmups and24 timed, twelve pairs. Loop order is
policy/batch/mode/operation, with U→R, R→U, U→R for timed batches and Route before
VectorBlock. Each operation recarries identical windows so each cell has a
complete carry/propose/full observation/remainder account. R audit readback is
included. Metadata checking and record persistence occur after that whole-unit
timer. No sample is discarded or added automatically.

Each cell must satisfy `min(U)-max(R) > max(range(U),range(R))` for event and
propose wall, and `max(R)-min(U) <= spread` for whole-unit cost. Every pair,
signed difference and ratio is retained. Parity/resource/accounting faults are
BLOCKED; mixed/noisy observations are INCONCLUSIVE; a regression concerns this
configuration. No learning KEEP, activation, training rate, p95, language or
energy claim follows. SIMT scheduling/locality can offset saved evaluations;
the sealed 46.92–47.22% baseline share is not a promised speedup.

The checkpoint bridge is evaluation-only and pins the original artifact/config,
ordinary image and new compiled source. It changes only source identity in
memory, records the unchanged payload digest and preserves original file bytes.
Production snapshot admission remains strict; no rebound checkpoint is saved.

## Verification and delivery state

Host checks at the implementation draft: standalone instrument tests **47 pass,
one intentionally ignored archived-byte test**; focused crate baseline-reuse
metadata/source tests **3 pass**; standalone strict Clippy **PASS**. Existing
dependency dead-code warnings are disclosed; they are not CUDA evidence. An
initial workspace test command lacked sparse `xtask` metadata, then passed
after materializing `xtask` and `tools/lab-runner`; this was a checkout setup
failure, not model evidence. The independent source review's identical FIT/
release-reference gap is repaired and covered by a host rejection control.

The merged #2558 scanner now admits generated R through its D32 token check and
exact one-entry census, alongside the preserved eleven U and one D entries and
five instrument roots. Its std-only self-test and repository scan pass, including
injected floating-literal, extra-entry and renamed-entry refusal controls.

Actual CUDA check/Clippy, ordinary image-byte comparison and construction
resources are **PENDING at this source freeze**. Exact-head construction,
review, cost and delivery receipts are recorded on [source PR #2561](https://github.com/UOR-Foundation/uor-r4/pull/2561)
before queue admission. [Construction recipe](construct.sh) runs only builds,
image inspection and host identity/template/plan commands. A separate reviewed
construction packet/admission is required before rental. No finite or timing
execution is included. Final protected delivery must bind exact source and
construction receipts and preserve the concurrent AGY interlock work.

The prior #2560 claim release is carried by keyed deletion; peer claims are
preserved. Post-#2560 hygiene finished exit0 with31GiB reported free. During local
source preparation,20 duplicated extracted input files were byte-compared against
retained tar originals and only those copies removed (~312MiB); unique evidence
was preserved. Lightweight host builds retain the30GiB floor.

Two construction attempts stopped before CUDA compilation. Attempt r1's provider
instance exited before setup; SSH refused connections after its initial readiness
check. Attempt r2 completed setup, then the clean-source preflight rejected its
own receipt files. Both failures are retained tooling evidence, with no model
result. The recipe repair preserves exact-head, tracked-tree and index checks,
allows only this session's output and precisely named runner/reaper receipts,
and rejects all other untracked files. Host Git fixtures exercise these boundaries.

Both archives passed SHA checks and full cloud MD5 readback before teardown:
`icloud:UOR-R4/results/codex/gate2561-r1-archive.tar` (56,320 bytes,
MD5 `7d42af37559759e3cf67a9c67db8ddd7`) and
`icloud:UOR-R4/results/codex/gate2561-r2-archive.tar` (116,224 bytes,
MD5 `71ad70f7732b7280d8111c4ae8be9f95`). Their lifecycle estimates are
$0.026898889 and $0.034249167; provider billing during the first instance's
stopped interval is not inferred. Both pods are deleted and fleet0 verified.
Source mathematics, image expectations and fixed experiment dose are unchanged.

Line: M1 qualification/cost tooling. Count0/3 and suspended headline37/232 remain
unchanged. At this revised source freeze, two startup/preflight leases total
$0.061148056; cumulative31-lease Codex estimate$3.235614444 plus separate PM,
startup/compile subtotal$0.802360555, storage/transfer USD
UNMEASURED. **Next:** exact-head source/construction approval and protected merge;
then distinct reviewed finite pins/release, accepted outcome, and only afterward
a separate fixed-dose timing release.
