# Seed-2 guard-integration appendix — executor input, 15 October 2026

**Docs-only; every build/CUDA/run command below is prospective and NOT_RUN here.**
Claude owns execution and the re-issued build/qualification sheet after cursor16,404;
Codex owns the checkpoint-zero guard. The build/qualification needs its own sheet,
FIT and Casey's personal authorization; seed-2 training needs its separate personal
release. This appendix grants neither. M2 stays HELD; released seed-1 is untouched.
Source inspected: `origin/main` `ff658789c`; guard merged by
[#2484](https://github.com/UOR-Foundation/uor-r4/pull/2484), commit `5ce408c6d`.

## Inputs and invocation

Re-issue the [completion plan](../rung0-completion-release-2026-10-14/SEED2_PLAN.json)
with seed2, `pilot_parent:null`, scope `m1-a2-completion-seed2-checkpoint0..16404`,
the frozen Route/V4096/B4/C192/kept8/batch64 configuration and canonical schedule.
Keep TRAIN/DEV/tokenizer FilePins and exact qualified numerical source/PTX/geometry
from the frozen pilot; `completion::validate_plan` refuses a changed numerical
identity. Re-pin reviewed source, executable, driver source, config, lineage,
sheet and Claude review from the **new** build. The old seed-2 packet's
`cb090e85…` driver pin predates the guard and is not reusable. New metadata source
may change lineage/binary identity without permission to alter numerical code.
The driver digest includes `completion_baseline.rs` (`training.rs:19–28`).

The new `CompletionPlan` carries absolute `run_state_directory`, pinned FIT sheet,
and pinned Claude `ReviewRecord` with verdict `APPROVE_A2_COMPLETION_ADMISSION_DIFF`,
matching scope/execution and review URL. Stage original source-bound data at its
config paths. After the admitted build, capture `discrete-stack
completion-build-identity CONFIG` (JSON stdout, no CUDA context) and
`completion-check-plan PLAN` (`PLAN_METADATA_VALID`, `owner_release:STILL_REQUIRED`).
Casey alone invokes `completion-owner-release PLAN NEW_RELEASE_DIR`; labs cannot
invoke it or inherit D33's pilot delegation.
Casey's release shell must have `UOR_LAB` **unset**, with both stdin/stdout on an
interactive terminal. `NEW_RELEASE_DIR` and the plan's `run_state_directory`
must both be absent. Owner-release creates the release directory; the later
`completion` invocation creates the state directory when acquiring its run lock.
Create only their parents during authorized setup. The release's `admission.json`
is the input to:

```text
discrete-stack completion CONFIG NEW_RELEASE_DIR/admission.json NEW_ATTEMPT_DIR
# Authorized recovery only; same plan/state, a new exclusive attempt directory:
discrete-stack completion-resume CONFIG NEW_RELEASE_DIR/admission.json NEW_ATTEMPT_DIR
```

**These are corpus-run commands, not guard-only probes.** `completion::run` validates
release → creates seed2 model and GPU DEV baseline → saves checkpoint0 → invokes
`baseline_authentication::require_before_apply` (`completion.rs:636–678`) → issues
corpus-update admission → enters transitions. There is no standalone authenticate-and-stop
CLI. Resume rechecks the immutable initial baseline before further apply; it never
substitutes the resumed model's baseline. The guard itself only authenticates stored
artifacts/observations; it does not prove GPU execution or perform a CPU reference.

## Re-issue qualification inventory — all 25, plus emitted-PTX admission

Freeze the new executable and explicit `UOR_NVCC_ARCH`/nvcc version in the sheet.
The CUDA feature build automatically runs `build.rs::admit_ptx` on emitted
`OUT_DIR/discrete.ptx`, rejecting float/tensor/exponential tokens and a missing,
unexpected or duplicate entry inventory. Preserve build log, arch, PTX and binary
SHA-256; there is no standalone `admit_ptx` CLI or success JSON receipt. A non-CUDA
build's lexer self-test is **not** emitted-PTX qualification.

| Inventory command → run command (each run gets a fresh directory) | Required exact controls |
|---|---|
| `list-controls` → `qualify ROUTE_DIR` | `exact_writer`, `virtual_proposals`, `context_transport`, `isolated_windows`, `collision_and_omitted_target`, `nonvacuity`, `artifact_and_identity`, `inventory_bounds`, `variable_length_prefixes`, `target_independent_admission`, `token_relabelling_preserves_geometry` |
| `list-flock-controls` → `qualify-flock FLOCK_DIR` | `flock_identity_read`, `flock_gate_counterfactual`, `flock_transport_witnesses`, `flock_payload_witness`, `flock_selection_and_eligibility`, `flock_refusals_and_noop` |
| `list-driver-controls` → `qualify-driver DRIVER_DIR` | `isolated_apply_boundary`, `route_apply_reality`, `gate_apply_reality`, `fixture_loop_resume`, `native_top8_ties`, `count_reference_excludes_dev`, `seeded_initialization`, `corpus_schedule_resume` |

All are `discrete-stack` subcommands; do not append an unsupported `--gpu` flag.
Names must match compiled lists11/6/8. Preserve per-control JSON stdout, stderr and
exit status, `ROUTE_DIR/controls.json`, `FLOCK_DIR/flock-controls.json`, and
`DRIVER_DIR/controls.json`: registered/attempted/passed11/6/8, failed/not_run0,
overall PASS; Route/Flock applied_updates0; driver apply_calls11,
applied_updates8, corpus_updates0, apply_dose_matches=true. These are finite
fixture updates, not a corpus-training release. A CUDA fault hard-stops its suite;
remaining entries become NOT_RUN. Stop further suite launches on any failed suite;
never reinterpret a generic CUDA fault as a successful domain refusal. The
[admitted finite sheet](../../harness/prep/m1-discrete-driver-finite-2026-10-13.md)
owns their literal bars. Cost measurement is separate, not silently added here.

Additionally list the guard's **metadata** controls separately from those25:
`zero_baseline_binds_model_rows_reference_and_seed_without_execution`,
`high_u_is_authenticated_but_instrument_invalid_not_a_model_negative`,
`typed_and_value_serialization_are_distinct_digest_domains`, and
`authentication_publication_is_immutable_and_resume_fails_closed`.
`reference_pin_matches_authenticated_pilot_when_supplied` is ignored by default:
explicitly select it with `--ignored`, supplying `UOR_BASELINE_PILOT_REPORT` whose
file SHA is `e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3`.
Retain test names/results; do not count a default skip as a pass. These tests
validate metadata/storage only; they do not exercise the CUDA-gated typed-Snapshot
integration on the future seed2 artifact.

Metadata-only selection for that future qualification: `cargo test -p
uor-r4-discrete --no-default-features --lib completion::baseline_authentication`
for the four ordinary controls; then the same command with filter
`reference_pin_matches_authenticated_pilot_when_supplied -- --ignored` and the
pinned `UOR_BASELINE_PILOT_REPORT` for the fifth. These are separate from the
25 controls on the qualified CUDA executable; all remain NOT_RUN in this appendix.

## Checkpoint-zero receipts and refusal semantics

Let `STATE` be the newly pinned run-state path and `ATTEMPT` the exclusive run output:

| Path | Expected content |
|---|---|
| `ATTEMPT/checkpoint-00000.json` | `uor-r4.completion-checkpoint/1`; initial cursor, model epoch0, zero VectorBlock/Route counters, null pilot parent, baseline evaluation and snapshot metadata |
| `STATE/initial.json`, initially also `STATE/latest.json` | `FilePin {path:absolute checkpoint path, sha256:checkpoint bytes}`; initial stays immutable, latest advances |
| `STATE/baseline-authentication.json` | `uor-r4.completion-seed2-baseline-authentication/1`, status `AUTHENTICATED_METADATA_BEFORE_APPLY`, seed2, plan/admission/initial/driver-checkpoint/lineage/evaluation/model-storage/typed-snapshot/reference digests; `u:{numerator,denominator:14287,definition,instrument_invalid}`, `target_admitted_rows`, `release_effect:"NONE"` |
| `STATE/baseline-authentication.pin.json` | `FilePin` of the authentication receipt's exact bytes and canonical path |

Whole baseline reference digest is frozen at
`b92da0bf1ccda1c9de56a33de41d4698dc214bd71c588a6938a87443b9aca0b9`.
The guard recounts 224 stored DEV batches (223×64 +15 rows), exact weighted losses,
rank/admission nesting and aggregate counts. `u` counts `token_rivals>0 &&
token_hinge<32` without requiring admission; seed1's105 is never reused. JSON-Value
model storage and typed Snapshot serialization are separate digest domains.

On mismatch it returns an error before update admission: wrong seed/parent,
nonzero cursor/counters/epoch, changed model/lineage/reference/rows/baseline,
wrong initial FilePin, altered authentication receipt/pin, or progressed resume
missing either authentication file. An intact checkpoint0 may recover a missing
publication; a progressed checkpoint may only verify it. CLI errors print to
stderr and exit2; guard assertions print `invalid discrete model: seed2 baseline:
<reason>` (e.g. `progressed resume missing baseline authentication`). File/JSON
errors have their underlying diagnostic, not necessarily that prefix. Failures
inside the run's work closure write `ATTEMPT/failure.json` when writable:
`status:INSTRUMENT_OR_EXECUTION_FAILURE`, error, `mechanism_negative:false`,
attempt_elapsed_ns, resume and training fields. Earlier plan/release/path/lock
failures need not produce it. Preserve stdout/stderr and partial files as evidence.
There is no separate success print from the guard; inspect the two receipt files.

**High-u distinction:** `100*u > 95*14287` authenticates metadata and sets
`instrument_invalid:true`; it does **not** cause automatic refusal or pause before
training. It is an interpretation/instrument flag, not a model negative. If the
new sheet requires stopping on this flag or an interactive checkpoint0 review,
it must specify a separately reviewed control change; current CLI must not be
represented as providing that behavior.

Next: Claude cites this appendix in its new sheet after16,404; exact new pins and
controls receive review/FIT and personal release before execution. Q4 Route-row
analysis uses #2492 once the final receipts seal. Seed2 support waits for Casey's GO.
