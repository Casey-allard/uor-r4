# Pre-registration — M1 exact GPU baseline reuse

- Date / owner: 2026-10-16 / Codex–Sol, operational PM, M1 #2029.
- Mechanism: existing discrete geometric read and shared writer; eliminate repeated evaluation of candidate-invariant baseline eligibility. [Mechanism brief](../../mechanisms/discrete-geometric-learning.md).
- Authority: owner delegated directing Codex work and independent adversarial review; [native-math direction6224381783](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224381783). The [sealed decomposition](../../labs/counterfactual-decomposition-run-2026-10-16/README.md), protected #2559 `ddb081da8f5d6902fb067192c006059db783c45f`, selects this dependency.
- Line: M1 qualification / cost tooling, inherited count **0/3**, suspended headline **37/232**, unchanged. No language/quality KEEP, count reset or mechanism verdict.
- Delivery: **docs only**; optimization source and every new GPU observation **NOT_RUN**. This sheet requests independent design FIT, not an execution release.
- Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**; seed2 not released. No apply, training, new spending class or CPU model/reference computation.

## Bar and deciding question

Can one call compute its unchanged baseline read/writer eligibility once per row,
then reuse that exact result across all sixteen virtual candidates, preserving
all candidate losses, refusals and proposal words while reducing measured cost?

The sealed diagnostic measured baseline read/writer/eligibility at
46.921799–47.220974% and writer at 99.979269–99.979452% of lane cycles in all four
policy/operation cells. These overlap. They justify this experiment, not a
47% saving or a 16-fold speedup: the old candidate lanes execute baseline work
in SIMT lockstep and may share memory transactions; different rows in a producer
pass may diverge or change locality. Timing decides whether fewer evaluations
reduce elapsed cost. The model law and geometric representation do not change.

The correctness bar is **exact equality**, not a tolerance: all sixteen u64
losses, CUDA/host refusal status, five proposal words, state hashes and epoch.
Resource or parity failure is a blocker to repair, never a mechanism negative.
The engineering timing decision below is descriptive; it neither qualifies
training rate nor establishes language quality, fresh transfer or p95.

## Base artifact and fixed surfaces

Use the same original checkpoint0 and configuration as #2559:

| Item | SHA256 |
| --- | --- |
| Original snapshot | `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4` |
| Configuration | `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4` |
| Sealed diagnostic raw record | `a190547844b937508ed13731437063d226a7871f9800b928e52ec1b786197430` |

The [prior execution identity](../../labs/counterfactual-decomposition-run-2026-10-16/execution-identity.json)
and [pin](../../labs/counterfactual-decomposition-run-2026-10-16/decomposition-pin.json)
are reference evidence, not pins for a new binary. Before execution, bind the
new source/executable, ordinary and reuse source/PTX/cubin/report identities,
geometry, fixtures/generator, exact checkpoint bridge and toolchain/driver/GPU.
The production loader continues to reject source mismatches. Any evaluation-only
in-memory bridge must be reviewed for this exact source pair and preserve the
original checkpoint bytes and every non-source field; no blanket compatibility
exception and no rebound checkpoint.

U = ordinary unchanged counterfactual; R = separately admitted opt-in reuse
counterfactual. Preserve U's exact source/PTX/cubin bytes against the sealed
ordinary image, not merely its function name. Reuse a separate AOT module and
the current standalone instrument; no new general framework or model crate.
Both modes retain one grid block of sixteen threads, the same virtual read,
writer, objective, ordered sum and chooser. A/R policies and Route(0) /
VectorBlock(0) stay separate. Gate/Flock/Relation are outside this opt-in slice;
their existing paths remain untouched and the reuse entry refuses unsupported
read/proposal modes before submission. No candidate pruning, identity shortcut,
member index, pooling, persistent cache or separate launch-shape experiment.

## One bounded implementation hypothesis

The existing `carry_windows` contract admits 1..64 rows. Within a single R call:

1. Sixteen lanes produce distinct baseline rows `row = lane + 16*k`, limited
   by the admitted row count. Each uses unchanged `read_vector(kind=2)` and
   unchanged `writer(kind=2)`, including its checks. Do not replace the writer
   with a new eligibility approximation or omit its arithmetic checks.
2. Store exactly two u32 words per row: baseline status and eligibility.
   Static shared scratch is **64 × 2 × 4 = 512 bytes**. Every valid row writes
   both words even on failure; failed rows have eligibility zero. A baseline
   read failure stores status2; writer MAX stores its arithmetic status or8;
   otherwise store status0 and `token_rivals != 0`. Baseline loss itself is
   checked but need not be retained: the ordinary sum adds virtual costs only.
3. All sixteen physical lanes reach one unconditional block barrier. Producers
   never return early or publish a global error. No candidate-dependent branch
   or lane exit can precede the barrier; exact launch dimensions are bound by
   the host dispatch. The cache has no life outside this invocation.
