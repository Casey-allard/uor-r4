# Pre-registration — M1 counterfactual-kernel decomposition

**Subsequent execution, 16 October:** the [sealed Stage2 record](../../labs/counterfactual-decomposition-run-2026-10-16/README.md)
completed the original 32-proposal diagnostic after independent FIT, finite
controls, readiness and distinct release6225716504. All four cells resolve H2
material baseline repetition and H3 writer dominance under the original bars;
all U/D parity, resource, state and perturbation checks pass, apply0. This does
not grant an optimization/training release. The prospective design and its
earlier pending labels below are retained as dated provenance.

- Date / owner: 2026-10-16 / Codex–Sol, M1 #2029.
- Authority: [PM commission 6217437395](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6217437395), [post-merge disposition 6218096172](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6218096172), [source cost card 6209851483](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6209851483).
- Line: M1 qualification instrumentation; count **0/3**, suspended headline **37/232**, unchanged. No mechanism negative, KEEP, acceptance or count reset.
- Delivery: **docs only**. No instrument, kernel, launch, allocator or model change in this PR. All new measurements below are **NOT_RUN**; this is the design submitted for PM-seat review.
- Subsequent state: design merged in #2545 (`98ac653c4`), with [PM-seat FIT6219037898](https://github.com/UOR-Foundation/uor-r4/pull/2545#issuecomment-6219037898). The original prospective design below is retained; the [instrument execution packet](../../labs/counterfactual-instrument-2026-10-16/execution-packet.md) operationalizes its six binding review requirements. No execution release follows.
- Review at original delivery: Kimi PM owned TEST FITNESS and dispatched the independent PM-seat review. Later owner direction in this Codex chat delegates operational PM routing to Codex while Kimi is unavailable, permits independent adversarial subagents and reciprocal AGY review, and leaves SpiralCore as separate DeepSeek research. Codex authoring is still independently reviewed; no training/compute hold changes.
- Finite-control scope clarification, **FIT PENDING / NOT_RELEASED**: the [control packet](../../labs/counterfactual-controls-2026-10-16/execution-packet.md) proposes admitted GPU cases plus explicit source-bound unreachable statuses and a zeroapply stale-epoch limitation. It does not silently replace the generic control requirements below. Independent exact-packet FIT must approve that prospective disposition before model execution; original design FIT alone is insufficient.
- Scope: **diagnostic, not qualification / not a p95 estimate / not a training rate**. Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**. FIT releases none of these holds.

## Bar and deciding question

Where does the approximately 7.25-second historical propose cost live *inside*
`discrete_counterfactual`, and which smallest exact implementation change is
worth testing? The sealed diagnostic now measures approximately **7.32–7.37 s**
per call. It already localizes **99.995770–99.998097%** of enclosing propose wall
to the counterfactual event interval, across all twelve timed profiles and all
four policy × operation cells. Chooser events are **13.344–42.495 µs**.

The [sealed record](../../labs/propose-subphase-diagnostic-run-2026-10-16/README.md)
and its [raw profiles](../../labs/propose-subphase-diagnostic-run-2026-10-16/k16-propose-subphase-record.json)
are observations; the work counts below are source-derived upper-shape counts;
the hypotheses are unmeasured causal explanations. Do not label a tiny active
thread fraction as measured occupancy, utilization, bandwidth or a speedup.

Success for this diagnostic means complete, semantics-matched internal phase
and work-count records that distinguish repeated baseline work, writer work
and API overhead, or a named unresolved attribution with its deciding test.
No model-quality bar is introduced. Coarse **80% / 20%** phase-share routing
below follows the predecessor sheet; these are engineering prioritization
rules, never mechanism acceptance thresholds. Keep all four cells separate.

## Base artifact and source boundaries

- Authoring authority: `origin/main=90c2d2b9d14a0edc1b4728b811415bddf3b5fc4d`.
- Sealed receipt: #2543, merge `1f5a46ebfdeb5546a7cc7b7f933c355769760aaf`;
  executed source `8199af0076d861bade607b7aa2fc29bfaecb81ae`.
- Exact checkpoint0 / config / geometry / original and executed source/PTX pins:
  [sealed diagnostic pin](../../labs/propose-subphase-diagnostic-run-2026-10-16/diagnostic-pin.json).
  Snapshot SHA256 `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4`;
  raw record SHA256 `3b53d6c49df76b627e2db407455b75493afa8396ea9f7afcf9e1651e3c8ff961`.
