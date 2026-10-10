# Exact legal-constructor basis diagnosis — 9 October 2026

## Question and fixed scope

The [direct legal-set constructor](../direct-legal-construction-2026-10-09/README.md)
returned `InternalError("Singular matrix")` for both absolute and equivalent
centered encodings. Which solver phase and specific selected basis fail, and is
the rejection an empty stored column or a nonzero pivot below the backend's
threshold? This is a saved-problem execution diagnosis before any numerical
repair. It is not a new training attempt or a model-quality comparison.

[Prospective M2 claim](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6092679883).
Use the unchanged centered constructor, all 1,920 original master coordinates,
20 fractional side encodings, 380 unit-normalized guard derivatives and original
objective. All 1,960 variables, 440 constraints, 15 legal destinations per
coordinate, zero-margin construction target, zero warm start and 4,096 branch-node
bound remain. No coefficient pruning, tolerance change, new gradient, native
scoring, candidate commitment, or held-out use.

The standalone Rust harness includes the production constructor by path. The
backend is pinned `microlp=0.6.0`; observation-only instrumentation applies to a
hash-verified local copy, never the shared Cargo registry. It records exact f64
bits for the selected basis and failed pivot context. The diagnostic patch is
retained research tooling, not a new production or serving dependency.

## Artifact recovery

`prepare.py` verifies every file in the saved #2101 attempt3 manifest and the
SHA256 of each required raw file, then packages their f32 bit patterns without
model arithmetic. Rust performs the same ordered normalization as the production
`protected_joint_vector::unit_rows` function. The original artifact is development
input245, 17 references and 380 original guards; it is not the separate 9/15
conditional artifact or a fresh evaluation.

- Archive: `icloud:UOR-R4/results/codex/codex-protected-joint-20261009.tar`,
  MD5 `467c102adc510a6352951596b969f1be`.
- Saved report SHA256: `b1d27f7b15e65d0aa8ea7e0a995b8bc76979610f97dbcb3055c8bbb434d0ac9b`.
- Saved manifest SHA256: `cdadd7c5cbbbdca2225a71006ab58f70129e8df9e015b92f35b19359028a7167`.
- 1,676 complete manifest files and 384 required raw files verified.
- Bit-container SHA256: `0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d`,
  3,925,770 bytes. The full receipt and original guard map remain with the evidence.

## Measured result and disposition

**KEEP the diagnostic and sealed execution evidence. Constructor NOT YET PROMOTED.**
The corrected trace reproduces `InternalError("Singular matrix")` and captures
100 factorization calls and two numeric LU failures. Root initialization and root
relaxation completed. At both failures, the counter records **322 previously
completed branch-node LP visits**, excluding root, nodes pruned without solving,
and the current failing visit. It does not count native/model candidates.

| Observation | First branch solve | Existing all-slack recovery |
| --- | ---: | ---: |
| Factorization sequence | 90 | 100 |
| LP phase / iteration | feasibility / 2,950 | feasibility / 3,024 |
| Pending eta updates before reset | 33 | 13 |
| Basis dimension / stored entries | 440 / 33,514 | 440 / 16,737 |
| Empty stored basis columns | 0 | 0 |
| Elimination column (zero based) | 439 | 433 |
| Maximum eligible residual pivot | 9.133998119987652e-11 | 6.8006852399509905e-12 |
| Backend absolute rejection threshold | 1e-10 | 1e-10 |

The first residual is nonzero on remaining row73 (original guard13), with a
Generate unary lower-side binary as the eliminated basis column. The second has
seven nonzero remaining residuals and eliminates a Prefix displacement column.
The exact original coordinate, guard and basis mappings are in [result.json](result.json).
These identify the observed elimination context, **not defective coordinates or
guards**; no named coordinate or guard receives an exception.

The backend already retries a numerically failed branch LP once from its all-slack
basis. The corrected trace directly labels that recovery, which advances through
additional iterations before failing a different selected basis. Another blind
encoding change or identical slack restart is therefore not the next task.

The first diagnostic source `bfd6adb0e8ba7675df0564c8bc292404aab03341` completed in
3.886 seconds, peak child RSS92,438,528 bytes. Review found its fallback phase label
attached to the integral-candidate reset rather than the branch-error reset; the
first trace's fallback attribution was source-inferred. A label-only correction,
recorded prospectively on the same claim, produced source
`7363273b4aa199778479b8c9e103da92f91eb5bb`. Its fresh trace took 3.600 seconds with
peak child RSS94,076,928 bytes. Both captured basis files are byte-identical across
traces; pivot bits, iterations and variable mappings are identical. Only the
second event's phase label changes to directly observe branch slack recovery.
See [the comparison](label-only-comparison.json) and [first result](attempt1-result.json).
The full original diagnostic source is preserved as an [indexed patch](../../history/branch-archive/legal-basis-initial-label-20261009.patch).