4. Each candidate then walks rows in the original increasing order. Check a
   row's stored status **before** eligibility, and publish/return that status
   only if this candidate reaches the row. Skip baseline-ineligible rows;
   otherwise execute the original virtual read/writer, rival-removal MAX
   sentinel, checked accumulation and early termination. Chooser ties and
   identity priority are unchanged. No barrier follows candidate divergence.

The producer may evaluate rows a particular candidate would never reach after
an early exit. Disclose that extra work; do not expose its errors prematurely,
count it as a candidate visit, or claim it preserves original work order.
Producer writes and consumer visited/eligible masks must be distinct in finite
control evidence. This design preserves the admitted semantic result, not every
internal instruction or scheduling order. Existing atomic status publication
is not promoted into a new deterministic ordering theorem for multiple faults.

Bind scratch and the audit handle to the existing model/stream, admitted batch,
parameter, targets, objective policy and epoch. No handle can supply a previous
call's cache. A bounded audit output, precreated with the opt-in profiler, records
row producer status/eligibility and six u64 words per lane: producer mask,
producer call count, consumer visited mask, consumer eligible mask, virtual
visited mask and terminal status. This is 512 + 16*6*8 = **1,280 audit bytes**,
separate from the 512 shared bytes. Require disjoint producer masks whose union
is the admitted row mask, each count equal to its mask popcount, and total
count equal to row count. Use a dedicated full-mask case for64; never shift by64.
Initialize/reset the audit storage on the same stream inside the measured
call; every lane emits its complete terminal record, including fault paths.
No clocks or host model arithmetic. The same R image and audit stores
are used in controls and timing. Include their cost; no hidden faster image.
New audit buffers are metadata storage only, not reuse of existing target,
loss, status or proposal allocations. Full observation readback is accounted
separately as well as in the whole-unit cost below.

## Data split, controls and fixed dose

Only admitted literal fixtures and the original deterministic synthetic timing
windows/targets are used. No corpus, training/dev/held-out acceptance panel,
new teacher or seed search. Exact row/occurrence/target/order identities are
pinned before execution. All model/read/writer/reference work is on CUDA.

**Finite stage, separately released first:** use the six literal fixtures and
expected semantic outcomes in the [existing control packet](../../labs/counterfactual-controls-2026-10-16/execution-packet.md),
replacing its U/D pair with U/R. This is **40 proposal attempts**: 32 successes
and 8 expected all-ineligible status8 refusals. Retain every loss/status/word,
including identity/tie, mixed eligibility and virtual early exit. For the
virtual-ineligible two-row case, R producers cover mask3; identity consumes
mask3, nonidentity candidates consume mask1, with their original MAX semantics.

Add exactly three boundary fixtures: repeat the literal `eligible_reference`
row/target to lengths **16, 17 and 64**. Each uses both policies × both
parameters × U/R, **24 additional attempts**. All sixteen losses and proposal
words match U's actual CUDA result; no host loss reduction/reference. Each
consumer visited/eligible mask covers the admitted rows; producer masks are
disjoint partitions whose union covers those rows. No unwritten slot is read.
Together with existing one/two-row fixtures this covers lane-stride boundaries.
Total finite dose: **64 attempts, 56 successes / 8 expected status8**.

Retain the existing **16 admission refusals** (8 per policy), replacing D/probe
with R/reuse-handle: invalid Route index U, invalid VectorBlock index U,
Gate-in-Route U, wrong batch/model U and R, wrong profiler U and R, wrong reuse
handle R. Pin the exact command inventory and refuse unsupported reuse mode at
source/API admission. No claim of dynamically testing all Gate/Flock modes.
The 128 independent immutable endpoints from #2557 remain evidence for the
unchanged writer and U image; do not repeat or count them as new R observations.

CUDA2/4/9/12 reachability and isolated stale epoch limitations remain exactly
as scoped in that packet: no corrupt-ABI injection, no apply to manufacture a
stale epoch, no fabricated overflow observation. Source review must re-establish
those bounds for the new control flow. If it cannot, the finite release stays
blocked pending a specifically reviewed repair; do not waive the invariant.

**Timing stage, a distinct release only after finite acceptance:** same
checkpoint0, seed1, epoch0, rows64, V4096/B4/C192/K16, Route read and original
window/target generator. Per policy, batch0 runs U then R for both operations
as excluded warmup; batches1..3 run paired U→R, R→U, U→R, Route before VectorBlock.
Exactly **32 proposals: 8 warmup / 24 timed**, twelve matched pairs. Recarry
identical windows for each mode, retaining hashes. No apply or persistent cache.

## Measurement, minimum power and outcome decisions

Measure the same-stream counterfactual and chooser events, enclosing propose
call wall and host partition, plus carry + propose + full loss/audit observation
and remainder as a whole-unit wall account. Do not add overlapping waits to GPU
events. Record exact allocation/audit bytes, image resources and actual launch.
Use the same ordinary host/event profiler for both modes; no internal clock64
instrumentation. Audit stores are included in R timing; its extra readback is
included in whole-unit accounting. No profiling-off claim follows.

