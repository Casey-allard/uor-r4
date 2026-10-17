# Native Relation learning path — 17 October 2026

Owner-prioritized implementation of the missing learned-attention path, using
existing discrete geometry and the shared writer. This is source preparation;
**no learned attention result, model execution or hold release is recorded.**
Owner implementation/coordination direction is recorded on [M1
6229960779](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229960779),
following the [source assessment
6229756412](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229756412).
Codex/cf1 owns implementation; Kimi retains fitness, protected queue and execution
coordination; AGY has an independent source/math review handoff.

## Problem and implementation

The previous `SchedulePolicy` implemented only vector-then-read ordering.
At V4096/B4/C192, a 16,404-batch canonical Relation dose would visit all
16,384 vector coordinates and twenty combine gates, with **zero** of the twenty-five
relation parameters attempted. All-zero combine initialization also makes an
initial relation-only proposal invisible at the writer loss. These are source
reachability gaps, not measured negatives for relational attention.

The existing driver now implements `InterleavedReadEvery8V1`: every eighth batch
is a read proposal; other batches visit vector coordinates. Scheduling uses the
**absolute completed-batch ordinal**, including after cursor wrap and resume.
The metadata cycle is 16,404, independently of Relation's 16,429-coordinate
inventory. A full registered dose has 14,354 vector and 2,050 read attempts;
Relation's read family comprises 920 combine and 1,130 relation attempts.
Corpus order, identities, geometric carrier, shared writer and objective are retained.

`read_initialization: active_leaf_v1` is an explicit prospective configuration:
after the byte-unchanged production CUDA initializer, the opted-in constructor
uploads literal +1 at each payload block's leaf-combine gate and zero at every
other combine/relation gate. This is declared storage preparation, with no
target-dependent computation or learning apply; epoch remains zero. The
initializer ABI and default body remain unchanged. The existing omitted/default `zero_gates_v1` keeps its
old meaning and serialization. Initialization identity binds configuration,
lineage, model binding and snapshot; a canonical/zero checkpoint cannot silently
resume into the new experiment. GPU storage readback verifies the declared
initial pattern. Learned gates may subsequently change.

An active path supplies a learning opportunity, not a guaranteed gain. Snapping,
ties and writer exclusion may still hide a particular proposal's effect. No
answer-derived gate, source index, teacher, surrogate optimizer or host forward
is added. Treatment and the primary relation-disabled control use the same
Relation read inventory, leaf initialization and interleaver. The control's
explicit `relation_learning_policy: frozen_zero_v1` keeps all 25 relation trits
zero, observes the same scheduled counterfactual proposals, and refuses their
commits through the existing transition. The policy binds config/lineage and
checkpoint admission; default omitted `learn_v1` preserves older configuration
bytes. It is an experimental control, not another reader or optimizer.
Both arms therefore receive 920 combine and 1,130 relation slots. Refused slots
retain genuine hypothetical sensitivity observations and have zero accepted
updates; they are not reported as all-tied model observations. Flock remains a
secondary identity control with 2,050 combine proposals, a frequency difference
that prevents using it as the primary causal comparison. A Route-only comparison
cannot attribute relation benefit.

## Scoped executor and evidence

The attention executor reuses `PreparedDriver`'s CUDA transition and existing
release/checkpoint/lock machinery. Legacy A2 pilot/completion permissions remain
limited to canonical Route with zero-gate initialization; attention uses a distinct
arm/seed release scope. No D33 delegate can create an attention owner release.
The record is an operational safeguard on the shared OS account, not cryptographic
proof of the owner's identity. No agent invokes the owner-release command.