- Fixed shape: seed1, epoch0, rows64, V4096 / B4 / C192 / K16, Route read;
  A=`nearest_address_margin_v1`, R=`occupied_cell_rank_hinge_v1`;
  each batch calls `Route(0)` then `VectorBlock(0)`, sixteen ordinals each.
- Source anchors in this base: [`device.rs`](../../../crates/uor-r4-discrete/src/device.rs)
  `launch_config`, `propose_inner`; [`discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu)
  `discrete_counterfactual`, `read_vector`, `writer`, `token_cell`, `discrete_choose`;
  [`proposal_timing.rs`](../../../crates/uor-r4-discrete/src/proposal_timing.rs)
  `HOST_SPANS`. The source cost card pins its earlier line references; function
  names here resolve against the authoring base rather than shifted line numbers.

The current launch is grid=(1,1,1), block=(16,1,1), one candidate ordinal per
thread. Each lane visits up to 64 rows serially. For each visited row it calls
unchanged baseline `read_vector(kind=2)` and `writer`, then tests baseline token
rivals (`tr != 0`) for eligibility. Only eligible rows run virtual read/writer.
Virtual refusal or absent rivals can terminate a lane early. Record actual
visits, not an assumption that all lanes reach all rows.

If all 64 rows are visited and eligible with no early exit, this is **1,024
baseline + 1,024 virtual read/writer evaluations** per proposal, although only
64 baseline rows are distinct. Each writer scans V tokens for occupancy and
again for each selected cell; with all sixteen cells selected the source work
count is **16 × 64 × 2 × 17 × 4096 = 142,606,336 token-cell classifications**,
excluding separate target-cell calls. It is not an instruction count or a
measured time split. R adds occupied-cell rank work; both policies share the
membership scans. Candidate0 still performs an identity-equivalent virtual
calculation in the current source.

## Data split

Use only the predecessor's deterministic synthetic timing windows/targets,
including its warmup sequence, in the same order. No corpus, train/dev/fresh
transfer panel or Q4 artifact. Preserve exact row/target identity for each
matched call. No apply, optimizer, epoch advance or checkpoint rewrite. This
probe neither trains a mechanism nor qualifies fresh transfer.

## Smallest falsifiable hypotheses and decisions

These hypotheses can coexist. In particular, a slow writer can be duplicated
across candidates *and* serialized by the launch; their predicted savings must
not be added as independent speedups.

| Hypothesis | Prediction in merged sub-phase numbers | Smallest distinguishing measurement | Bounded optimization justified if supported |
| --- | --- | --- | --- |
| **H1: one-block, one-lane-per-candidate serialization limits throughput.** Independent candidate/row work is left undistributed. | Counterfactual event stays dominant; `status_read_check` waits for it; submission/chooser remain tiny. All sixteen lanes visit many rows. The merged instrument alone cannot distinguish H1 from expensive per-row work or local-memory pressure. | Record actual grid/block, per-lane visited/eligible counts, total lane cycles and compiler/runtime resources. These establish serialization, not its causal seconds. A later separately reviewed launch-only comparison must distribute the same candidate×row work and perform an exact checked reduction, while holding writer/recomputation constant and comparing all sixteen losses/status/proposal words. | Parallelize candidate×row work with deterministic exact reduction. Merely changing block16 to block128 leaves only sixteen active candidate lanes and does not test H1. No writer cache or candidate pruning in that first comparison. No presumed 64×/SM-count speedup. |
| **H2: repeated baseline read/writer/eligibility is a material avoidable share.** | Baseline work contributes at least 20% of measured lane cycles; baseline calls repeat for each lane's visited row. With fully eligible rows and comparable baseline/virtual writer cost, baseline may approach half of read/writer work; that is a conditional prediction, not a promised fraction. Merged counterfactual/status spans look the same as H1. | Separate baseline read, baseline writer, eligibility and virtual read/writer cycles and exact invocation counts per candidate. Check row identity and baseline rival mask agreement across lanes; report early exits. Baseline share below 20% deprioritizes H2 as the first optimization. | Compute baseline loss/eligibility once per row on GPU and reuse only within the same model binding, epoch, policy and exact carried batch/targets. Preserve baseline-fixed eligibility, including all-ineligible/error paths. A later one-variable cache comparison measures saved time; the decomposition cannot establish it. |
| **H3: writer vocabulary/cell replay dominates each row, rather than geometric read or eligibility.** | Baseline+virtual writer phases occupy at least 80% of measured lane cycles in each cell; read/eligibility/reduction together at most 20%. Both A/R share most work; an R-only tail appears in final rank work if material. | Coarse writer subspans: occupancy scan; selected-cell selection plus member scans; final target-distance/hinge/rank work. Count selected cells, `token_cell` calls and member distance evaluations separately. These locate writer work without pretending to separate membership decoding from distance instructions inside a shared loop. If read dominates instead, H3 is contradicted and indexing is not the justified first change. | GPU-resident exact membership/occupancy index scoped to model/codebook epoch and virtual move. Route leaves codebook membership unchanged; VectorBlock(0) touches an endpoint, so virtual membership must reflect that token's moved endpoint. Retain every crowded-cell token, exact cell order, ties and target/rival rules. No approximate shortlist or CPU index scoring. If decoding versus distance remains unresolved, a later exact membership-reuse intervention is the deciding test. |
| **H4: per-call allocation/upload/readback accounts for seconds.** | Would require large disjoint non-wait host spans or a large wall-minus-events residual. Sealed data already contradicts this as the dominant explanation: all other individual host-span means <63 µs/cell; wall minus counterfactual+chooser events is 126.865–267.292 µs/sample. `status_read_check`'s ~7.3 s overlaps the GPU event and is not transfer time. | Reuse the merged nine host spans and event intervals, not another allocation-only run. Report targets/spec upload, losses/status zeroing, proposal allocation and cleanup individually; preserve clock residual and separate readback/caller-sync costs. No exclusive allocation causal estimate from a mixed host span. | Pool the five small buffers or batch-owned uploads only if later data makes overhead material to the optimized path. H4 does **not** justify allocation-first work now, nor deleting epoch/status checks or synchronization. A future pooling comparison is separately reviewed after dominant kernel work is reduced. |

The observed per-call footprint is five allocations and **476 payload bytes**:
targets256, spec48, losses128, status4, proposal40; epoch read8 and status read4
are separate DtoH transfers. Allocator/event metadata are excluded. The model,
geometry and carried batch are already device-resident. This accounting comes
from source, not from a measured transfer-bandwidth result.

## Prospective instrument: exclusive work spans, overlapping GPU time

After design FIT, separately implement and review a timing-only diagnostic
variant. Keep the ordinary production path available unchanged. Extend the
merged enclosing event/host instrument with integer CUDA `clock64` records and
integer counters. No floating-point model/reference/instrument, matmul,
prohibited squashing or CPU scoring; host work is receipt parsing only.

For each of the sixteen candidate lanes record disjoint accumulated cycle
intervals for baseline read, baseline writer, baseline eligibility, virtual
read, virtual writer, and row accumulation/control. Include lane start/end,
setup/tail and uncovered clock/store overhead so accounting gaps are visible.
Writer internals have the three nested coarse subspans in H3. Nested writer
cycles are a partition of writer time, **never added again** to outer totals.
No per-token/per-coordinate clocks, global atomic timing accumulation, new
barrier, changed early return or extra synchronization. Store per-lane results
once, with explicit completed/fault flags and bounded buffers, including
refusal paths. Check buffer bounds and overflow as instrument integrity.

Record exact visited/eligible/virtual-row counts, writer/read call counts,
selected-cell counts and source work counts per lane. Candidate0 and lanes
that return early remain visible. Counts must reflect executed work rather
than multiplying nominal rows. Collect build flags, registers, local/shared
bytes and spill diagnostics plus GPU identity/driver/clock context. The sealed
**21,536 local bytes** is resource metadata; it does not establish spills or
occupancy loss. No unbounded profiler sweep or extra metric pass is included.

Cycle spans on different lanes overlap. Report each lane separately, the lane
with maximum total cycles and the across-lane work-share ratio (sum of phase
cycles / sum of accounted lane cycles), with all per-lane shares retained.
A maximum-duration lane is only a critical-path proxy under this fixed launch,
not exclusive wall attribution. **Never sum lane cycles into GPU wall seconds,
add GPU events to CPU wait spans, or convert cycles using an endpoint clock as
though a continuous fixed frequency were measured.** `clock64` includes stalls;
it does not distinguish instruction throughput, memory latency or bandwidth.
The enclosing same-stream CUDA events remain the elapsed-time authority.

## Steps and minimum test power

This is a bounded localization diagnostic, not a decisive learning/quality run;
the D22 ≥2-seed/≥200-fresh-row learning criterion is not being claimed satisfied.
Use one seed and three descriptive matched samples per operation/policy cell.
No p95, quality KEEP, fresh-transfer result or training sizing follows.

1. **Gates first:** PM-seat TEST FITNESS on this design; separately reviewed
   instrumentation source, actual CUDA compile/strict-clippy transcript, focused
   GPU parity controls, protected merge and fresh-main delivered-byte check;
   then PM acceptance of exact executable/PTX/source/geometry/config/data pins
   and a separate execution release/resource admission. This PR authorizes
   none of those executions. No kernel work before design FIT.
2. **Identity:** preserve original checkpoint bytes and loader refusal. Internal
   clocks necessarily change PTX and composite source identity. The predecessor
   bridge requires unchanged PTX and cannot be reused by changing a hash. A
   separately reviewed evaluation-only bridge must explicitly admit the exact
   old/new source **and PTX** pair with identical payload/epoch/config/geometry,
   record both mappings and forbid training resume. Missing bridge is a named
   blocker before lease; no production-loader weakening or `jq` rebinding.
3. **Matched modes:** U=ordinary merged proposal path, D=decomposition path on
   the same device and synthetic batch, with the same enclosing profiler in
   both modes. Retain U's unchanged ordinary kernel/PTX identity separately
   from D's diagnostic module identity in the reviewed execution packet.
   Each A/R arm has one excluded warmup batch per mode, then exactly three
   timed batches per mode. Within a timed batch alternate U→D, D→U, U→D order;
   each mode still proposes Route(0) then VectorBlock(0). No apply between calls.
   Totals: **24 timed + 8 warmup proposals**, 6 matched timed pairs across
   both operations per policy; equivalently **3 pairs per each of 4 cells**.
   There are 12 matched timed proposal pairs overall. No extra samples/retry.
4. **Parity:** compare all sixteen integer candidate losses, status/refusal,
   chosen ordinal, identity/selected loss, sensitivity and proposal epoch
   between each U/D pair. This requires a reviewed read-only loss/status
   observation in both modes; the existing five-word observation alone is
   insufficient. Record source-bound ordered batch/target identities, both
   policy snapshots before/after, epoch0, zero apply and unchanged checkpoint
   file hashes. Failure/overflow/ineligible controls need focused finite GPU
   parity cases before this diagnostic release, not a CPU oracle.
5. **Perturbation and accounting:** keep U/D host spans, enclosing events,
   internal cycles, resource metadata and readback bytes/time. Same-input mode
   comparisons bound runtime instrumentation effects only descriptively;
   separately compiled modules may alter register/spill layout. For each cell
   let E be the largest absolute paired D−U counterfactual-event duration and
   S the larger within-mode max−min event spread. Report E, S and their rational
   shares of U duration. If max(E,S) reaches 20% of that cell's minimum U event
   time, or per-lane phase shares straddle the 20/80 routing boundaries, call
   attribution **UNRESOLVED** rather than applying a dominant-phase label.
   The 20% limit matches the coarse routing resolution, not timing precision.
   A result below it remains descriptive; do not subtract E as a correction.
   Event/wall consistency still uses predecessor max(1 ms, 1% wall) tolerance.
6. **Seal and stop:** preserve every cell, warmup/fault record, exact commands,
   exit status, build/source/module identities, cost and before/after state.
   Archive SHA256 manifest and verify cloud MD5 download round-trip **before**
   release/delete; verify fleet state. Record a blocked or mixed result without
   extending dose or starting an optimization on the same lease.

## Outcome decisions and optimization boundary

- H2 material (≥20% baseline share in every retained lane with complete work,
  stable across samples) plus parity/accounting intact: propose baseline reuse
  as the smallest one-variable optimization; do not claim its saved seconds.
- H3 dominant (≥80% writer share throughout a cell) with H2 small: propose exact
  writer membership reuse, or the named unresolved inner-loop discriminator.
- High baseline **and** writer shares: they are nested explanations. Start
  with shared baseline reuse if it removes duplicated writer work with the
  narrower change; retain the writer breakdown for the next decision.
- H1 remains a structural candidate under any split. If H2/H3 do not select a
  narrow reuse change, the next deciding test is the separately preregistered
  launch-only parallel decomposition. This sheet does not claim to measure
  its speedup, and no launch variant is executed in the localization dose.
- A/R or Route/VectorBlock disagreement: publish separate choices. Do not pool
  cells to manufacture dominance or optimize only a favorable cell silently.
- Phase ambiguity, large perturbation, parity or accounting fault: **UNRESOLVED /
  INSTRUMENT BLOCKER**, retain evidence and return the named repair/discriminator
  to PM; no geometry rejection, silent repeat or training-rate extrapolation.
- H4 stays secondary on sealed evidence. If dominant kernel costs later fall,
  remeasure its share before proposing scratch/transfer reuse.

Every subsequent optimization needs exact loss/proposal/refusal parity with the
unchanged path, including baseline eligibility, crowded cells, virtual endpoint
moves, identity/ties, checked accumulation/overflow and stale epoch. Parallel
reduction must preserve the existing failure/early-exit semantics, not merely
produce a plausible sum. GPU-only exact geometry and one writer remain binding.
A successful optimization then precedes the separately released **100 timed
batch** qualification rerun, revised training sizing and owner's D-Release
judgment. None is piggybacked on this diagnostic.

## Cost and resource plan

**This delivery: zero leases, zero GPU/model execution, $0 new GPU cost.**
Local docs/source review and checks are non-model work. Carry the sealed
**18-lease episode estimate $2.230435278** forward unchanged; it is not an invoice.
Storage/transfer dollar charges remain **UNMEASURED**, never assumed free.

Prospective matched dose doubles the predecessor's sixteen total proposals to
32: about **236 s** of propose work at 7.37 s/call, before diagnostic overhead.
At the predecessor $0.908/h quote this is about **$0.060** for that component
alone. Provisioning, builds, finite parity controls, admission, clocks/readback,
archive and teardown are additional. A **20-minute planning allowance** at
that quote is about **$0.303**, not a price guarantee, authorization or hard
healthy-run cutoff. Pin actual quote/full job estimates in the execution packet;
if overhead invalidates the measurement, preserve the bounded result rather
than expanding dose. No new compute spending class is proposed.

If separately released, one RTX5090-class Vast.ai GPU through `uor-pod` only;
status/ownership/caps before lease, named lab/session, EXIT trap before creation,
lease duration covering setup/build/controls/dose/archive plus margin, renew
before expiry. Preserve GPU/toolchain match or get PM disposition for a hardware
change. Caps ≤4 pods and ≤$8/h across labs. **Enforce `df` ≥30 GiB free before
lease** and retain the receipt: the [PM floor all-clear](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6217793838)
does not replace the next preflight. Log cumulative provisioning/build/control/
measurement/retry/storage/transfer/teardown costs and retained bytes; archive
round-trip before release then down. No unsupervised lease or peer-resource use.

## Stop rule

First identity, parity, unexpected apply/epoch change, bounds/clock/accounting,
command, resource-cap or disk-floor fault stops the affected execution and
seals partial evidence for PM disposition. No on-lease patch, extra batch or
unreviewed retry. Instrument faults are D22/D31 blockers to repair, never
mechanism negatives. Renew a healthy admitted run rather than canceling it
because a planning allowance elapsed. Preserve peer work and all original
artifacts. No allocation pooling, launch change, writer cache, Q4, qualification
or training on this sheet's present authority.

## TEST FITNESS and next dependency

TEST FITNESS: FIT — Kimi PM, #2545 comment6219037898, exact head09ad85bb4 (design only; no compute release).

Kimi PM rules on this exact design with PM-seat independent review. A design
FIT permits the commissioned next source work only within its reviewed scope;
actual diagnostic execution still requires its source/device/parity gates,
reviewed identity bridge, protected delivery and PM execution release above.
The next deliverable after this docs-only PR is called by PM; no automatic
kernel rewrite, lease or training follows.
