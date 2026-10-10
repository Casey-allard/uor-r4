# Saved-basis residual-qualified numerical repair — October 9

## Question and fixed policy

Can a generic, scale-aware LU singularity rule cross the two captured numerical
failures from [#2132](../legal-basis-diagnosis-2026-10-09/README.md) while retaining
accurate solves against the original matrices? This is an offline constructor
boundary, not a serving operator or a new learned representation.

[Prospective claim](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093025651).
Base main: d4628eff9362ce3c8cef7de92245b66e80ab2c7a.

For dimension n and machine epsilon u, use the fixed threshold
tau_j = 32*n*u*max(original basis column maxabs, computed residual column maxabs).
Retain the backend's 0.1 pivot preference and reject zero/nonfinite or insufficient
pivots. Global simplex EPS, all coefficients, 1,920 coordinates, 380 guards,
legal destination domains and the 4,096 branch-node bound remain unchanged.
The constants are a prospective numerical policy, not a theorem or exact-rank test.

Before accepting each fresh factorization, check every unit right-hand side in
normal and transpose orientation against the original CSC basis. Require finite
arithmetic and normwise backward error

    ||b - B*x||inf / (||B||inf*||x||inf + ||b||inf) <= 128*n*u

with the corresponding transpose norm for B^T. A zero denominator passes only
with zero residual. No iterative refinement or threshold sweep is included.
Small backward error does not establish a condition number or small forward error.
Later eta updates are outside this fresh-factor guarantee.

## Artifact and execution scope

Original saved input SHA256:
0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d.
It retains original f32 master, objective and 380 protected derivative rows from
#2101; the harness reconstructs the same ordered f64 unit rows. Zero new gradients.

Captured 440 by 440 basis SHA256 values:
- c5c73e1b9f7baf598cdda33e4a458f9208f0c943b4d471ba646eb129c495a794
- 55d8c2eadbcfc072d86a6310602184603030a318986ebeb2a2d885958f10ae05

All 235 manifest-listed payload files were authenticated after cloud-store's MD5
round-trip of codex-legal-basis-diagnosis-20261009.tar
(MD5 32dcdd9a0f2aa1092430490d754f49e2, 13,264,384 bytes).

Production constructor source is included unchanged, SHA256
f0242836d08dbadbb5659cd8f158d79c8de9420c9de2fb213de9ef149510c24b.
The isolated local backend is microlp 0.6.0 plus the corrected #2132 observer and
this explicitly identified numerical patch. An unchanged nested constructor
receipt's microlp=0.6.0 field names its base version, not unmodified upstream.
No Cargo registry or production serving dependency is modified.

First qualify both captured bases. Only after both pass and source review passes
may one unchanged saved constructor replay run. Any screened offer must still
pass the native transaction, export/reload and immediate actual-artifact checks.
A numerical rejection is not a model negative; bounded no-offer is not global
infeasibility. Native qualification has not been performed merely because a
standalone saved constructor returned an assignment.

## Resources and delivery

Projection: 60 minutes preparation, build, review, conditional execution and
protected delivery; two CPU threads, at most 6 GiB RAM, 4 GiB temporary and
512 MiB retained evidence. Disk floor: 30 GiB plus 128 MiB margin. No pods or paid
compute. This saved CPU-only numerical/native work follows the checked-in
October 9 compute clarification. The shared ledger was charged through
2026-10-10T02:54:19.500371+00:00 and extended 60 minutes under standing same-class
owner authorization before execution, preserving previous charges.

## Measured result and decision

**KEEP the scoped numerical repair/evidence; backend integration NOT YET PROMOTED.**
Both original matrices now qualify, but the unchanged saved constructor still
returns no assignment. This repair is not sufficient to make the legal constructor
operational. No native proposal was scored, no update was committed and no language
quality was measured. Accepted 8/512 complete development replies and the separate
conditional 9/15 artifact remain unchanged.

Executed source freeze: 1fe9602d22fa19d9c8765952c6fb20a4a6f45ac7.
Binary SHA256: 0e929bcf61b9107d0f1d904b035dbcc1f77b2b96cd3e5c6621fd9be71d97c7c3.
Repair source SHA256: 5d7e0da370bc11131bbf54d220446f6354159cdc61e60e512fdef9f3b3733fd4.
Installer SHA256: 0e40fd0210ffd4937c70f0153fdbb900ffbe17f90f792d13933bccf20601a297.
Complete compact outcomes and per-path errors: [result.json](result.json).

Each 440-dimensional captured basis passes 1,760 unit solves: 440 RHS times
normal/transpose times dense/sparse. These are matrix qualifications, without a
training/evaluation split or language artifact execution.

| Captured original basis | Max normal backward error across dense/sparse | Max transpose backward error across dense/sparse | Fixed tolerance | Qualification seconds |
| --- | ---: | ---: | ---: | ---: |
| #2132 first failure, 33,514 stored entries | 8.554049903446984e-17 | 5.430635821225957e-17 | 1.2505552149377763e-11 | 0.163697209 |
| #2132 slack recovery, 16,737 stored entries | 3.55350733423591e-17 | 4.127080277298578e-18 | 1.2505552149377763e-11 | 0.132780 |

The minimum chosen-pivot/threshold ratios are 29.215817 and 118871.219192;
observed maximum residual/original column growth is 4.007091 and 6.330084.
These ratios do not compare unscaled pivots: the second old rejected pivot's
original column has maximum magnitude about 6.30e-6. No exact elimination-path
identity or condition-number claim follows.

The one gated saved-constructor replay took **27.046445750 seconds** inside the
harness (27.167279459 seconds supervised), peak child RSS 82,870,272 bytes.
It recorded **150 factorizations, 148 completed original-basis verifications,
and two later pivot rejections**. No residual check rejected an accepted factor.

| Later rejection | Phase | LP iteration | Elimination column | Maximum eligible pivot | Scale-aware threshold |
| --- | --- | ---: | ---: | ---: | ---: |
| Factor 134 | branch feasibility after 322 completed branch-node LP visits | 4799 | 393 | 6.028782215899844e-22 | 4.663019847385633e-12 |
| Factor 150 | existing all-slack recovery at the same branch visit | 5034 | 439 | 2.8275992658421956e-15 | 5.573822218590424e-12 |

Both new original CSC matrices and full failure context are sealed. Their SHA256
values are 337e0917af3a3c397ce70db1d096f24adc07c054de32a5cf6e1ed6c23491e221
and 4b5abc63cc98cde36c8ff1ba47f1e9388548f81dad3cb26bf15e73bfb2fbd2b6.
The 322 count still denotes completed LP branch visits, not candidates; the failing
visit never completed. More successful factorizations did not advance this counter.

Final backend status is BACKEND_NUMERIC_OR_NO_INCUMBENT, termination
InternalError("Singular matrix"). Wrapper exit zero means completed sealed
measurement, not constructor success. The exact constructor returns no assignment,
destination or eligible proposal. Actual-nine, native export/reload, full512,
fresh and multi-turn checks are NOT_RUN for this unit. Existing model evidence is
retained, not rerun to relabel this numerical failure as a model result.

## Validation and reproducibility

Five external focused repair fixtures actually executed and passed: rescaled small
pivots; nonsymmetric normal/transpose and sparse-zero solutions; singular/malformed
input; overflow during solve/residual denominator arithmetic; and genuine factors
rejected against a changed original basis. The dependency's internal cfg(test)
module was not executed. Thirteen harness tests passed: nine unchanged constructor,
two mapping and two output-claim/seal fixtures. Source review caught and corrected
sparse-output initialization and worst-RHS bookkeeping before these actual bases ran.
Two setup failures (workspace membership and uncached upstream test dependencies)
are retained; the isolated fixture manifest avoids unrelated upstream dev dependencies.

Run the corrected prior observation installer on its pinned cached microlp source,
then this record's apply-repair.py. The installer authenticates and preserves the
observed copy; the new module, changed backend files, Cargo files and actual binary
are independently identified. Set CARGO_TARGET_DIR inside local/ and build with
two jobs using this record's harness/Cargo.toml and tests/Cargo.toml.

Restore codex-legal-basis-diagnosis-20261009 under local/restore using cloud-store;
authenticate its preservation-manifest.json and copy input/input.json to local/input.
run.py basis1 and basis2 claim exclusive outputs and bind their original hashes,
source, executable, reports and manifests. run.py constructor admits execution
only after both exact matrices qualify with the same executable and their sealed
roots reverify. Each run remains exclusive; no output root is reused.

The unpromoted dependency patch is also preserved in
[the branch archive](../../history/branch-archive/basis-residual-repair-20261009.patch)
and its INDEX. It applies to the corrected observed microlp copy; git apply --check
passed on that exact base. No production Cargo override or runtime change is activated.

No geometric role, fact, model capability or milestone status changed. Geometry
figures, STATUS, ROADMAP, tracker table and top-level README therefore retain their
existing scoped claims. The training README and prior diagnostic record link this
follow-through. No blanket tests, new training or new GPU job were needed.


## Next

Observe the eta-updated solve and pivot transition that precedes a rejected fresh
basis, with the numerical policy and all model constraints unchanged. Distinguish
the exact reset caller. Before basis mutation, retain the original **old** basis,
actual entering-column and transpose unit RHS, eta chain, selected entering/leaving
variables and row/column pivot coefficients. Compare their residuals and, where
factorable, fresh solves against that same old basis. In pivot(), membership changes
before reset, so comparing the old eta chain to the new rejected basis is invalid.

Failure contexts contain 33 and 38 pending eta updates respectively. This motivates
the comparison but does not establish eta drift as the cause. If eta residuals fail
while fresh old-basis solves pass, a generic residual-triggered refresh is justified
for subsequent testing. If they agree, inspect pivot acceptance/near-dependence.
A stalled-feasibility refresh without a proposed replacement needs its own current
basis/value residual, not a fabricated last-pivot cause. No new numerical tolerance,
node sweep, guard exception or model run is authorized by this result alone.