The new path records actual VectorBlock/GateCombine/GateRelation attempts,
candidate-cost sensitivity, declared relation refusals and accepted updates, checkpoints before first apply
and every 64 transitions/end, and refuses incompatible resume state. Final output
retains the trained checkpoint, full DEV rows and same-trained-state relation-off
ablation. A separately pinned transfer corpus is evaluated on CUDA after the
fixed training dose, without checkpoint or initialization selection on its labels.
The attention packet retains the existing minimum 2,000 transfer targets and
500 successor rows; successor coverage and held-out provenance require the
reviewed data packet. The generic transfer evaluator's 200-row D22 floor does
not replace those attention-specific requirements.
Different file hashes alone do not prove semantic independence; the reviewed
packet must establish held-out provenance and its fixed decision bars.

The existing successor-panel scorer can consume the per-row evaluations. Report
all 3,847 successor rows and the 10,440-row complement, actual target admission
and conditional rank, matched-arm comparison and same-checkpoint ablation. A
TRAIN-only successor control tests frequency explanations. A context-local rule
is tautological on the target-defined successor panel and is not learned evidence.

## Verification and execution status

| Check | Status |
| --- | --- |
| Host metadata library tests, including schedule/cursor/default/lineage/control | Prior6ca599a53 compiler-repair candidate:65 passed,0 failed,1 ignored; not rerun for prime-zeta delta; no model execution; [candidate hashes/logs](compiler-repair-host/metadata-tests.log) |
| Active initialization and matched-off CUDA control | Implemented as additional `qualify-relation` control; NOT_RUN |
| Host `check --no-default-features --all-targets` | PASS for prime-zeta repair candidate; [log](compiler-gate6-repair-host/metadata-check.log), existing nonfatal warnings retained |
| Strict host Clippy | BLOCKED by existing dependency lints in unchanged integer crate; not a pass |
| Advisory host Clippy | Completed with warnings; no suppression added |
| CUDA compiler/Clippy | Gate6 at6ca599a53: all-target check PASS0 (47.91s), strict package Clippy FAIL101 at three unchanged-main prime_zeta sites; confined repair pending new exact-head transcript |
| Finite CUDA qualification / model work | NOT_RUN |
| Independent integrated source review | APPROVE through `202855294`; AGY source/math APPROVE at `f6c7fa5e1`; subsequent initializer/fixture and canonical-sheet deltas independently approved; no runtime verdict |
| Scientific initialization/comparator design | FIT (design), PR #2574; no execution release |
| Training / M2 / Stage B | HELD |
| Q4 | NOT_EXECUTABLE |
| Model updates / compiler pod leases | 0 / 6; gate6 native elapsed cost proxy $0.161958333, cumulative mixed Codex estimate $4.320722222 plus PM; not invoices |

The local disk was below the 30 GiB pod-admission floor at source preparation;
no pod was started. Source preparation used a sparse owned worktree. Subsequent
host metadata verification materialized about 90 MiB of tracked sources and
used a small isolated, regenerable Codex target directory, with debug information
and incremental output disabled. This is compiler/storage/schedule verification,
not CPU model/reference execution or pod admission. The shared model-time JSON was not available
at its historical path; this preparation adds no model-time charge. Live harness
receipts record the coordination; model/compile timing remains NOT_RUN.
The old 3.74-second batch projection is historical, not a current budget.
A future reviewed packet must price its actual source, full arm/seed dose,
construction, baseline/final/ablation/transfer evaluations, retries/resumes,
checkpoint/archive storage and resource margin from current measurements.

The first host invocation exposed missing tracked include files in the sparse
checkout; materializing those exact files fixed setup. The first test run then
found a stale source-inventory spelling after the guarded apply expression and
an absent ignored `local/` parent for an existing storage test. The inventory
still permits exactly the same two apply call sites; its expected expression
was updated, and the test's parent directory was created without changing its
source. The complete rerun passed. Strict host Clippy stopped on pre-existing
`native_geometry.rs` dead code and `geometric_span.rs` style in the integer
dependency; the source unit does not silently cap that result or expand into
unrelated lint repair. CUDA check/strict Clippy and finite behavior remain
separate, unexecuted requirements.