Both traces return zero assignments/proposals. There are zero new training,
backward, native-scoring or candidate-acceptance calls. The enclosing diagnostic's
exit0/COMPLETED means the sealed trace was produced, not that the solver succeeded.
Exact matrix rank, conditioning and global feasible-set claims remain unmeasured.

## Reproduction and checks

Restore the indexed original archive with `cloud-store fetch
codex-protected-joint-20261009 local/restore` while setting `CLOUD_STORE_STAGE`
inside the owned worktree. Run `prepare.py`, then `apply-observer.py --upstream`
with the pinned Cargo-registry microlp0.6.0 source directory. The installer verifies
all upstream file hashes and refuses an existing destination; it only changes its
new `local/microlp-0.6.0` copy. Build the standalone `harness/Cargo.toml` with
`--release --locked --offline`, two jobs and `CARGO_TARGET_DIR` inside `local/`.
The harness has its own pinned lockfile and does not build the training model.

`run.py 2` runs the current diagnostic into an exclusive root; `analyze.py 2`
verifies its complete seal and decodes the saved bit receipts. To reconstruct the
first diagnostic exactly, apply its archived patch to the stated base and use the
first patch/constructor identities. Do not run a third unchanged trace.

All **14 focused fixtures passed**: nine unchanged constructor fixtures, two
mapping/normalization fixtures, two native report-output fixtures, and a capture
fixture exercising real symbolic and numeric LU failures. Run tests with one
thread and `UOR_DIAGNOSTIC_TEST_ROOT`/`TMPDIR` under `local/`; observer state is
process-global. The initial build's missing public-function documentation error
was repaired before any actual replay and its log is retained; it is not model
failure evidence. [checks.json](checks.json) pins final code, test and executable
identities. Independent source review and saved-result mathematical review found
no solver-arithmetic change or model-quality inference.

The final runtime SHA256 is
`1bba40a7fb5fad023ad71c81aaa24f361307002b827a9add2e399647fb6a1664`.
Both output roots are sealed with the unchanged native report module; all eight
files per trace were reverified independently by the saved analysis. The complete
input container, captures, final runtime, source freezes and build logs are
retained in the iCloud archive indexed by `storage.json`; the original large
artifact remains separately retained rather than duplicated.

## Interpretation boundary

A small computed pivot localizes a numerical rejection; it is not an exact-rank
proof, condition-number estimate or global infeasibility result. A computed zero
can arise from floating-point elimination. Backend diagnostics do not qualify a
model or weaken the original actual-bit screen and native acceptance gates.
Accepted Source48/Generate64 remains 8/512 complete development replies; the
separate conditional 9/15 artifact remains unchanged. Actual-nine, full512, fresh
and multi-turn qualification are not part of this diagnostic.

## Resources

Admission projected 45 minutes for preparation, compile, replay, review and
protected delivery; two CPU threads, 4 GiB process RAM, at most 3 GiB new temporary
storage and 512 MiB retained evidence. Free space at admission was 40,019,226,624
bytes against the 30 GiB plus 128 MiB floor. The existing shared ledger was
1,459,465,346 ms used / 1,463,400,028 ms allowed; charges continue from
2026-10-10T02:18:26.318584Z. No GPU, pod or paid compute; no new model derivatives
or native grading.

**Next:** Repair the generic basis-factorization numerical boundary using the two
saved failing bases: assess a scale-aware pivot/rejection policy or robust
factorization with scale-aware solve-residual verification against the original
captured basis matrix before trusting a basis (not just consistency with the
computed factors). Preserve
all coefficients, constraints, branch bounds and native acceptance. Do not merely
lower global EPS, prune tiny coefficients, add guard exceptions or repeat the
existing slack retry. A backend repair must first validate the captured basis
solves and then demonstrate a screened legal offer before any language claim.


## Follow-through

The [residual-qualified numerical repair](../basis-residual-repair-2026-10-09/README.md)
now passes all 1,760 dense/sparse normal/transpose unit solves on each captured basis.
One unchanged saved-constructor replay nevertheless rejects two later bases after
148 completed fresh-factor checks and returns no assignment. The original diagnosis
remains valid; the backend is still unqualified and no language result follows.