Retain every paired signed difference and ratio and per-cell min/max/median.
For each metric define `gap = min(U) - max(R)` and
`spread = max(max(U)-min(U), max(R)-min(R))` over its three timed samples.
The conservative descriptive improvement bar is `gap > spread` for both
counterfactual event and enclosing propose wall in **each** of the four cells,
with no whole-unit regression beyond its within-mode spread in any cell
(`max(R)-min(U) <= spread` for that metric). Event/wall accounting retains the
existing max(1ms,1% wall) consistency allowance; it is not a speedup threshold.

- Exact parity/resources/accounting fail: **BLOCKED**, preserve the complete
  partial record and repair the deciding defect. No speedup or negative claim.
- All four cells meet the timing bar and whole-unit condition: **engineering
  benefit observed at this fixed scope**; propose the separately gated
  100-timed-batch qualification rerun. This is not D22 KEEP or training release.
- Mixed cells, overlapping ranges or spread too large: **INCONCLUSIVE** for a
  general cost improvement; retain each cell and name the deciding limitation.
  No automatic extra sample, layout sweep or selective favorable-cell report.
- R is slower beyond spread: retain a configuration-specific regression;
  do not activate R. Examine its fixed work/scheduling/resource evidence before
  proposing a different implementation. The native mechanism is not rejected.

Do **not** reuse the decomposition's `abs(D-U)` perturbation veto here: U/R
separation is the intended effect. This run has one seed and three descriptive
pairs per cell, and does not satisfy the D22 decisive learning/fresh-transfer
contract. No language panel, p95, training-rate extrapolation, new learning law
or full-path energy claim. Saved work counts are not converted to seconds.

## Steps, construction and execution gates

Independent TEST FITNESS on this sheet precedes optimization source work.
Then separately review the narrow source change, host metadata tests and actual
CUDA check/Clippy transcripts at the exact head. Preserve ordinary image bytes;
scan every compiled/loaded module and reject prohibited instructions. Assemble
R's cubin from its admitted PTX, pin the report and load that exact image.

The old U/D shared-memory equality applies only to the old clock instrument.
For this new comparison admit **U shared0 / R static shared512 bytes**, no dynamic
shared allocation. Each runtime image must match **its own** AOT resource report
exactly; require zero AOT spill loads/stores and block16 launchability. Record
registers/local/stack and deltas; derive and independently review any additional
resource ceiling from construction evidence before model execution. No timing
based resource tuning or carry-over of the superseded register bar. A failed
resource predicate needs a reviewed repair, not a post-run waiver.

Protect and verify the source before finite execution. Publish exact pins,
fixed inventory, full cost and a distinct finite release; accept/seal its actual
outcome. Only then may a separate timing release authorize the fixed32 dose.
No on-lease patch, opportunistic timing during controls, automatic retry,
qualification run or model update. Owner holds remain regardless of FIT.

## Cost and stop rule

This documentation delivery uses **zero leases / $0 new GPU cost**. Carry
29-lease Codex estimate **$3.174466388** plus separate PM receipts unchanged;
storage/transfer USD remain **UNMEASURED**. Metadata review uses bounded local
host work, no host model; no datasets or training artifacts are generated.

Prospective timing upper planning reference: 32 × 7.48s ≈239.4s if both paths
cost the observed U maximum; this is not a runtime prediction for R. At the
last observed $0.536/h quote that component is ~$0.0356. Provisioning, build,
finite controls, admission, preservation and teardown are additional. Plan a
30-minute lease allowance per separately released stage (~$0.268 at that old
quote); verify actual quote/full cumulative estimate before rental. This is an
estimate, not a cap on a healthy progressing run. No new spending class.

Before each stage enforce local and pod `df >=30 GiB`, exact ownership, fleet
<=4 pods and <=$8/h across labs; use one Vast RTX5090-class GPU through `uor-pod`
only. Pin actual hardware/toolchain/driver; changes need prospective disposition.
Name lab/session, trap before creation, renew before lease expiry. Record source,
command, exits, every warmup/fault, cost and before/after state. SHA256 manifests
and full cloud MD5 download/readback must verify **before** release then down.
Verify final fleet state; retain lease and evidence when state is unknown/live.

First parity, identity, state/apply, accounting, launch or hard resource/disk
fault ends the affected dose and preserves evidence for diagnosis. A failed
instrument is repaired as a blocker; it is never scored as mechanism failure.
Self-set wall estimates do not cancel healthy runs. No unscheduled expansion.

## Verdict

TEST FITNESS: FIT

Independent design review `review_2551` is recorded on
[#2029, comment6226136376](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6226136376)
and [PR #2560](https://github.com/UOR-Foundation/uor-r4/pull/2560#issuecomment-6226136069).
It binds the prospective sheet at `925422da5290af9cab410c8bf4f413f2467dc563`,
SHA256 `b90aaae2f7068da11823f44e6158b1de930a0e701964ad9d7dc18ee982f7b405`.
This later verdict annotation changes no hypothesis, dose, bar or permission.
No optimization code or execution release is included in this PR.