Line: M1 learned-attention link (Relation interleaved read), accepted by PM
[6229949186](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229949186).
No historical count is reset by this source delivery; the milestone owns count
disposition and its suspended 37/232 headline remains unchanged.

## Next dependency and interpretation

### First compiler lease: preflight blocked, no compiler result

The compiler-only release at milestone comments `6230657270`/`6230824049`,
reaffirmed by `6231087599`, admitted source `bf8c7e965`. The first local launch
refused a missing runner companion before allocation; restoring exact-source
`hot-set.txt`/`uor-store` passed independent packaging review and the existing
runner's dry-run preflight. No pod or charge followed that first refusal.

The actual lease `vast-56364467` then ran from 07:49:17Z to 07:53:11Z on
17 October, with provider-reported uptime of 232 seconds at $1.051/hour,
an estimated $0.067731111. The ledger up-to-down wall interval is 234 seconds,
a separate $0.068315000 estimate; neither figure is an invoice measurement.
SSH became ready after 148 seconds. Setup reports exit zero,
but the gate script reports exit one before printing the source head or
invoking any Cargo gate. All CUDA/host compiler and metadata results at this
head remain NOT_RUN; no model or qualification command ran.

The wrapper creates its output directory beneath `evals/` before requiring a
completely clean checkout, and the pod runner also creates its job files under
`jobs/codex/`. Neither path is ignored by this source's `.gitignore`. A local
Git-only reproduction reports both as untracked. This establishes a defect in
the wrapper's preflight composition. The sealed remote record does not contain
the rejected status listing, so attribution to that exact predicate is a source
inference rather than a captured remote diagnostic. The repair must retain
whole-tree tracked-source checks, admit only this session's generated output
and job paths, and print the source/status evidence before refusing. It does
not change model code, the scientific design or its bars.

The 336,896-byte archive
`icloud:UOR-R4/results/codex/attention-compile-gate1-archive.tar`, MD5
`afbaee141d854857c30a92cfa6b686e6`, passed local SHA inventories and complete
cloud MD5 readback before release/down; the fleet returned to zero. The failed
packet and transcripts are preserved. A corrected packet needs prospective
review and explicit disposition before another paid attempt; no on-lease patch
or automatic paid retry is authorized by this receipt. Cumulative Codex lease
estimate is $3.558663333, plus separate PM costs; storage/transfer remain
UNMEASURED. This is an operator preflight blocker, not a mechanism negative.
Independent outcome review accepts the operational blocker receipt and verifies
all 8 remote, 18 operator and 10 frozen packet SHA entries. A prospective
session-scoped guard repair passes eight Git-only fixture cases: owned runtime
paths pass; unrelated files, peer jobs, unstaged/staged source changes, tracked
changes within allowed paths, a staged addition removed from the working tree,
and wrong HEAD are refused. The index and working tree are checked separately. Those metadata checks
are not compiler/model execution or permission for another lease.

### Second compiler lease: captured reaper-path refusal, no compiler result

Distinct PM compiler-only release [6231616281](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6231616281)
admitted source `c4907cd04c44073dd28813734c5dd2f1223c684b`, packet manifest
`3fe972314751367b2e5aa166df2e0ba828eaf843315ca44b4a7a83e118c61dc9` and
readiness `44929a1d4c703a14538d31a1ea311fdff677b038a05da8208696e821842d3455`.
After fresh full disk-reserve and fleet admission, `vast-56382752` was created
at 10:40:32Z. SSH became ready after 546 seconds. Setup exited zero; the native
job ran at 10:51:01–10:51:02Z and exited one **before any Cargo command**.
All four compiler/metadata gates and finite CUDA controls remain NOT_RUN;
no model, qualification, learning or evaluation command ran.

Unlike gate1's inferred predicate, this guard directly prints matching expected
and actual HEAD and captures the sole unexpected path:

```text
?? pods/reaper.log
```

The unchanged setup helper starts the reaper, which writes that exact file.
The session-scoped eval/job repair omitted this transitive setup output. The
[raw status](compiler-gate2/source-status-unexpected.txt),
[command transcript](compiler-gate2/command.log) and
[independent outcome review](compiler-gate2/independent-outcome-review.md)
record an operator inventory defect, with no compiler or mechanism verdict.
The raw reaper log and launcher log were not collected in this attempt and
are UNAVAILABLE; their contents are not reconstructed from the status record.

The 353,280-byte archive
`icloud:UOR-R4/results/codex/attention-compile-gate2-archive.tar`, MD5
`b368f635e426cbb6675a327bc05980d5`, passed all 10 remote, 20 operator and
12 frozen-packet SHA entries plus complete cloud MD5 readback before release
10:51:48Z/down 10:51:55Z. Fleet returned to zero. Provider uptime of 679 seconds
at $0.96/hour estimates $0.181066667; the separate 683-second ledger wall
interval estimates $0.182133333. Neither is an invoice. Cumulative Codex
provider-based episode estimate is $3.739730000 plus separate PM costs;
storage/transfer remain UNMEASURED. The closed release cannot authorize a retry.

The prospective correction audits the complete invoked setup/runner/reaper/gate
path inventory. It adds only the exact untracked `pods/reaper.log` allowance,
retains global index-vs-HEAD and worktree-vs-index checks, and collects raw
reaper/launcher logs, installed reaper/setup provenance and failed probe receipts.
Failed probe paths remain unexpected and blocking. No source ignore, broad
`pods/**` exception, numerical CUDA, scientific-sheet, compiler-command, model
or frozen Route/timing3 change is introduced. Eleven local Git-only guard cases
pass: the setup-generated eval/job/reaper topology is accepted; an unrelated
pod file and a staged change to the allowed reaper path are refused, alongside
the eight earlier cases. These are operator metadata checks, not compiler/model
execution or permission. A final-head prospective packet needs independent
review and a distinct PM dispatch before another paid lease.

### Third compiler lease: CUDA check passed; strict package Clippy blocked

Distinct PM dispatch [6233440554](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6233440554)
released compiler-only source `96ecbd4389cf646274d74ac373c46293150ab81e`,
manifest `310891985028c706f5bfaf5bfd55942951e75cb8a7606d02047aff62543c42dd`
and readiness `c6c9c1311fd415ac2b7d56f793f70c39c96a919ac46f8995bfcab2381c44871b`.
The complete transitive inventory and fifteen path-class fixtures were
independently reviewed by Codex's non-author reviewer and AGY; the final
inventory admits only the exact reaper log beyond the previous session paths.
Fresh full disk reserve and actual fleet/rate admission passed before allocation.

`vast-56389498` reached SSH after34 seconds and setup exited zero. The source
guard passed. At the released head, **CUDA all-target check passed**, exit0,
in56.22 seconds. **Strict package-only all-target Clippy failed**, exit101,
on ten distinct package sites. Dependency warnings remained separate/nonfatal.
The remaining host check and metadata-test gates and final post-Cargo source
guard were **NOT_RUN** after fail-closed stop. No model, finite qualification,
learning or evaluation command ran. The [compiler log](compiler-gate3/cuda-check.log),
[Clippy log](compiler-gate3/cuda-clippy.log) and
[independent outcome review](compiler-gate3/independent-outcome-review.md)
retain the exact evidence; this is a construction blocker, not a model negative.

The unchanged operator retained available PTX/cubins/resources and raw reaper,
launcher and setup provenance. All38 remote,21 operator and13 frozen packet
SHA entries passed, followed by complete cloud MD5 readback of the8,028,672-byte
archive `icloud:UOR-R4/results/codex/attention-compile-gate3-archive.tar`, MD5
`45e85311190317be14e31bbe2f707af1`, before release11:50:36Z/down11:50:42Z,
fleet0. Provider uptime270s at$0.977/hour estimates$0.073275000; separate
ledger wall273s estimates$0.074089167. Cumulative Codex provider-based estimate
is$3.813005000 plus PM; no invoice claim, storage/transfer UNMEASURED.
The closed release authorizes no retry.

The smallest source repair addresses those ten sites without lint suppression:
retain full exact-reachability admission and emit unused Rust reachability
arrays as audit comments; preserve all packed geometry words and kappas.
Replace reverse-filter parsing with equivalent `rfind`, use the validated
nonzero-context `is_multiple_of`, consume two final mutable-option borrows,
move the byte-identical redirect test module to the file end, fill a test
slice directly and simplify one constant test expression. No numerical CUDA,
initializer, scientific sheet, schedule/control/learning law, frozen completion,
timing3 artifact or binary changes. Source/composite/driver hashes deliberately
change with these Rust edits: rebuild and repin rather than reusing gate3's
compiler pass, old checkpoint lineage or execution identity. Host no-CUDA all-target check passed and the65 metadata/storage tests passed
(0 failures,1 ignored); [candidate source hashes](compiler-repair-host/CANDIDATE-SHA256SUMS)
and both logs retain their exact code identity. These host checks execute no
model/reference forward and make no strict CUDA lint claim. CUDA
construction at that repaired head remains NOT_RUN until a distinct released
compiler packet runs. No paid retry or on-lease source patch follows from this
record.

Finish exact-head source review and meaningful checks in this source unit, obtain
prospective fitness on the amended matched experiment, and obtain the distinct
applicable owner/PM execution releases and fresh resource admission. Source
preparation is not gated on another timing optimization programme. The existing
bounded timing closure and K16 draft remain separately preserved.

If relation proposals cannot affect loss, repair that particular credit path
before extending training. If relation helps conditionally but writer admission
collapses, preserve the read observation and isolate that writer interaction.
Only an attributable learned gain with the declared non-regression and fresh
transfer criteria earns a bounded learned-attention claim. No result here closes
a mechanism family, resets a line count or moves the suspended M1 headline.

The scientific sheet was delivered in parallel in [PR
#2574](https://github.com/UOR-Foundation/uor-r4/pull/2574), merged at `fb4519226`
with FIT (design), not an execution release. Source is aligned to its unchanged
production initializer and three-part ACTIVE/INERT/INIT LOCK construction
contract. Source/design reconciliation is recorded in [6230176389](https://github.com/UOR-Foundation/uor-r4/pull/2574#issuecomment-6230176389)
and [6230362901](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6230362901).
The real training transition makes one `Device::propose` call per batch; the
separate virtual-only `propose_next` API is not called by that transition.
A cost projection that adds both APIs as if both execute each batch doubles the
proposal component. Actual Relation throughput is still unmeasured; neither a
K16 diagnostic call nor older Route throughput is a current Relation budget.


### Gate4–gate6 closure and the confined final lint repair

Gate4 (release6233862027) stopped at SSH publickey authentication before setup;
all four Cargo gates were NOT_RUN. Its native early-startup failure deletes before
the up-failed ledger event, so an outer RETAIN message was not live retention.
An available local snapshot was SHA/cloud-verified before deletion; later archives
and a separate chronology correction were sealed afterward. Remote logs/inventory
were UNAVAILABLE. PM6234312088 accepted that explicit preservation boundary.
Matching local/private-derived/account key fingerprints and correct static endpoint
pairing did not establish the failed host's actual key injection/root cause.

Gate5 (release6234814065) stopped before SSH/setup after an unusable CREATE reply;
exact-label reconciliation found own stopped56404709. The available archive
363520bytes/MD5eebf1b7e5aa7f1bb2b9fd11040997986 was verified before down14:19:40Z;
fleet0. Raw CREATE response was unavailable, cause UNVERIFIED. Independent8859f9ce
and PM6235067540 accepted the closure; the latter commissioned only diagnostics
and bounded stopped-instance handling. This was not a second publickey failure.

Gate6's distinct [release6235906058](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6235906058)
bound source6ca599a536f6887e0f52afd1ee9efd13a0a27d38, reviewed24-member packet
c5f70217d3077fa8a1ef40354ffa9f4c2ef7414eadb8e6314c97e19f3a013a1d and readiness
aa2398288fa436efbc2a6a1b9f2df04ded02cddef179840e8ee27dd8d0aec590. Fresh full
floor/reserve and fleet/rate passed. Own vast-56414377/RTX5090/Czechia CZ/$1.725h
reached SSH (native167s) and setup0. The initial exact-source/index/worktree guard
passed. [CUDA all-target check](compiler-gate6/cuda-check.log) passed0/47.91s;
[strict package Clippy](compiler-gate6/cuda-clippy.log) failed101 at three distinct
prime_zeta.rs sites119/349/382. Binary and test messages repeat the same sites.
The two host metadata gates and final source guard were NOT_RUN after fail-closed
stop. No CUDA test binary, finite/model/qualification/learning/evaluation ran.
The [independent actual review](compiler-gate6/independent-outcome-review.md),
SHA04dda4b1104cc65ae1734c039982f4b044dae2a23fded2bc2b531199623725dd,
accepts the construction blocker and archived closure.

All38 remote+43 operator+24 packet SHA checks passed, followed by full cloud MD5
of8152576bytes/4d48c7556e35ecd0aafcca6558a94a52 at
icloud:UOR-R4/results/codex/attention-compile-gate6-archive.tar before release15:52:45Z
and down15:52:51Z/fleet0. Receipt and release share a whole second; operator
sequencing establishes their order. A separate post-down four-member lifecycle
supplement is23552bytes/MD537190b09abfe500159891a42dccfd969. Raw diagnostic
responses and reversible encoding remain private, outside these public source
receipts. Native normalized elapsed338s gives$0.161958333; up/downwall341s gives
separate$0.163395833. Neither measures actual billed/active-GPU time. Cumulative
mixed historical Codex estimate$4.320722222 plus PM; storage/transfer UNMEASURED.

[PM6236268062](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6236268062)
accepted gate6 and approved exactly the three lint-only repairs. Pre-edit
prime_zeta.rs is byte-identical to fresh origin/main; last-changing commit
7b26da89def32ff6ea98f0b04d099ddfc2ae2d9b/#2525. The divisor is a known nonzero
prime; is_multiple_of preserves its predicate. Enumerating the16 scores preserves
root order; iterating interior alignments preserves the old1..blocks-1 range,
descending-score ordering and ascending-root tie-break. No waiver or numerical
CUDA/scientific sheet/initializer/learning/control/default change. These edits
change source/executable identity, so gate6's old-head pass cannot qualify the
repaired head. Gate7 requires confined AGY delta review, exact packet rebind and a
distinct PM dispatch, then one transcript containing all four original commands.
No automatic retry, model permission or learned-attention result follows.

Local repaired-candidate all-target no-CUDA check and rustfmt PASS. An additional
strict package-only no-CUDA Clippy attempt is BLOCKED by feature-inactive
dead-code diagnostics; this is not the accepted CUDA Clippy gate and is not
expanded into unrelated repair. Retained [logs and candidate identity](compiler-gate6-repair-host/scope.md).
They execute no generator/model/reference and do not replace the final CUDA
transcript. Existing metadata tests were not repeated for these three idioms.

PM6236462377 requires merge-forward of #2579/d3a4013ad owner resolutions(a)/(g).
The merge retains the already-approved post-init literal-storage declaration
from this source branch and adds only the owner leaf-only/one-seed resolutions.
Main #2579 still carries the superseded internal-initializer sentence; it is not
reintroduced into this source. The actual merged corrected sheet SHA is
9e74331701aba3e124f85a34dcd677b25a4290f9d233cd465579e55ad9141f46; ATTENTION_SHEET_SHA256 binds those exact bytes.
This differs from main57d74842 because the accepted declaration correction is
preserved. No source initialization, dose law, control or numerical CUDA change.
